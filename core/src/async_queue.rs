use crate::traits::SyncQueueT;
use async_lock::Mutex;
use blocking::Unblock;
use delegate::delegate;
use futures::{AsyncRead, AsyncWrite};
use std::io;
use std::io::{Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct AsyncQueue<Q> {
    inner: Mutex<Unblock<Q>>,
}

impl<Q> From<Q> for AsyncQueue<Q> {
    fn from(q: Q) -> Self {
        Self {
            inner: Mutex::new(Unblock::new(q)),
        }
    }
}

impl<Q> AsyncRead for AsyncQueue<Q>
where
    Q: SyncQueueT + Read + Send + 'static,
{
    delegate! {
        to Pin::new(self.inner.get_mut()) {
            fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>>;
        }
    }
}

impl<Q> AsyncWrite for AsyncQueue<Q>
where
    Q: SyncQueueT + Write + Send + 'static,
{
    delegate! {
        to Pin::new(self.inner.get_mut()) {
            fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>>;
            fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>;
            fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>;
        }
    }
}
