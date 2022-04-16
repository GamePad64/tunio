#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Error {
    UnexpectedMetadata,
    InterfaceNotFound,
    InternalError,
    AccessDenied,
}

#[cfg(not(target_os = "windows"))]
impl From<nix::Error> for Error {
    fn from(_: nix::Error) -> Self {
        Error::InternalError
    }
}
