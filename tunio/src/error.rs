use std::io;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("interface name contains invalid characters")]
    InterfaceNameUnicodeError,
    #[error("interface name is too long")]
    InterfaceNameTooLong(usize),
    #[error("tun library not loaded: {reason:?}")]
    LibraryNotLoaded { reason: String },
    #[error("netconfig error: {0}")]
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
