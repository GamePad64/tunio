use crate::platform::util::sync::Queue;
use crate::platform::util::QueueFdT;
use crate::platform::utun::queue::create_device;
use crate::platform::utun::{Driver, PlatformIfConfig};
use crate::traits::{InterfaceT, SyncQueueT};
use crate::{Error, IfConfig};
use delegate::delegate;
use netconfig::sys::InterfaceHandleExt;
use std::io::{self, Read, Write};

pub struct UtunInterface<Q> {
    name: String,
    queue: Q,
}

impl<Q: QueueFdT> InterfaceT for UtunInterface<Q> {
    type PlatformDriver = Driver;
    type PlatformIfConfig = PlatformIfConfig;

    fn new(
        _driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error> {
        let queue = Q::new(create_device(&params.name)?);

        Ok(Self {
            name: params.name,
            queue,
        })
    }

    fn up(&mut self) -> Result<(), Error> {
        Ok(self.handle().set_flags(
            (libc::IFF_POINTOPOINT | libc::IFF_MULTICAST | libc::IFF_UP | libc::IFF_RUNNING) as _,
        )?)
    }

    fn down(&mut self) -> Result<(), Error> {
        Ok(self
            .handle()
            .set_flags((libc::IFF_POINTOPOINT | libc::IFF_MULTICAST) as _)?)
    }

    fn handle(&self) -> netconfig::InterfaceHandle {
        netconfig::InterfaceHandle::try_from_name(self.name()).unwrap()
    }
}

impl<Q> UtunInterface<Q> {
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub type Interface = UtunInterface<Queue>;

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
