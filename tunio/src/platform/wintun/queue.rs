use super::wrappers::Session;
use crate::traits::QueueT;
use std::io::{self, Read, Write};

impl QueueT for Queue {}

pub struct Queue {
    session: Session,
}

impl Queue {
    pub(crate) fn new(session: Session) -> Self {
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
