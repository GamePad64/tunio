//! # [Universal TUN/TAP device driver](https://www.kernel.org/doc/Documentation/networking/tuntap.txt) support for tunio.
//!
//! This module provides support for TUN/TAP driver, used in Linux.
//!
//! Supported features:
//! - TUN/TAP modes
//! - Sync and async mode
//!
//! Low-level documentation for this driver can be found [here](https://www.kernel.org/doc/Documentation/networking/tuntap.txt).

mod interface;
mod queue;

use derive_builder::Builder;
use tunio_core::traits::{DriverT, PlatformIfConfigT};
use tunio_core::Error;

#[cfg(feature = "tokio")]
pub use interface::TokioInterface;
pub use interface::{Interface, LinuxInterface};

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
    type PlatformIfConfig = PlatformIfConfig;

    fn new() -> Result<Self, Error> {
        Ok(Self {})
    }
}
