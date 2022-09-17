use crate::async_queue::AsyncQueue;
use crate::traits::SyncQueueT;
use delegate::delegate;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::io::OwnedFd;

pub trait FdQueueT {
    fn new(device: OwnedFd) -> Self;
}

pub struct SyncFdQueue(fs::File);

impl SyncQueueT for SyncFdQueue {}

impl FdQueueT for SyncFdQueue {
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

pub type AsyncFdQueue = AsyncQueue<SyncFdQueue>;

impl FdQueueT for AsyncFdQueue {
    fn new(device: OwnedFd) -> Self {
        SyncFdQueue::new(device).into()
    }
}
