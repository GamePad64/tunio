use crate::config::IfaceConfig;
use crate::platform::linux::queue::Queue;
use crate::platform::linux::Driver;
use crate::traits::InterfaceT;
use crate::Error;
use netconfig::sys::InterfaceHandleExt;
use std::pin::Pin;
use std::task::{Context, Poll};
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

impl AsyncRead for LinuxInterface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.queue).poll_read(cx, buf)
    }
}

impl AsyncWrite for LinuxInterface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.queue).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.queue).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.queue).poll_flush(cx)
    }
}
