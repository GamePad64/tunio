pub mod interface;
mod queue;

use crate::config::IfaceConfig;
use crate::traits::{DriverT, PlatformIfaceConfigT};
use crate::Error;

pub use interface::LinuxInterface;

pub struct Driver;

#[derive(Default, Clone)]
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
