use crate::traits::PlatformIfConfigT;
use derive_builder::Builder;

#[derive(Builder, Clone)]
pub struct PlatformIfConfig {
    /// Wintun ring capacity. Must be power of 2 between 128KiB and 64MiB
    #[builder(default = "2 * 1024 * 1024")]
    pub(crate) capacity: u32,
    #[builder(default = "String::new()")]
    pub(crate) description: String,
}

impl Default for PlatformIfConfig {
    fn default() -> Self {
        PlatformIfConfigBuilder::default().build().unwrap()
    }
}

impl PlatformIfConfigT for PlatformIfConfig {
    type Builder = PlatformIfConfigBuilder;
}
