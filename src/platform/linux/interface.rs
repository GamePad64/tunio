use super::queue::{create_device, Device};
use super::Driver;
use super::PlatformIfConfig;
use crate::config::IfConfig;
use crate::platform::util::{sync::Queue, QueueFdT};
use crate::traits::{InterfaceT, SyncQueueT};
use crate::Error;
use delegate::delegate;
use log::debug;
use netconfig::sys::InterfaceHandleExt;
use std::io;
use std::io::{Read, Write};

pub struct LinuxInterface<Q> {
    name: String,
    pub(crate) queue: Q,
}

impl<Q> LinuxInterface<Q> {
    pub fn name(&self) -> &str {
        &*self.name
    }
}

impl<Q: QueueFdT> InterfaceT for LinuxInterface<Q> {
    type PlatformDriver = Driver;
    type PlatformIfConfig = PlatformIfConfig;

    fn new(
        _driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error> {
        let Device { device, name } = create_device(&*params.name, params.layer)?;
        let queue = Q::new(device.into());

        if &*params.name != name {
            debug!(
                "Interface name is changed \"{}\" -> \"{}\"",
                &*params.name, name
            );
        }

        Ok(Self { name, queue })
    }

    fn up(&mut self) -> Result<(), Error> {
        Ok(self.handle().set_up(true)?)
    }

    fn down(&mut self) -> Result<(), Error> {
        Ok(self.handle().set_up(false)?)
    }

    fn handle(&self) -> netconfig::InterfaceHandle {
        netconfig::InterfaceHandle::try_from_name(self.name()).unwrap()
    }
}

pub type Interface = LinuxInterface<Queue>;

impl SyncQueueT for Interface {}

impl Read for Interface {
    delegate! {
        to self.queue {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error>;
        }
    }
}

impl Write for Interface {
    delegate! {
        to self.queue {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }
}
