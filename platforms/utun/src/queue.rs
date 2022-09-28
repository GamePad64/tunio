use crate::Error;
use libc::{PF_SYSTEM, SYSPROTO_CONTROL};
use nix::sys::socket::SysControlAddr;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd};

const UTUN_CONTROL_NAME: &str = "com.apple.net.utun_control";

pub(crate) fn create_device(name: &str, blocking: bool) -> Result<OwnedFd, Error> {
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
    if !blocking {
        tun_device.set_nonblocking(true)?;
    }
    tun_device.connect(&sa).unwrap();

    Ok(unsafe { OwnedFd::from_raw_fd(tun_device.into_raw_fd()) })
}
