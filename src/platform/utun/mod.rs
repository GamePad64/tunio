use crate::traits::{DriverT, PlatformIfConfigT};
use crate::Error;
use derive_builder::Builder;

mod interface;
mod queue;

pub use interface::Interface;

pub struct Driver {}

impl DriverT for Driver {
    type PlatformIfConfig = PlatformIfConfig;

    fn new() -> Result<Self, Error> {
        Ok(Driver {})
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
