use super::queue::{create_device, Device};
use super::Driver;
use super::PlatformIfConfig;
use crate::config::IfConfig;
use crate::platform::util::{AsyncTokioQueue, Queue};
use crate::traits::{InterfaceT, QueueT};
use crate::Error;
use delegate::delegate;
use log::debug;
use netconfig::sys::InterfaceHandleExt;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct AsyncTokioInterface {
    name: String,
    queue: AsyncTokioQueue,
}

impl AsyncTokioInterface {
    pub fn name(&self) -> &str {
        &*self.name
    }
}

impl InterfaceT for AsyncTokioInterface {
    type PlatformDriver = Driver;
    type PlatformIfConfig = PlatformIfConfig;

    fn new(
        _driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error> {
        let Device { device, name } = create_device(&*params.name, params.layer)?;
        let queue = AsyncTokioQueue::new(device);

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

impl AsyncRead for AsyncTokioInterface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.queue).poll_read(cx, buf)
    }
}

impl AsyncWrite for AsyncTokioInterface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.queue).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.queue).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.queue).poll_shutdown(cx)
    }
}
