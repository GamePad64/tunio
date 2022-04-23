use crate::traits::PlatformIfaceConfigT;

#[derive(Default)]
pub struct PlatformInterfaceConfig {
    pub description: String,
}

impl PlatformInterfaceConfig {
    pub fn description(&self) -> String {
        self.description
    }

    pub fn set_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
}

impl PlatformIfaceConfigT for PlatformInterfaceConfig {}
