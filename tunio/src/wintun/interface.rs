use crate::traits::Interface;
use crate::wintun::driver::WinTunDriver;
use crate::wintun::handle::UnsafeHandle;
use crate::wintun::params::WinTunInterfaceParams;
use crate::wintun::stream::WinTunStream;
use crate::Error;
use log::error;
use std::sync::Arc;
use std::{io, ptr};
use uuid::Uuid;
use widestring::U16CString;
use wintun_sys;
use wintun_sys::{GUID, WINTUN_ADAPTER_HANDLE, WINTUN_MIN_RING_CAPACITY};

const MAX_NAME: usize = 255;

pub struct WinTunInterface {
    driver: Arc<WinTunDriver>,
    handle: UnsafeHandle<WINTUN_ADAPTER_HANDLE>,
}

impl Interface for WinTunInterface {
    type DriverT = WinTunDriver;
    type InterfaceParamsT = WinTunInterfaceParams;

    fn new(
        driver: Arc<Self::DriverT>,
        params: Self::InterfaceParamsT,
    ) -> Result<Arc<Self>, crate::Error> {
        let [name_u16, description_u16] = [&*params.name, &*params.description].map(encode_name);
        let (name_u16, description_u16) = (name_u16?, description_u16?);

        let guid = uuid_to_guid(&Uuid::new_v4());

        let handle = unsafe {
            driver.wintun.WintunCreateAdapter(
                name_u16.as_ptr(),
                description_u16.as_ptr(),
                &guid as *const GUID,
            )
        };

        if handle.is_null() {
            let err = io::Error::last_os_error();
            error!("Failed to create adapter: {err}");
            return Err(Error::from(err));
        }

        Ok(Arc::new(Self {
            driver,
            handle: UnsafeHandle(handle),
        }))
    }
}

impl WinTunInterface {
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
            let err = io::Error::last_os_error();
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
    let result = U16CString::from_str(string).map_err(|_| Error::InterfaceNameUnicodeError)?;
    match result.len() {
        0..=MAX_NAME => Ok(result),
        l => Err(Error::InterfaceNameTooLong(l)),
    }
}
