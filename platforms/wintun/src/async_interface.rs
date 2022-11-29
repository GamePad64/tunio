use super::async_queue::AsyncQueue;
use super::interface::CommonInterface;
use futures::{AsyncRead, AsyncWrite};
use std::io::{self};
use std::pin::Pin;
use std::task::{Context, Poll};

pub type AsyncInterface = CommonInterface<AsyncQueue>;

impl AsyncRead for AsyncInterface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.inner_queue_mut() {
            Ok(queue) => Pin::new(queue).poll_read(cx, buf),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl AsyncWrite for AsyncInterface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.inner_queue_mut() {
            Ok(queue) => Pin::new(queue).poll_write(cx, buf),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.inner_queue_mut() {
            Ok(queue) => Pin::new(queue).poll_flush(cx),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.inner_queue_mut() {
            Ok(queue) => Pin::new(queue).poll_close(cx),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

#[cfg(feature = "tokio")]
impl tokio::io::AsyncRead for AsyncInterface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.inner_queue_mut() {
            Ok(queue) => {
                let ret = Pin::new(queue).poll_read(cx, buf.initialize_unfilled());
                if let Poll::Ready(Ok(n)) = ret {
                    buf.set_filled(buf.filled().len() + n);
                }
                ret.map_ok(|_len| ())
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

#[cfg(feature = "tokio")]
impl tokio::io::AsyncWrite for AsyncInterface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.inner_queue_mut() {
            Ok(queue) => Pin::new(queue).poll_write(cx, buf),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.inner_queue_mut() {
            Ok(queue) => Pin::new(queue).poll_flush(cx),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.inner_queue_mut() {
            Ok(queue) => Pin::new(queue).poll_close(cx),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
