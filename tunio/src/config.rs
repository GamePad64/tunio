use crate::traits::DriverT;

pub struct IfaceConfig<Driver: DriverT> {
    pub(crate) name: String,

    pub(crate) platform: Driver::PlatformInterfaceConfig,
}

impl<Driver: DriverT> Default for IfaceConfig<Driver> {
    fn default() -> Self {
        Self {
            name: "".into(),
            platform: Driver::PlatformInterfaceConfig::default(),
        }
    }
}

impl<Driver: DriverT> IfaceConfig<Driver> {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name<T: Into<String>>(mut self, name: T) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_platform<F>(self, f: F) -> IfaceConfig<Driver>
    where
        F: Fn(Driver::PlatformInterfaceConfig) -> Driver::PlatformInterfaceConfig,
    {
        let mut new_self = self;
        new_self.platform = f(new_self.platform);
        new_self
    }

    pub fn platform(&self) -> &Driver::PlatformInterfaceConfig {
        &self.platform
    }
}
