use crate::win32::{win_convert, Metadata};
use crate::{Error, InterfaceHandleCommonT};
use ipnet::IpNet;
use log::warn;
use std::collections::HashSet;
use std::net::SocketAddr;
use widestring::{U16CStr, U16CString};
use windows::core::GUID;
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_NOT_FOUND};
use windows::Win32::NetworkManagement::IpHelper::{
    ConvertInterfaceGuidToLuid, ConvertInterfaceLuidToNameW, CreateUnicastIpAddressEntry,
    DeleteUnicastIpAddressEntry, FreeMibTable, GetIfEntry2, GetIpInterfaceEntry,
    GetUnicastIpAddressTable, InitializeUnicastIpAddressEntry, SetIpInterfaceEntry, MIB_IF_ROW2,
    MIB_IPINTERFACE_ROW, MIB_UNICASTIPADDRESS_ROW, NET_LUID_LH,
};
use windows::Win32::NetworkManagement::Ndis::NDIS_IF_MAX_STRING_SIZE;
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC};

pub struct InterfaceHandle {
    luid: NET_LUID_LH,
}

pub trait InterfaceHandleExt {
    fn from_luid(luid: NET_LUID_LH) -> Self;
    fn from_guid(guid: GUID) -> Self;
}

impl InterfaceHandleExt for InterfaceHandle {
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
}

impl InterfaceHandleExt for crate::InterfaceHandle {
    fn from_luid(luid: NET_LUID_LH) -> Self {
        Self(InterfaceHandle::from_luid(luid))
    }

    fn from_guid(guid: GUID) -> Self {
        Self(InterfaceHandle::from_guid(guid))
    }
}

impl InterfaceHandleCommonT for InterfaceHandle {
    fn metadata(&self) -> Result<crate::Metadata, Error> {
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
                GetIfEntry2(&mut row).map_err(|_| Error::InterfaceNotFound)?;
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

    fn add_ip(&self, network: IpNet) {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceLuid = self.luid;
        row.Address = win_convert::xSocketAddr(SocketAddr::new(network.addr(), 0)).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe {
            CreateUnicastIpAddressEntry(&row).unwrap();
        }
    }

    fn remove_ip(&self, network: IpNet) {
        let mut row = MIB_UNICASTIPADDRESS_ROW::default();
        unsafe { InitializeUnicastIpAddressEntry(&mut row as _) };

        row.InterfaceLuid = self.luid;
        row.Address = win_convert::xSocketAddr(SocketAddr::new(network.addr(), 0)).into();
        row.OnLinkPrefixLength = network.prefix_len();

        unsafe {
            DeleteUnicastIpAddressEntry(&row).unwrap();
        }
    }

    fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
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

    fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
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
