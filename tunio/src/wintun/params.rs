use crate::builder::{DriverBuilder, InterfaceBuilder};

pub struct WinTunDriverParams {
    pub library_name: Option<String>,
}

impl<'a> From<DriverBuilder<'a>> for WinTunDriverParams {
    fn from(builder: DriverBuilder) -> Self {
        Self {
            library_name: builder.wintun_library_name.map(str::to_string),
        }
    }
}

pub struct WinTunInterfaceParams {
    pub name: String,
    pub description: String,
}

impl<'a> From<InterfaceBuilder<'a>> for WinTunInterfaceParams {
    fn from(builder: InterfaceBuilder) -> Self {
        Self {
            name: builder.name.to_string(),
            description: builder.description.to_string(),
        }
    }
}
