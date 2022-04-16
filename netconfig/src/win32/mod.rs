use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIpInterfaceTable};
use windows::Win32::Networking::WinSock::AF_UNSPEC;

use crate::win32::handle::InterfaceHandleExt;
pub use handle::InterfaceHandle;
pub use metadata::{Metadata, MetadataExt};

mod handle;
mod metadata;
mod win_convert;

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
