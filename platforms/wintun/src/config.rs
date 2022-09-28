use derive_builder::Builder;
use tunio_core::traits::PlatformIfConfigT;

/// It is generally better to use [`PlatformIfConfigBuilder`] to create a new PlatformIfConfig instance.
#[derive(Builder, Clone)]
pub struct PlatformIfConfig {
    /// Wintun ring capacity. Must be power of 2 between 128KiB and 64MiB
    #[builder(default = "2 * 1024 * 1024")]
    pub capacity: u32,
    #[builder(default = "String::new()")]
    pub description: String,
    /// GUID of this network interface. It is recommended to set it manually,
    /// or new device will be created on each invocation, and it will quickly
    /// pollute Windows registry.
    #[builder(default = "windows::core::GUID::new().unwrap().to_u128()")]
    pub guid: u128,
}

impl Default for PlatformIfConfig {
    fn default() -> Self {
        PlatformIfConfigBuilder::default().build().unwrap()
    }
}

impl PlatformIfConfigT for PlatformIfConfig {
    type Builder = PlatformIfConfigBuilder;
}
