use crate::config::IfaceConfig;
use crate::platform::wintun::driver::Driver;
use crate::platform::wintun::handle::HandleWrapper;
use crate::platform::wintun::queue::Queue;
use crate::traits::InterfaceT;
use crate::Error;
use log::error;
use std::sync::Arc;
use std::{io, ptr};
use widestring::U16CString;
use windows::core::GUID;
use windows::Win32::NetworkManagement::IpHelper::{ConvertInterfaceLuidToGuid, NET_LUID_LH};
use wintun_sys;
use wintun_sys::{WINTUN_ADAPTER_HANDLE, WINTUN_MIN_RING_CAPACITY};

const MAX_NAME: usize = 255;

pub struct Interface {
    wintun: Arc<wintun_sys::wintun>,
    handle: HandleWrapper<WINTUN_ADAPTER_HANDLE>,
    stream: Option<Queue>,
}

impl InterfaceT for Interface {
    fn up(&mut self) -> Result<(), Error> {
        let capacity = WINTUN_MIN_RING_CAPACITY * 16;
        // let range = MIN_RING_CAPACITY..=MAX_RING_CAPACITY;
        // if !range.contains(&capacity) {}

        let handle = unsafe { self.wintun.WintunStartSession(self.handle.0, capacity) };

        if handle.is_null() {
            let err = io::Error::last_os_error();
            error!("Failed to create session: {err}");
            return Err(Error::from(err));
        }

        self.stream = Some(Queue::new(HandleWrapper(handle), self.wintun.clone()));

        Ok(())
    }

    fn down(&mut self) -> Result<(), Error> {
        if self.stream.is_some() {
            let _ = self.stream.take();
            Ok(())
        } else {
            Err(Error::InterfaceStateInvalid)
        }
    }
}

impl Interface {
    pub(crate) fn new(
        wintun: Arc<wintun_sys::wintun>,
        params: IfaceConfig<Driver>,
    ) -> Result<Self, crate::Error> {
        let [name_u16, description_u16] =
            [&*params.name, &*params.platform.description].map(encode_name);
        let (name_u16, description_u16) = (name_u16?, description_u16?);

        let guid = GUID::new().unwrap();
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
            stream: None,
        })
    }

    pub fn get_luid(&self) -> NET_LUID_LH {
        let mut luid_buf: wintun_sys::NET_LUID = unsafe { std::mem::zeroed() };
        unsafe {
            self.wintun
                .WintunGetAdapterLUID(self.handle.0, &mut luid_buf as _)
        }
        NET_LUID_LH {
            Value: unsafe { luid_buf.Value },
        }
    }

    pub fn get_guid(&self) -> GUID {
        let mut guid = GUID::zeroed();
        unsafe {
            ConvertInterfaceLuidToGuid(&self.get_luid(), &mut guid as _).unwrap();
        }
        guid
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        unsafe { self.wintun.WintunCloseAdapter(self.handle.0) };
        self.handle = HandleWrapper(ptr::null_mut());
    }
}

fn encode_name(string: &str) -> Result<U16CString, Error> {
    let result = U16CString::from_str(string).map_err(|_| Error::InterfaceNameUnicodeError)?;
    match result.len() {
        0..=MAX_NAME => Ok(result),
        l => Err(Error::InterfaceNameTooLong(l)),
    }
}
