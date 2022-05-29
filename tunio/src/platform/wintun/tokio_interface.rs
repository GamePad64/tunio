use super::interface::CommonInterface;
use super::tokio_queue::AsyncTokioQueue;
use super::Driver;
use super::PlatformIfConfig;
use crate::config::IfConfig;
use crate::traits::InterfaceT;
use crate::Error;
use std::io::ErrorKind;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct AsyncTokioInterface {
    inner: CommonInterface,
    queue: Option<AsyncTokioQueue>,
}

impl InterfaceT for AsyncTokioInterface {
    type PlatformDriver = Driver;
    type PlatformIfConfig = PlatformIfConfig;

    fn new(
        driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error> {
        Ok(Self {
            inner: CommonInterface::new(driver.wintun(), params)?,
            queue: None,
        })
    }

    fn up(&mut self) -> Result<(), Error> {
        self.queue = Some(AsyncTokioQueue::new(self.inner.make_session()?));
        Ok(())
    }

    fn down(&mut self) -> Result<(), Error> {
        let _ = self.queue.take();
        Ok(())
    }

    fn handle(&self) -> netconfig::InterfaceHandle {
        self.inner.handle()
    }
}

impl AsyncRead for AsyncTokioInterface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_read(cx, buf),
            None => Poll::Ready(Err(std::io::Error::from(ErrorKind::BrokenPipe))),
        }
    }
}

impl AsyncWrite for AsyncTokioInterface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_write(cx, buf),
            None => Poll::Ready(Err(std::io::Error::from(ErrorKind::BrokenPipe))),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_flush(cx),
            None => Poll::Ready(Err(std::io::Error::from(ErrorKind::BrokenPipe))),
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_shutdown(cx),
            None => Poll::Ready(Err(std::io::Error::from(ErrorKind::BrokenPipe))),
        }
    }
}
