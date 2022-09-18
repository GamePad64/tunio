use derive_builder::Builder;
use tunio_core::traits::{DriverT, PlatformIfConfigT};
use tunio_core::Error;

mod interface;
mod queue;

pub use interface::Interface;
#[cfg(feature = "tokio")]
pub use interface::TokioInterface;

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
