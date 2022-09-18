use super::logger::wintun_logger;
use super::PlatformIfConfig;
use std::sync::Arc;
use tunio_core::traits::DriverT;
use tunio_core::Error;

pub struct Driver {
    pub wintun: Arc<wintun_sys::wintun>,
}

impl DriverT for Driver {
    type PlatformIfConfig = PlatformIfConfig;

    fn new() -> Result<Self, Error> {
        let library_name = "wintun".to_string();
        let wintun = Arc::new(
            unsafe { wintun_sys::wintun::new(library_name) }.map_err(|e| {
                Error::LibraryNotLoaded {
                    reason: format!("{e:?}"),
                }
            })?,
        );

        unsafe {
            wintun.WintunSetLogger(Some(wintun_logger));
        }

        Ok(Self { wintun })
    }
}

impl Driver {
    pub(crate) fn wintun(&self) -> &Arc<wintun_sys::wintun> {
        &self.wintun
    }
}
