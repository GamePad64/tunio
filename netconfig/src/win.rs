use crate::error::Error;
use core::default::Default;
use core::result::Result;
use core::result::Result::{Err, Ok};
use getset::{CopyGetters, Getters};
use ipnet::IpNet;
use log::warn;
use std::collections::HashSet;
use std::net::SocketAddr;
use widestring::ucstr::U16CStr;
use widestring::ucstring::U16CString;
use windows::core::GUID;
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_NOT_FOUND};
use windows::Win32::NetworkManagement::IpHelper::{
    ConvertInterfaceGuidToLuid, ConvertInterfaceLuidToNameW, CreateUnicastIpAddressEntry,
    DeleteUnicastIpAddressEntry, FreeMibTable, GetIfEntry2, GetIpInterfaceEntry,
    GetIpInterfaceTable, GetUnicastIpAddressTable, InitializeUnicastIpAddressEntry,
    SetIpInterfaceEntry, AF_INET, AF_INET6, AF_UNSPEC, MIB_IF_ROW2, MIB_IPINTERFACE_ROW,
    MIB_UNICASTIPADDRESS_ROW, NET_LUID_LH,
};
use windows::Win32::NetworkManagement::Ndis::NDIS_IF_MAX_STRING_SIZE;

pub trait MetadataExt {
    fn luid(&self) -> NET_LUID_LH;
    fn guid(&self) -> GUID;
    fn index(&self) -> u32;
    fn alias(&self) -> String;
    fn description(&self) -> String;
}

impl MetadataExt for crate::Metadata {
    fn luid(&self) -> NET_LUID_LH {
        self.0.luid()
    }

    fn guid(&self) -> GUID {
        self.0.guid()
    }

    fn index(&self) -> u32 {
        self.0.index()
    }

    fn alias(&self) -> String {
        self.0.alias().clone()
    }

    fn description(&self) -> String {
        self.0.description().clone()
    }
}

pub(crate) struct InterfaceHandle {
    luid: NET_LUID_LH,
}

mod win_convert {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
    use windows::Win32::Foundation::CHAR;
    use windows::Win32::NetworkManagement::IpHelper::{ADDRESS_FAMILY, AF_INET, AF_INET6};
    use windows::Win32::Networking::WinSock::{
        IN6_ADDR, IN6_ADDR_0, IN_ADDR, IN_ADDR_0, SOCKADDR_IN, SOCKADDR_IN6, SOCKADDR_IN6_0,
        SOCKADDR_INET,
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
}

#[derive(Getters, CopyGetters, Default)]
pub(crate) struct Metadata {
    #[getset(get_copy = "pub")]
    luid: NET_LUID_LH,
    #[getset(get_copy = "pub")]
    guid: GUID,
    #[getset(get_copy = "pub")]
    index: u32,
    #[getset(get_copy = "pub")]
    mtu: u32,
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    alias: String,
    #[getset(get = "pub")]
    description: String,
}

impl Metadata {
    pub(crate) fn handle(&self) -> crate::InterfaceHandle {
        crate::InterfaceHandle(InterfaceHandle::from_luid(self.luid))
    }
}

impl InterfaceHandle {
    fn from_luid(luid: NET_LUID_LH) -> Self {
        Self { luid }
    }

    fn from_guid(guid: GUID) -> Self {
        let mut luid = NET_LUID_LH::default();
        unsafe {
            ConvertInterfaceGuidToLuid(&guid, &mut luid).unwrap();
        }

        Self::from_luid(luid)
    }

    pub(crate) fn metadata(&self) -> Result<crate::Metadata, Error> {
        let mut result = Metadata {
            luid: self.luid,
            ..Default::default()
        };

        // MIB_IF_ROW2 data
        {
            let mut row = MIB_IF_ROW2 {
                InterfaceLuid: self.luid,
                ..Default::default()
            };
            unsafe {
                GetIfEntry2(&mut row);
            }
            result.description = U16CStr::from_slice_truncate(&row.Description)
                .map_err(|_| Error::UnexpectedMetadata)?
                .to_string()
                .map_err(|_| Error::UnexpectedMetadata)?;
            result.alias = U16CStr::from_slice_truncate(&row.Alias)
                .map_err(|_| Error::UnexpectedMetadata)?
                .to_string()
                .map_err(|_| Error::UnexpectedMetadata)?;
            result.guid = row.InterfaceGuid;
            result.index = row.InterfaceIndex;
            result.mtu = row.Mtu;
        }

        // Interface name
        {
            let mut name_buf = vec![0u16; (NDIS_IF_MAX_STRING_SIZE + 1) as _];
            unsafe { ConvertInterfaceLuidToNameW(&self.luid, &mut name_buf) }
                .map_err(|_| Error::UnexpectedMetadata)?;

            result.name = U16CString::from_vec_truncate(name_buf)
                .to_string()
                .map_err(|_| Error::UnexpectedMetadata)?;
        }

        Ok(crate::Metadata(result))
    }

