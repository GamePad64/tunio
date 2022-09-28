use std::os::unix::io::OwnedFd;

pub mod syncfd;
#[cfg(feature = "tokio")]
pub mod tokiofd;

pub trait FdQueueT {
    const BLOCKING: bool;

    fn new(device: OwnedFd) -> Self;
}
