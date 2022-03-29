use crate::linux::ifreq::{ifreq, siocgifflags, siocsifflags, tunsetiff, IfName};
use crate::linux::params::LinuxInterfaceParams;
use crate::linux::LinuxDriver;
use crate::{Error, InterfaceT};
use futures::ready;
use libc::IFF_TUN;
use std::ffi::CString;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{fs, io};
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct LinuxInterface {
    driver: Arc<LinuxDriver>,

    tun_device: AsyncFd<fs::File>,
    socket: std::net::UdpSocket,
    name: String,

    running: bool,
    init_flags: libc::c_int,
}

impl InterfaceT for LinuxInterface {
    type DriverT = LinuxDriver;
    type InterfaceParamsT = LinuxInterfaceParams;

    fn new(driver: Arc<Self::DriverT>, params: Self::InterfaceParamsT) -> Result<Self, Error> {
        let tun_device = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open("/dev/net/tun")?;

        let init_flags = IFF_TUN;

        let mut req = ifreq::new(&*params.name);
        req.ifr_ifru.ifru_flags = init_flags as _;

        unsafe { tunsetiff(tun_device.as_raw_fd(), &req as *const _ as _) }.unwrap();

        Ok(Self {
            driver,
            tun_device: AsyncFd::new(tun_device)?,
            socket: std::net::UdpSocket::bind("[::]:0")?,
            name: params.name,
            running: false,
            init_flags,
        })
    }

    fn is_ready(&self) -> bool {
        self.running
    }

    fn open(&mut self) -> Result<(), Error> {
        self.set_flags(libc::IFF_UP | libc::IFF_RUNNING)?;

        self.running = true;
        Ok(())
    }
}

impl LinuxInterface {
    fn set_flags(&mut self, flags: libc::c_int) -> io::Result<()> {
        let mut req = ifreq::new(&*self.name);
        req.ifr_ifru.ifru_flags = (self.init_flags | flags) as _;

        unsafe { siocsifflags(self.socket.as_raw_fd(), &req) }?;

        Ok(())
    }
}

impl AsyncRead for LinuxInterface {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let _ = self.check_ready()?;

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

impl AsyncWrite for LinuxInterface {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let _ = self.check_ready()?;

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
        let _ = self.check_ready()?;

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