    pub(crate) fn add_ip(&self, network: IpNet) {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceLuid = self.luid;
        row.Address = win_convert::xSocketAddr(SocketAddr::new(network.addr(), 0)).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe {
            CreateUnicastIpAddressEntry(&row).unwrap();
        }
    }

    pub(crate) fn remove_ip(&self, network: IpNet) {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceLuid = self.luid;
        row.Address = win_convert::xSocketAddr(SocketAddr::new(network.addr(), 0)).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe {
            DeleteUnicastIpAddressEntry(&row).unwrap();
        }
    }

    pub(crate) fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
        let mut table = std::ptr::null_mut();

        unsafe { GetUnicastIpAddressTable(AF_UNSPEC.0 as _, &mut table) }
            .map_err(|_| Error::InternalError)?;
        let table = scopeguard::guard(table, |table| {
            if !table.is_null() {
                unsafe {
                    FreeMibTable(table as _);
                }
            }
        });

        let mut addresses_set = HashSet::new();

        unsafe {
            for i in 0..(*(*table)).NumEntries as _ {
                let row = &(*(*table)).Table.get_unchecked(i);
                let sockaddr = win_convert::xSocketAddr::from(row.Address);

                if row.InterfaceLuid != self.luid {
                    continue;
                }

                addresses_set.insert(
                    IpNet::new(sockaddr.0.ip(), row.OnLinkPrefixLength)
                        .map_err(|_| Error::UnexpectedMetadata)?,
                );
            }
        }

        Ok(addresses_set.iter().cloned().collect())
    }

    pub(crate) fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
        for family in [AF_INET, AF_INET6] {
            let mut row = MIB_IPINTERFACE_ROW {
                Family: family.0 as _,
                InterfaceLuid: self.luid,
                ..Default::default()
            };

            match unsafe { GetIpInterfaceEntry(&mut row).map_err(|e| e.win32_error().unwrap()) } {
                Ok(_) => Ok(()),
                Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
                Err(ERROR_NOT_FOUND) => {
                    warn!("Interface not found with family: {:?}", family);
                    continue;
                }
                _ => Err(Error::InternalError),
            }?;

            row.NlMtu = mtu;

            match unsafe { SetIpInterfaceEntry(&mut row).map_err(|e| e.win32_error().unwrap()) } {
                Ok(_) => Ok(()),
                Err(ERROR_FILE_NOT_FOUND) => Err(Error::InterfaceNotFound),
                Err(ERROR_NOT_FOUND) => {
                    warn!("Interface not found with family: {:?}", family);
                    continue;
                }
                Err(ERROR_ACCESS_DENIED) => Err(Error::AccessDenied),
                Err(_) => Err(Error::InternalError),
            }?;
        }
        Ok(())
    }
}

pub(crate) fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    let mut table = std::ptr::null_mut();

    let result = unsafe { GetIpInterfaceTable(AF_UNSPEC.0 as _, &mut table) };
    let table = scopeguard::guard(table, |table| {
        if !table.is_null() {
            unsafe {
                FreeMibTable(table as _);
            }
        }
    });

    unsafe {
        if result.is_ok() {
            let mut result = Vec::with_capacity((*(*table)).NumEntries as _);
            for i in 0..(*(*table)).NumEntries as _ {
                let row = &(*(*table)).Table.get_unchecked(i);
                result.push(crate::InterfaceHandle(InterfaceHandle::from_luid(
                    row.InterfaceLuid,
                )));
            }
            result
        } else {
            vec![]
        }
    }
}
