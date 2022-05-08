use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InterfaceNameUnicodeError,
    InterfaceNameTooLong(usize, usize),
    LibraryNotLoaded { reason: String },
    InterfaceStateInvalid,
    NetConfigError(netconfig::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<netconfig::Error> for Error {
    fn from(err: netconfig::Error) -> Self {
        Error::NetConfigError(err)
    }
}
