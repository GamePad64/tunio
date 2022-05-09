//! # [Universal TUN/TAP device driver](https://www.kernel.org/doc/Documentation/networking/tuntap.txt) support for tunio.
//!
//! This module provides support for TUN/TAP driver, used in Linux.
//!
//! Supported features:
//! - TUN/TAP modes
//! - Sync and async mode
//!
//! Low-level documentation for this driver can be found [here](https://www.kernel.org/doc/Documentation/networking/tuntap.txt).

pub mod interface;
pub mod queue;

use crate::config::IfConfig;
use crate::traits::{DriverT, PlatformIfConfigT};
use crate::Error;
use derive_builder::Builder;

pub use interface::Interface;
pub use queue::Queue;

pub struct Driver {}

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
        Interface::new(config)
    }
}
