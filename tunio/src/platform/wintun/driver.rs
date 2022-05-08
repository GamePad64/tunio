use crate::config::IfConfig;
use crate::error::Error;
use crate::platform::wintun::config::PlatformIfConfig;
use crate::platform::wintun::interface::Interface;
use crate::platform::wintun::logger::wintun_logger;
use crate::traits::DriverT;
use std::sync::Arc;
use wintun_sys;

pub struct Driver {
    pub wintun: Arc<wintun_sys::wintun>,
}

impl DriverT for Driver {
    type PlatformIf = Interface;
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

    fn new_interface(
        &mut self,
        config: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self::PlatformIf, Error> {
        Interface::new(self.wintun.clone(), config)
    }
}
