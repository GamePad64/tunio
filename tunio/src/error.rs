use crate::config::Layer;
use std::io;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("interface name is not valid Unicode")]
    InterfaceNameUnicodeError,
    #[error("interface name too long: {0} > {1}")]
    InterfaceNameTooLong(usize, usize),
    #[error("library not loaded: {reason}")]
    LibraryNotLoaded { reason: String },
    #[error("netconfig error: {0}")]
    NetConfigError(netconfig::Error),
    #[error("interface name error: {0}")]
    InterfaceNameError(String),
    #[error("config value is invalid ({reason}): {name}={value}")]
    InvalidConfigValue {
        name: String,
        value: String,
        reason: String,
    },
    #[error("layer is unsupported: {0:?}")]
    LayerUnsupported(Layer),
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
