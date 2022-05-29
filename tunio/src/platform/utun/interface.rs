use crate::platform::utun::queue::Queue;
use crate::platform::utun::PlatformIfConfig;
use crate::traits::{InterfaceT, QueueT};
use crate::{Error, IfConfig};
use delegate::delegate;
use netconfig::sys::InterfaceHandleExt;
use std::io::{self, Read, Write};

pub struct Interface {
    name: String,
    queue: Queue,
}

impl InterfaceT for Interface {
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

impl Interface {
    pub(crate) fn new(params: IfConfig<PlatformIfConfig>) -> Result<Self, Error> {
        let queue = Queue::new(&params.name)?;

        Ok(Self {
            name: params.name,
            queue,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl QueueT for Interface {}

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
