use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InterfaceNameUnicodeError,
    InterfaceNameTooLong(usize),
    LibraryNotLoaded { reason: String },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
