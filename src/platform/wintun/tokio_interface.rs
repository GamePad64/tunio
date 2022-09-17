use super::interface::CommonInterface;
use super::tokio_queue::AsyncQueue;
use futures::{AsyncRead, AsyncWrite};
use std::io::{self, ErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};

pub type AsyncInterface = CommonInterface<AsyncQueue>;

impl AsyncRead for AsyncInterface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_read(cx, buf),
            None => Poll::Ready(Err(io::Error::from(ErrorKind::BrokenPipe))),
        }
    }
}

impl AsyncWrite for AsyncInterface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_write(cx, buf),
            None => Poll::Ready(Err(io::Error::from(ErrorKind::BrokenPipe))),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_flush(cx),
            None => Poll::Ready(Err(io::Error::from(ErrorKind::BrokenPipe))),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_close(cx),
            None => Poll::Ready(Err(io::Error::from(ErrorKind::BrokenPipe))),
        }
    }
}
