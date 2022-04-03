#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Error {
    UnexpectedMetadata,
    InterfaceNotFound,
    InternalError,
    AccessDenied,
}
