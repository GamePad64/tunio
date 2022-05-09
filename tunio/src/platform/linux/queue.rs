use crate::config::Layer;
use crate::traits::QueueT;
use crate::Error;
use delegate::delegate;
use libc::{IFF_NO_PI, IFF_TAP, IFF_TUN};
use netconfig::sys::posix::ifreq::ifreq;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::{fs, io};
#[cfg(feature = "async-tokio")]
use tokio::io::unix::AsyncFd;

impl QueueT for Queue {}

mod ioctls {
    nix::ioctl_write_int!(tunsetiff, b'T', 202);
    nix::ioctl_write_int!(tunsetpersist, b'T', 203);
    nix::ioctl_write_int!(tunsetowner, b'T', 204);
    nix::ioctl_write_int!(tunsetgroup, b'T', 206);
}

pub struct Queue {
    #[cfg(feature = "async-tokio")]
    tun_device: AsyncFd<fs::File>,
    #[cfg(not(feature = "async-tokio"))]
    tun_device: fs::File,
    name: String,
}

impl Queue {
    pub(crate) fn new(name: &str, layer: Layer) -> Result<Queue, Error> {
        let tun_device = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open("/dev/net/tun")?;

        let mut init_flags = match layer {
            Layer::L2 => IFF_TAP,
            Layer::L3 => IFF_TUN,
        };
        init_flags |= IFF_NO_PI;

        let mut req = ifreq::new(name);
        req.ifr_ifru.ifru_flags = init_flags as _;

        unsafe { ioctls::tunsetiff(tun_device.as_raw_fd(), &req as *const _ as _) }.unwrap();

        // Name can change due to formatting
        let name = String::try_from(&req.ifr_ifrn)
            .map_err(|e| Error::InterfaceNameError(format!("{e:?}")))?;

        Ok(Queue {
            #[cfg(feature = "async-tokio")]
            tun_device: AsyncFd::new(tun_device)?,
            #[cfg(not(feature = "async-tokio"))]
            tun_device,
            name,
        })
    }

    pub(crate) fn name(&self) -> &str {
        &*self.name
    }

    pub(crate) fn tun_file_ref(&self) -> &fs::File {
        cfg_if::cfg_if! {
            if #[cfg(feature = "async-tokio")] {
                self.tun_device.get_ref()
            }else{
                &self.tun_device
            }
        }
    }
}

impl Read for Queue {
    delegate! {
        to self.tun_file_ref() {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error>;
        }
    }
}

impl Write for Queue {
    delegate! {
        to self.tun_file_ref() {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }
}

#[cfg(feature = "async-tokio")]
mod async_tokio {
    use super::Queue;
    use crate::traits::AsyncQueueT;
    use futures::ready;
    use std::io;
    use std::io::{Read, Write};
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    impl AsyncQueueT for Queue {}

    impl AsyncRead for Queue {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            let self_mut = self.get_mut();
            let mut b = vec![0; buf.capacity()];
            loop {
                let mut guard = ready!(self_mut.tun_device.poll_read_ready_mut(cx))?;

                match guard.try_io(|inner| inner.get_mut().read(&mut b)) {
                    Ok(n) => return Poll::Ready(n.map(|n| buf.put_slice(&b[..n]))),
                    Err(_) => continue,
                }
            }
        }
    }

    impl AsyncWrite for Queue {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            let self_mut = self.get_mut();
            loop {
                let mut guard = ready!(self_mut.tun_device.poll_write_ready_mut(cx))?;

                match guard.try_io(|inner| inner.get_mut().write(buf)) {
                    Ok(result) => return Poll::Ready(result),
                    Err(_would_block) => continue,
                }
            }
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            let self_mut = self.get_mut();
            loop {
                let mut guard = ready!(self_mut.tun_device.poll_write_ready_mut(cx))?;

                match guard.try_io(|inner| inner.get_mut().flush()) {
                    Ok(result) => return Poll::Ready(result),
                    Err(_) => continue,
                }
            }
        }

        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }
}
