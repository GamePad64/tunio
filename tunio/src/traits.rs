use crate::config::IfaceConfig;
use crate::Error;
use std::io::{Read, Write};

pub trait PlatformIfaceConfigT: Default {}

pub trait DriverT: Sized {
    type PlatformInterface: InterfaceT;
    type PlatformInterfaceConfig: PlatformIfaceConfigT;

    fn new() -> Result<Self, crate::Error>
    where
        Self: Sized;

    fn new_interface(
        &mut self,
        config: IfaceConfig<Self>,
    ) -> Result<Self::PlatformInterface, Error>;

    fn new_interface_up(
        &mut self,
        config: IfaceConfig<Self>,
    ) -> Result<Self::PlatformInterface, Error> {
        let mut interface = self.new_interface(config)?;
        interface.up()?;
        Ok(interface)
    }
}

pub trait InterfaceT: Sized {
    fn up(&mut self) -> Result<(), Error>;
    fn down(&mut self) -> Result<(), Error>;
}

pub trait QueueT: Read + Write {}
