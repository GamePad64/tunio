use super::HandleWrapper;
use crate::Error;
use log::error;
use std::io;
use std::sync::Arc;
use widestring::U16CString;
use windows::core::GUID;
use windows::Win32::NetworkManagement::IpHelper::NET_LUID_LH;
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

        let guid = wintun_sys::GUID {
            Data1: guid.data1,
            Data2: guid.data2,
            Data3: guid.data3,
            Data4: guid.data4,
        };

        let adapter_handle = unsafe {
            wintun.WintunCreateAdapter(
                name_u16.as_ptr(),
                description_u16.as_ptr(),
                &guid as *const wintun_sys::GUID,
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

    pub fn luid(&self) -> NET_LUID_LH {
        let mut luid_buf: wintun_sys::NET_LUID = unsafe { std::mem::zeroed() };
        unsafe {
            self.wintun
                .WintunGetAdapterLUID(self.handle.0, &mut luid_buf as _)
        }
        NET_LUID_LH {
            Value: unsafe { luid_buf.Value },
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

pub fn encode_name(string: &str) -> Result<U16CString, Error> {
    let result = U16CString::from_str(string).map_err(|_| Error::InterfaceNameUnicodeError)?;
    match result.len() {
        0..=MAX_NAME => Ok(result),
        l => Err(Error::InterfaceNameTooLong(l, MAX_NAME)),
    }
}
