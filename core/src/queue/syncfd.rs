use crate::queue::FdQueueT;
use crate::traits::SyncQueueT;
use delegate::delegate;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, OwnedFd, RawFd};

pub struct SyncFdQueue(fs::File);

impl SyncQueueT for SyncFdQueue {}

impl FdQueueT for SyncFdQueue {
    const BLOCKING: bool = true;

    fn new(device: OwnedFd) -> Self {
        Self(device.into())
    }
}

impl Read for SyncFdQueue {
    delegate! {
        to self.0 {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error>;
        }
    }
}

impl Write for SyncFdQueue {
    delegate! {
        to self.0 {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }
}

impl AsRawFd for SyncFdQueue {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
