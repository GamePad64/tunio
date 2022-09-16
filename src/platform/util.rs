use std::os::unix::io::OwnedFd;

pub(crate) trait QueueFdT {
    fn new(device: OwnedFd) -> Self;
}

pub(crate) mod sync {
    use super::QueueFdT;
    use crate::traits::SyncQueueT;
    use delegate::delegate;
    use std::fs;
    use std::io::{self, Read, Write};
    use std::os::unix::io::OwnedFd;

    pub struct Queue(fs::File);

    impl SyncQueueT for Queue {}

    impl QueueFdT for Queue {
        fn new(device: OwnedFd) -> Self {
            Self(device.into())
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
}

#[cfg(feature = "async-tokio")]
pub(crate) mod async_tokio {
    use super::QueueFdT;
    use crate::traits::AsyncTokioQueueT;
    use std::io;
    use std::os::unix::io::{FromRawFd, IntoRawFd, OwnedFd};
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    pub struct AsyncTokioQueue(tokio::fs::File);

    impl QueueFdT for AsyncTokioQueue {
        fn new(device: OwnedFd) -> Self {
            unsafe { Self(tokio::fs::File::from_raw_fd(device.into_raw_fd())) }
        }
    }

    impl AsyncTokioQueueT for AsyncTokioQueue {}

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

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), io::Error>> {
            Pin::new(&mut self.0).poll_flush(cx)
        }

        fn poll_shutdown(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), io::Error>> {
            Pin::new(&mut self.0).poll_flush(cx)
        }
    }
}
