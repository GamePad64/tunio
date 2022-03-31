use crate::traits::PlatformIfaceConfigT;
use getset::{FluentSetters, Getters};

#[derive(Getters, FluentSetters, Default)]
#[getset(get = "pub", set_fluent = "pub")]
pub struct WinTunPlatformIfaceConfig {
    pub description: String,
}

impl PlatformIfaceConfigT for WinTunPlatformIfaceConfig {}
