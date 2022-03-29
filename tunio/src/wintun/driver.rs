use crate::wintun::logger::wintun_logger;
use std::sync::Arc;
use wintun_sys;

pub struct WinTunDriver {
    pub wintun: Arc<wintun_sys::wintun>,
}

impl WinTunDriver {
    pub fn new() -> Self {
        let wintun = Arc::new(unsafe { wintun_sys::wintun::new("wintun").unwrap() });
        unsafe {
            wintun.WintunSetLogger(Some(wintun_logger));
        }

        Self {
            wintun: Arc::new(unsafe { wintun_sys::wintun::new("wintun").unwrap() }),
        }
    }
}
