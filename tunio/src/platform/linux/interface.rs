use crate::config::IfaceConfig;
use std::io::{Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::platform::linux::queue::Queue;
use crate::platform::linux::Driver;
use crate::traits::InterfaceT;
use crate::Error;

use netconfig::sys::InterfaceHandleExt;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct LinuxInterface {
    name: String,
    queue: Queue,
}

impl LinuxInterface {
    pub(crate) fn new(params: IfaceConfig<Driver>) -> Result<Self, Error> {
        let queue = Queue::new(&*params.name)?;

        Ok(Self {
            queue,
            name: params.name,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn queue(&mut self) -> &mut Queue {
        &mut self.queue
    }
}

impl InterfaceT for LinuxInterface {
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
