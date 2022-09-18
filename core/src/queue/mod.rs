use std::os::unix::io::OwnedFd;

pub mod syncfd;
pub mod tokiofd;

pub trait FdQueueT {
    fn new(device: OwnedFd) -> Self;
}
