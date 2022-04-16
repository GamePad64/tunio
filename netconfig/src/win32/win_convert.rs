#![allow(non_camel_case_types)]
#![allow(dead_code)]
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use windows::Win32::Foundation::CHAR;
use windows::Win32::Networking::WinSock::{
    ADDRESS_FAMILY, AF_INET, AF_INET6, IN6_ADDR, IN6_ADDR_0, IN_ADDR, IN_ADDR_0, SOCKADDR_IN,
    SOCKADDR_IN6, SOCKADDR_IN6_0, SOCKADDR_INET,
};

struct xIpv4Addr(pub(crate) Ipv4Addr);

impl Into<IN_ADDR> for xIpv4Addr {
    fn into(self) -> IN_ADDR {
        IN_ADDR {
            S_un: IN_ADDR_0 {
                S_addr: u32::from(self.0).to_be(),
            },
        }
    }
}

impl From<IN_ADDR> for xIpv4Addr {
    fn from(ip: IN_ADDR) -> Self {
        Self(Ipv4Addr::from(u32::from_be(unsafe { ip.S_un.S_addr })))
    }
}

struct xIpv6Addr(pub(crate) Ipv6Addr);

impl Into<IN6_ADDR> for xIpv6Addr {
    fn into(self) -> IN6_ADDR {
        IN6_ADDR {
            u: IN6_ADDR_0 {
                Byte: self.0.octets(),
            },
        }
    }
}

impl From<IN6_ADDR> for xIpv6Addr {
    fn from(ip: IN6_ADDR) -> Self {
        Self(Ipv6Addr::from(unsafe { ip.u.Byte }))
    }
}

struct xSocketAddrV4(pub(crate) SocketAddrV4);

impl Into<SOCKADDR_IN> for xSocketAddrV4 {
    fn into(self) -> SOCKADDR_IN {
        SOCKADDR_IN {
            sin_family: AF_INET.0 as _,
            sin_port: self.0.port(),
            sin_addr: xIpv4Addr(*self.0.ip()).into(),
            sin_zero: [CHAR(0u8); 8],
        }
    }
}

impl From<SOCKADDR_IN> for xSocketAddrV4 {
    fn from(sa: SOCKADDR_IN) -> Self {
        xSocketAddrV4(SocketAddrV4::new(
            xIpv4Addr::from(sa.sin_addr).0,
            sa.sin_port,
        ))
    }
}

struct xSocketAddrV6(pub(crate) SocketAddrV6);

impl Into<SOCKADDR_IN6> for xSocketAddrV6 {
    fn into(self) -> SOCKADDR_IN6 {
        SOCKADDR_IN6 {
            sin6_family: AF_INET6.0 as _,
            sin6_port: self.0.port(),
            sin6_flowinfo: self.0.flowinfo(),
            sin6_addr: xIpv6Addr(*self.0.ip()).into(),
            Anonymous: SOCKADDR_IN6_0 {
                sin6_scope_id: self.0.scope_id(),
            },
        }
    }
}

impl From<SOCKADDR_IN6> for xSocketAddrV6 {
    fn from(sa: SOCKADDR_IN6) -> Self {
        xSocketAddrV6(SocketAddrV6::new(
            xIpv6Addr::from(sa.sin6_addr).0,
            sa.sin6_port,
            sa.sin6_flowinfo,
            unsafe { sa.Anonymous.sin6_scope_id },
        ))
    }
}

pub struct xSocketAddr(pub(crate) SocketAddr);

impl Into<SOCKADDR_INET> for xSocketAddr {
    fn into(self) -> SOCKADDR_INET {
        match self.0 {
            SocketAddr::V4(addr) => SOCKADDR_INET {
                Ipv4: xSocketAddrV4(addr).into(),
            },
            SocketAddr::V6(addr) => SOCKADDR_INET {
                Ipv6: xSocketAddrV6(addr).into(),
            },
        }
    }
}

impl From<SOCKADDR_INET> for xSocketAddr {
    fn from(sa: SOCKADDR_INET) -> Self {
        xSocketAddr(unsafe {
            match ADDRESS_FAMILY(sa.si_family as _) {
                AF_INET => SocketAddr::from(xSocketAddrV4::from(sa.Ipv4).0),
                AF_INET6 => SocketAddr::from(xSocketAddrV6::from(sa.Ipv6).0),
                _ => panic!("Invalid address family"),
            }
        })
    }
}
