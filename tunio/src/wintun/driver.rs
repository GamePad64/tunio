use crate::traits::Driver;
use crate::wintun::logger::wintun_logger;
use crate::wintun::params::WinTunDriverParams;
use crate::Error;
use std::sync::Arc;
use wintun_sys;

pub struct WinTunDriver {
    pub wintun: Arc<wintun_sys::wintun>,
}

impl Driver for WinTunDriver {
    type DriverParamsT = WinTunDriverParams;

    fn new(params: Self::DriverParamsT) -> Result<Arc<Self>, Error> {
        let library_name = params.library_name.unwrap_or_else(|| "wintun".to_string());
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

        Ok(Arc::new(Self { wintun }))
    }
}
