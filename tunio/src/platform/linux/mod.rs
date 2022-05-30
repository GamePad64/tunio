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
#[cfg(feature = "async-tokio")]
mod tokio_interface;

use crate::traits::{DriverT, PlatformIfConfigT};
use crate::Error;
use derive_builder::Builder;

pub use interface::{Interface, LinuxInterface};
#[cfg(feature = "async-tokio")]
pub use tokio_interface::AsyncTokioInterface;

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
