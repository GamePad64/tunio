use crate::config::IfaceConfig;
use crate::platform::linux::ifreq::{ifreq, siocgifflags, siocsifflags, tunsetiff, IfName};
use crate::platform::linux::queue::Queue;
use crate::platform::linux::Driver;
use crate::traits::InterfaceT;
use crate::Error;
use futures::ready;
use libc::IFF_TUN;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fs, io, net};
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct LinuxInterface {
    socket: net::UdpSocket,
    name: String,

    queue: Queue,
}

impl LinuxInterface {
    pub(crate) fn new(params: IfaceConfig<Driver>) -> Result<Self, Error> {
        let queue = Queue::new(&*params.name)?;

        Ok(Self {
            queue,
            socket: net::UdpSocket::bind("[::1]:0")?,
            name: params.name,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl InterfaceT for LinuxInterface {
    fn up(&mut self) -> Result<(), Error> {
        self.set_flags(libc::IFF_UP | libc::IFF_RUNNING)?;

        Ok(())
    }

    fn down(&mut self) -> Result<(), Error> {
        todo!()
    }
}

impl LinuxInterface {
    fn set_flags(&mut self, flags: libc::c_int) -> io::Result<()> {
        let mut req = ifreq::new(&*self.name);
        // req.ifr_ifru.ifru_flags = (self.init_flags | flags) as _;

        unsafe { siocsifflags(self.socket.as_raw_fd(), &req) }?;

        Ok(())
    }
}
