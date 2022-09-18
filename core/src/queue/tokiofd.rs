use crate::queue::syncfd::SyncFdQueue;
use crate::queue::FdQueueT;
use crate::traits::AsyncQueueT;
use futures::{AsyncRead, AsyncWrite};
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::OwnedFd;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tokio::io::unix::AsyncFd;

pub struct TokioFdQueue {
    inner: AsyncFd<SyncFdQueue>,
}

impl AsyncQueueT for TokioFdQueue {}

impl FdQueueT for TokioFdQueue {
    fn new(device: OwnedFd) -> Self {
        Self {
            inner: AsyncFd::new(SyncFdQueue::new(device)).unwrap(),
        }
    }
}

impl AsyncRead for TokioFdQueue {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let self_mut = self.get_mut();
        loop {
            let mut guard = ready!(self_mut.inner.poll_read_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().read(buf)) {
                Ok(Ok(n)) => {
                    return Poll::Ready(Ok(n));
                }
                Ok(Err(e)) => return Poll::Ready(Err(e)),
                Err(_) => continue,
            }
        }
    }
}

impl AsyncWrite for TokioFdQueue {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let self_mut = self.get_mut();
        loop {
            let mut guard = ready!(self_mut.inner.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(Ok(n)) => {
                    return Poll::Ready(Ok(n));
                }
                Ok(Err(e)) => return Poll::Ready(Err(e)),
                Err(_) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let self_mut = self.get_mut();
        loop {
            let mut guard = ready!(self_mut.inner.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().flush()) {
                Ok(Ok(())) => {
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(e)) => return Poll::Ready(Err(e)),
                Err(_) => continue,
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
