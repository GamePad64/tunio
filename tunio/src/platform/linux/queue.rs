use crate::traits::{QueueT, TokioQueueT};
use crate::Error;
use futures::ready;
use netconfig::sys::posix::ifreq::ifreq;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fs, io};
use tokio::io::{unix::AsyncFd, AsyncRead, AsyncWrite, ReadBuf};

impl QueueT for Queue {}
impl TokioQueueT for Queue {}

mod ioctls {
    nix::ioctl_write_int!(tunsetiff, b'T', 202);
    nix::ioctl_write_int!(tunsetpersist, b'T', 203);
    nix::ioctl_write_int!(tunsetowner, b'T', 204);
    nix::ioctl_write_int!(tunsetgroup, b'T', 206);
}

pub struct Queue {
    tun_device: AsyncFd<fs::File>,
}

impl AsRawFd for Queue {
    fn as_raw_fd(&self) -> RawFd {
        self.inner_file().as_raw_fd()
    }
}

impl Queue {
    pub(crate) fn new(name: &str) -> Result<Queue, Error> {
        let tun_device = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open("/dev/net/tun")?;

        let init_flags = libc::IFF_TUN;

        let mut req = ifreq::new(name);
        req.ifr_ifru.ifru_flags = init_flags as _;

        unsafe { ioctls::tunsetiff(tun_device.as_raw_fd(), &req as *const _ as _) }.unwrap();

        Ok(Queue {
            tun_device: AsyncFd::new(tun_device)?,
        })
    }

    fn inner_file(&self) -> &fs::File {
        self.tun_device.get_ref()
    }

    fn inner_file_mut(&mut self) -> &mut fs::File {
        self.tun_device.get_mut()
    }
}

impl Read for Queue {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner_file_mut().read(buf)
    }
}

impl Write for Queue {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner_file_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner_file_mut().flush()
    }
}

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
