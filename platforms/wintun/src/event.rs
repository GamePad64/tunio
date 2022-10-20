#![allow(dead_code)]
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{CreateEventA, SetEvent};

pub(crate) struct SafeEvent(HANDLE);

impl SafeEvent {
    pub fn new(manual_reset: bool, initial_state: bool) -> Self {
        Self(unsafe { CreateEventA(None, manual_reset, initial_state, None).unwrap() })
    }

    pub fn set_event(&self) {
        unsafe {
            SetEvent(self.handle());
        }
    }

    pub fn handle(&self) -> HANDLE {
        self.0
    }
}

impl Drop for SafeEvent {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
