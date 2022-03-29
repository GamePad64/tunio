use crate::{DriverBuilder, InterfaceBuilder};

pub struct LinuxDriverParams {}

impl<'a> From<DriverBuilder<'a>> for LinuxDriverParams {
    fn from(builder: DriverBuilder) -> Self {
        Self {}
    }
}

pub struct LinuxInterfaceParams {
    pub name: String,
}

impl<'a> From<InterfaceBuilder<'a>> for LinuxInterfaceParams {
    fn from(builder: InterfaceBuilder) -> Self {
        Self {
            name: builder.name.to_string(),
        }
    }
}
