use crate::traits::{AsyncQueueT, QueueT};
use delegate::delegate;
use std::io::{Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fs, io};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct Queue(fs::File);

impl QueueT for Queue {}

impl Queue {
    pub(crate) fn new(device: fs::File) -> Self {
        Self(device)
    }
}

impl Read for Queue {
    delegate! {
        to self.0 {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error>;
        }
    }
}

impl Write for Queue {
    delegate! {
        to self.0 {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }
}

pub struct AsyncTokioQueue(tokio::fs::File);

impl AsyncTokioQueue {
    pub(crate) fn new(device: fs::File) -> Self {
        Self(tokio::fs::File::from_std(device))
    }
}

impl AsyncQueueT for AsyncTokioQueue {}

impl AsyncRead for AsyncTokioQueue {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for AsyncTokioQueue {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
}
