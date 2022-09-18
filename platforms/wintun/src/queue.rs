use super::wrappers::Session;
use std::io::{self, Read, Write};
use tunio_core::traits::SyncQueueT;

pub trait SessionQueueT {
    fn new(session: Session) -> Self;
}

impl SyncQueueT for Queue {}

pub struct Queue {
    session: Session,
}

impl SessionQueueT for Queue {
    fn new(session: Session) -> Self {
        Self { session }
    }
}

impl Read for Queue {
    delegate::delegate! {
        to self.session {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
        }
    }
}

impl Write for Queue {
    delegate::delegate! {
        to self.session {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }
}
