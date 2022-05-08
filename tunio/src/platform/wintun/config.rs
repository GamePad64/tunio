use crate::traits::PlatformIfaceConfigT;

#[derive(Clone)]
pub struct PlatformInterfaceConfig {
    capacity: u32,
    description: String,
}

impl PlatformInterfaceConfig {
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn set_capacity(mut self, capacity: u32) -> Self {
        self.capacity = capacity;
        self
    }
    pub fn set_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
}

impl Default for PlatformInterfaceConfig {
    fn default() -> Self {
        Self {
            capacity: 2 * 1024 * 1024,
            description: "".into(),
        }
    }
}

impl PlatformIfaceConfigT for PlatformInterfaceConfig {}
