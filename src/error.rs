use get_last_error::Win32Error;
use widestring::error::ContainsNul;

#[derive(Debug)]
pub enum Error {
    InterfaceNameUnicodeError,
    InterfaceNameTooLong(usize),
    UnknownWin32Error(Win32Error),
}

impl From<ContainsNul<u16>> for Error {
    fn from(_: ContainsNul<u16>) -> Self {
        Self::InterfaceNameUnicodeError
    }
}

impl From<Win32Error> for Error {
    fn from(e: Win32Error) -> Self {
        Self::UnknownWin32Error(e)
    }
}
