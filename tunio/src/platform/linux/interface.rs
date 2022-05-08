use super::PlatformIfConfig;
use crate::config::IfConfig;
use crate::platform::linux::queue::Queue;
use crate::traits::{AsyncQueueT, InterfaceT, QueueT};
use crate::Error;
use delegate::delegate;
use netconfig::sys::InterfaceHandleExt;
use std::io;
use std::io::{Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct Interface {
    name: String,
    queue: Queue,
}

impl Interface {
    pub(crate) fn new(params: IfConfig<PlatformIfConfig>) -> Result<Self, Error> {
        let queue = Queue::new(&*params.name, params.layer)?;

        Ok(Self {
            queue,
            name: params.name,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl InterfaceT for Interface {
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

impl QueueT for Interface {}
impl AsyncQueueT for Interface {}

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

impl AsyncRead for Interface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.queue).poll_read(cx, buf)
    }
}

impl AsyncWrite for Interface {
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
        Pin::new(&mut self.queue).poll_flush(cx)
    }
}
