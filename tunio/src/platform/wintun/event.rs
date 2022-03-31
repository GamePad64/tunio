use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{CreateEventA, SetEvent};

pub(crate) struct SafeEvent(pub(crate) HANDLE);

impl Default for SafeEvent {
    fn default() -> Self {
        Self(unsafe { CreateEventA(std::ptr::null(), false, false, None) })
    }
}

impl SafeEvent {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_event(&self) {
        unsafe {
            SetEvent(self.0);
        }
    }
}

impl Drop for SafeEvent {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
