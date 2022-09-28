use super::HandleWrapper;
use log::error;
use std::io;
use std::sync::Arc;
use tunio_core::Error;
use widestring::U16CString;
use windows::core::{GUID, PCWSTR};
use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;
use wintun_sys::WINTUN_ADAPTER_HANDLE;

const MAX_NAME: usize = 255;

pub struct Adapter {
    wintun: Arc<wintun_sys::wintun>,
    handle: HandleWrapper<WINTUN_ADAPTER_HANDLE>,
}

impl Adapter {
    pub fn new(
        guid: GUID,
        name: &str,
        description: &str,
        wintun: Arc<wintun_sys::wintun>,
    ) -> Result<Self, Error> {
        let [name_u16, description_u16] = [name, description].map(encode_name);
        let (name_u16, description_u16) = (name_u16?, description_u16?);

        let adapter_handle = unsafe {
            wintun.WintunCreateAdapter(
                PCWSTR::from_raw(name_u16.as_ptr()),
                PCWSTR::from_raw(description_u16.as_ptr()),
                &guid,
            )
        };

        if adapter_handle.is_null() {
            let err = io::Error::last_os_error();
            error!("Failed to create adapter: {err}");
            return Err(Error::from(err));
        }

        Ok(Self {
            wintun,
            handle: HandleWrapper(adapter_handle),
        })
    }

    pub fn luid(&self) -> u64 {
        let mut luid_buf = NET_LUID_LH::default();
        unsafe {
            self.wintun
                .WintunGetAdapterLUID(self.handle.0, &mut luid_buf as _);
            luid_buf.Value
        }
    }

    pub fn handle(&self) -> WINTUN_ADAPTER_HANDLE {
        self.handle.0
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        unsafe { self.wintun.WintunCloseAdapter(self.handle.0) };
    }
}

fn encode_name(string: &str) -> Result<U16CString, Error> {
    let result = U16CString::from_str(string).map_err(|_| Error::InterfaceNameUnicodeError)?;
    match result.len() {
        0..=MAX_NAME => Ok(result),
        l => Err(Error::InterfaceNameTooLong(l, MAX_NAME)),
    }
}
