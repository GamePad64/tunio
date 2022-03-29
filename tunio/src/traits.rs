use std::sync::Arc;

pub trait Driver {
    type DriverParamsT;

    fn new(params: Self::DriverParamsT) -> Result<Arc<Self>, crate::Error>;
}

pub trait Interface {
    type DriverT: Driver;
    type InterfaceParamsT;

    fn new(
        driver: Arc<Self::DriverT>,
        params: Self::InterfaceParamsT,
    ) -> Result<Arc<Self>, crate::Error>;
}
