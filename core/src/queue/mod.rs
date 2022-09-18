use std::os::unix::io::OwnedFd;

pub mod syncfd;
#[cfg(feature = "tokio")]
pub mod tokiofd;

pub trait FdQueueT {
    fn new(device: OwnedFd) -> Self;
}
