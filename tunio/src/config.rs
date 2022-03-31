use crate::traits::DriverT;
use getset::{FluentSetters, Getters};

#[derive(Getters, FluentSetters)]
#[getset(get = "pub", set_fluent = "pub")]
pub struct IfaceConfig<Driver: DriverT> {
    pub(crate) name: String,

    #[getset(skip)]
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
