use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

pub trait DriverT {
    type DriverParamsT;

    fn new(params: Self::DriverParamsT) -> Result<Arc<Self>, crate::Error>;
}

pub trait InterfaceT {
    type DriverT: DriverT;
    type InterfaceParamsT;

    fn new(
        driver: Arc<Self::DriverT>,
        params: Self::InterfaceParamsT,
    ) -> Result<Self, crate::Error>
    where
        Self: Sized;

    fn is_ready(&self) -> bool;

    fn check_ready(&self) -> io::Result<()> {
        match self.is_ready() {
            true => Ok(()),
            false => Err(io::Error::new(
                ErrorKind::ConnectionRefused,
                "Tun interface is down",
            )),
        }
    }

    fn open(&mut self) -> Result<(), crate::Error>;
}
