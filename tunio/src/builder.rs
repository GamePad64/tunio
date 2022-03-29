#[derive(Default)]
pub struct DriverBuilder<'a> {
    pub(crate) wintun_library_name: Option<&'a str>,
}

impl<'a> DriverBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wintun_library_name(mut self, wintun_library_name: &'a str) -> Self {
        self.wintun_library_name = Some(wintun_library_name);
        self
    }
}

#[derive(Default)]
pub struct InterfaceBuilder<'a> {
    pub(crate) name: &'a str,
    pub(crate) description: &'a str,
}

impl<'a> InterfaceBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }
}
