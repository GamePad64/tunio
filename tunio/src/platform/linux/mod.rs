mod ifreq;
pub mod interface;
mod queue;

use crate::config::IfaceConfig;
use crate::traits::{DriverT, PlatformIfaceConfigT};
use crate::Error;
use std::ffi::CString;
use std::sync::Arc;

pub use interface::LinuxInterface;

pub struct Driver;

#[derive(Default)]
pub struct PlatformInterfaceConfig {}

impl PlatformIfaceConfigT for PlatformInterfaceConfig {}

impl DriverT for Driver {
    type PlatformInterface = LinuxInterface;
    type PlatformInterfaceConfig = PlatformInterfaceConfig;

    fn new() -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(Driver {})
    }

    fn new_interface(
        &mut self,
        config: IfaceConfig<Self>,
    ) -> Result<Self::PlatformInterface, Error> {
        interface::LinuxInterface::new(config)
    }
}
