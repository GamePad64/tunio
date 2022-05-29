use crate::config::IfConfig;
use crate::traits::{DriverT, InterfaceT, PlatformIfConfigT};
use crate::Error;
use derive_builder::Builder;

mod interface;
mod queue;

pub use interface::Interface;
pub use queue::Queue;

pub struct Driver {}

impl DriverT for Driver {
    type PlatformIf = Interface;
    type PlatformIfConfig = PlatformIfConfig;

    fn new() -> Result<Self, Error> {
        Ok(Driver {})
    }

    fn new_interface(
        &mut self,
        config: IfConfig<PlatformIfConfig>,
    ) -> Result<Self::PlatformIf, Error> {
        let mut iface = self.new_interface_up(config)?;
        iface.down()?;
        Ok(iface)
    }

    fn new_interface_up(
        &mut self,
        config: IfConfig<PlatformIfConfig>,
    ) -> Result<Self::PlatformIf, Error> {
        Interface::new(config)
    }
}

#[derive(Builder, Clone)]
pub struct PlatformIfConfig {}

impl PlatformIfConfigT for PlatformIfConfig {
    type Builder = PlatformIfConfigBuilder;
}

impl Default for PlatformIfConfig {
    fn default() -> Self {
        PlatformIfConfigBuilder::default().build().unwrap()
    }
}
