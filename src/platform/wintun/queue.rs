use super::wrappers::Session;
use crate::traits::SyncQueueT;
use std::io::{self, Read, Write};

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
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.session.read(buf)
    }
}

impl Write for Queue {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.session.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.session.flush()
    }
}
