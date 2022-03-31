use crate::config::IfaceConfig;
use crate::platform::wintun::driver::WinTunDriver;
use crate::platform::wintun::handle::HandleWrapper;
use crate::platform::wintun::queue::WinTunStream;
use crate::traits::InterfaceT;
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
    wintun: Arc<wintun_sys::wintun>,
    handle: HandleWrapper<WINTUN_ADAPTER_HANDLE>,
    stream: Option<WinTunStream>,
}

impl InterfaceT for WinTunInterface {
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

        self.stream = Some(WinTunStream::new(
            HandleWrapper(handle),
            self.wintun.clone(),
        ));

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

impl WinTunInterface {
    pub(crate) fn new(
        wintun: Arc<wintun_sys::wintun>,
        params: IfaceConfig<WinTunDriver>,
    ) -> Result<Self, crate::Error> {
        let [name_u16, description_u16] =
            [&*params.name, &*params.platform.description].map(encode_name);
        let (name_u16, description_u16) = (name_u16?, description_u16?);

        let guid = uuid_to_guid(&Uuid::new_v4());

        let adapter_handle = unsafe {
            wintun.WintunCreateAdapter(
                name_u16.as_ptr(),
                description_u16.as_ptr(),
                &guid as *const GUID,
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
}

impl Drop for WinTunInterface {
    fn drop(&mut self) {
        unsafe { self.wintun.WintunCloseAdapter(self.handle.0) };
        self.handle = HandleWrapper(ptr::null_mut());
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
