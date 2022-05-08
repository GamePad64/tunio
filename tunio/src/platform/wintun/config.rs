use crate::traits::PlatformIfConfigT;
use derive_builder::Builder;

#[derive(Builder, Clone)]
pub struct PlatformIfConfig {
    pub(crate) capacity: u32,
    pub(crate) description: String,
}

impl Default for PlatformIfConfig {
    fn default() -> Self {
        Self {
            capacity: 2 * 1024 * 1024,
            description: "".into(),
        }
    }
}

impl PlatformIfConfigT for PlatformIfConfig {
    type Builder = PlatformIfConfigBuilder;
}
