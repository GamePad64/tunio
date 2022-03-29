use crate::logger::wintun_logger;
use crate::wintun_raw;
use std::sync::Arc;

pub struct WinTunDriver {
    pub(crate) wintun: Arc<wintun_raw::wintun>,
}

impl WinTunDriver {
    pub fn new() -> Self {
        let wintun = Arc::new(unsafe { wintun_raw::wintun::new("wintun").unwrap() });
        unsafe {
            wintun.WintunSetLogger(Some(wintun_logger));
        }

        Self {
            wintun: Arc::new(unsafe { wintun_raw::wintun::new("wintun").unwrap() }),
        }
    }
}
