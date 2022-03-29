use crate::driver::WinTunDriver;
use crate::error::Error;
use crate::handle::UnsafeHandle;
use crate::stream::WinTunStream;
use get_last_error::Win32Error;
use log::error;
use std::ptr;
use std::sync::Arc;
use uuid::Uuid;
use widestring::U16CString;
use wintun_sys;
use wintun_sys::{GUID, WINTUN_ADAPTER_HANDLE, WINTUN_MIN_RING_CAPACITY};

pub struct WinTunInterface {
    driver: Arc<WinTunDriver>,
    handle: UnsafeHandle<WINTUN_ADAPTER_HANDLE>,
}

const MAX_NAME: usize = 255;

fn uuid_to_guid(uuid: &Uuid) -> GUID {
    let fields = uuid.as_fields();
    GUID {
        Data1: fields.0,
        Data2: fields.1,
        Data3: fields.2,
        Data4: *fields.3,
    }
}

fn encode_name(string: &str) -> Result<U16CString, Error> {
    let result = U16CString::from_str(string)?;
    match result.len() {
        0..=MAX_NAME => Ok(result),
        l => Err(Error::InterfaceNameTooLong(l)),
    }
}

impl WinTunInterface {
    pub fn new(driver: Arc<WinTunDriver>, name: &str, r#type: &str) -> Result<Self, Error> {
        let [name_u16, type_u16] = [name, r#type].map(encode_name);
        let (name_u16, type_u16) = (name_u16?, type_u16?);

        let guid = uuid_to_guid(&Uuid::new_v4());

        let handle = unsafe {
            driver.wintun.WintunCreateAdapter(
                name_u16.as_ptr(),
                type_u16.as_ptr(),
                &guid as *const GUID,
            )
        };

        if handle.is_null() {
            let err = Win32Error::get_last_error();
            error!("Failed to create adapter: {err}");
            return Err(Error::from(err));
        }

        Ok(Self {
            driver,
            handle: UnsafeHandle(handle),
        })
    }

    pub fn create_stream(self: &Arc<Self>) -> Result<WinTunStream, Error> {
        let capacity = WINTUN_MIN_RING_CAPACITY * 16;
        // let range = MIN_RING_CAPACITY..=MAX_RING_CAPACITY;
        // if !range.contains(&capacity) {}

        let handle = unsafe {
            self.driver
                .wintun
                .WintunStartSession(self.handle.0, capacity)
        };

        if handle.is_null() {
            let err = Win32Error::get_last_error();
            error!("Failed to create session: {err}");
            return Err(Error::from(err));
        }

        Ok(WinTunStream::new(
            UnsafeHandle(handle),
            self.driver.clone(),
            self.clone(),
        ))
    }
}

impl Drop for WinTunInterface {
    fn drop(&mut self) {
        unsafe { self.driver.wintun.WintunCloseAdapter(self.handle.0) };
        self.handle = UnsafeHandle(ptr::null_mut());
    }
}
