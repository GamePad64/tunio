use crate::config::{IfConfig, IfConfigBuilder};
use crate::Error;
use std::io::{Read, Write};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait PlatformIfConfigT: Default + Clone {
    type Builder: Default;
}

pub trait DriverT: Sized {
    type PlatformIf: InterfaceT;
    type PlatformIfConfig: PlatformIfConfigT;

    fn new() -> Result<Self, Error>
    where
        Self: Sized;

    fn new_interface(
        &mut self,
        config: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self::PlatformIf, Error>;

    fn new_interface_up(
        &mut self,
        config: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self::PlatformIf, Error> {
        let mut interface = self.new_interface(config)?;
        interface.up()?;
        Ok(interface)
    }

    fn if_config_builder() -> IfConfigBuilder<Self::PlatformIfConfig> {
        IfConfigBuilder::default()
    }
}

pub trait InterfaceT: Sized + QueueT {
    fn up(&mut self) -> Result<(), Error>;
    fn down(&mut self) -> Result<(), Error>;
    fn handle(&self) -> netconfig::InterfaceHandle;
}

pub trait QueueT: Read + Write {}
pub trait AsyncQueueT: AsyncRead + AsyncWrite {}
