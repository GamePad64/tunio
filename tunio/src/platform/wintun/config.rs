use crate::traits::PlatformIfConfigT;
use derive_builder::Builder;

#[derive(Builder, Clone, Default)]
pub struct PlatformIfConfig {
    #[builder(default = "2 * 1024 * 1024")]
    pub(crate) capacity: u32,
    pub(crate) description: String,
}

impl PlatformIfConfigT for PlatformIfConfig {
    type Builder = PlatformIfConfigBuilder;
}
