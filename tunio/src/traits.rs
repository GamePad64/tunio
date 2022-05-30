use crate::Error;
use crate::{IfConfig, IfConfigBuilder};
use std::io::{Read, Write};

pub trait PlatformIfConfigT: Default + Clone {
    type Builder: Default;
}

pub trait DriverT: Sized {
    type PlatformIfConfig: PlatformIfConfigT;

    fn new() -> Result<Self, Error>
    where
        Self: Sized;

    fn if_config_builder() -> IfConfigBuilder<Self::PlatformIfConfig> {
        IfConfigBuilder::default()
    }
}

pub trait InterfaceT: Sized {
    type PlatformDriver: DriverT;
    type PlatformIfConfig: PlatformIfConfigT;

    fn new(
        driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error>;
    fn new_up(
        driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error> {
        let mut interface = Self::new(driver, params)?;
        interface.up()?;
        Ok(interface)
    }

    fn up(&mut self) -> Result<(), Error>;
    fn down(&mut self) -> Result<(), Error>;
    fn handle(&self) -> netconfig::InterfaceHandle;
}

pub trait QueueT: Read + Write {}
cfg_if::cfg_if! {
    if #[cfg(feature = "async-tokio")] {
        use tokio::io::{AsyncRead, AsyncWrite};
        pub trait AsyncTokioQueueT: AsyncRead + AsyncWrite {}
    }
}
