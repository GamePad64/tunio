pub mod interface;
pub mod queue;

use crate::config::IfConfig;
use crate::traits::{DriverT, PlatformIfConfigT};
use crate::Error;
use derive_builder::Builder;

pub use interface::Interface;
pub use queue::Queue;

pub struct Driver;

#[derive(Builder, Default, Clone)]
pub struct PlatformIfConfig {}

impl PlatformIfConfigT for PlatformIfConfig {
    type Builder = PlatformIfConfigBuilder;
}

impl DriverT for Driver {
    type PlatformIf = Interface;
    type PlatformIfConfig = PlatformIfConfig;

    fn new() -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(Driver {})
    }

    fn new_interface(
        &mut self,
        config: IfConfig<PlatformIfConfig>,
    ) -> Result<Self::PlatformIf, Error> {
        Interface::new(config)
    }
}
