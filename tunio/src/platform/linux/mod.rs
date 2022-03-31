mod ifreq;
pub mod interface;
pub mod params;

use crate::linux::interface::LinuxInterface;
use crate::linux::params::{LinuxDriverParams, LinuxInterfaceParams};
use crate::traits::DriverT;
use crate::{Error, InterfaceT};
use std::ffi::CString;
use std::sync::Arc;

pub struct LinuxDriver;

impl DriverT for LinuxDriver {
    type DriverParamsT = LinuxDriverParams;

    fn new(_params: Self::DriverParamsT) -> Result<Arc<Self>, Error> {
        Ok(Arc::new(Self {}))
    }
}

impl LinuxDriver {}
