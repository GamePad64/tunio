use crate::traits::QueueT;
use crate::Error;
use delegate::delegate;
use libc::{PF_SYSTEM, SYSPROTO_CONTROL};
use nix::sys::socket::SysControlAddr;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io::{self, Read, Write};
use std::mem;
use std::os::unix::io::AsRawFd;
#[cfg(feature = "async-tokio")]
use tokio::io::unix::AsyncFd;

const UTUN_CONTROL_NAME: &str = "com.apple.net.utun_control";

impl QueueT for Queue {}

pub struct Queue {
    #[cfg(feature = "async-tokio")]
    tun_device: AsyncFd<Socket>,
    #[cfg(not(feature = "async-tokio"))]
    tun_device: Socket,
}

impl Queue {
    pub(crate) fn new(name: &str) -> Result<Self, Error> {
        let mut id = match name {
            s if s.starts_with("utun") => s[4..].parse().map_err(|_| Error::InterfaceNameInvalid),
            _ => Err(Error::InterfaceNameInvalid),
        }?;
        id += 1;

        let tun_device = Socket::new(
            Domain::from(PF_SYSTEM),
            Type::DGRAM,
            Some(Protocol::from(SYSPROTO_CONTROL)),
        )
        .unwrap();

        let sa = SysControlAddr::from_name(tun_device.as_raw_fd(), UTUN_CONTROL_NAME, id).unwrap();

        let (_, sa) = unsafe {
            SockAddr::init(|sa_storage, len| {
                let sockaddr = sa_storage as *mut libc::sockaddr_ctl;
                *sockaddr = *sa.as_ref();
                *len = mem::size_of::<libc::sockaddr_ctl>() as _;
                Ok(())
            })
        }
        .unwrap();
        tun_device.connect(&sa).unwrap();

        Ok(Queue {
            #[cfg(feature = "async-tokio")]
            tun_device: AsyncFd::new(tun_device)?,
            #[cfg(not(feature = "async-tokio"))]
            tun_device,
        })
    }

    pub(crate) fn tun_file_ref(&self) -> &Socket {
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
