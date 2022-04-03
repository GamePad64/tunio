mod error;
#[cfg(target_os = "windows")]
pub mod win;

pub use error::Error;
pub use ipnet::IpNet;
use std::collections::HashSet;

#[cfg(target_os = "windows")]
pub struct InterfaceHandle(win::InterfaceHandle);
#[cfg(target_os = "windows")]
pub struct Metadata(win::Metadata);

impl Metadata {
    pub fn name(&self) -> String {
        self.0.name().clone()
    }

    pub fn handle(&self) -> InterfaceHandle {
        self.0.handle()
    }

    pub fn mtu(&self) -> u32 {
        self.0.mtu()
    }
}

impl InterfaceHandle {
    pub fn metadata(&self) -> Result<Metadata, Error> {
        self.0.metadata()
    }

    pub fn add_ip(&self, network: IpNet) {
        self.0.add_ip(network)
    }

    pub fn remove_ip(&self, network: IpNet) {
        self.0.remove_ip(network)
    }

    pub fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
        self.0.get_addresses()
    }

    pub fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
        self.0.set_mtu(mtu)
    }
}

#[cfg(target_os = "windows")]
pub fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    win::list_interfaces()
}

pub fn list_addresses() -> Vec<IpNet> {
    let interfaces = list_interfaces();

    let addresses = interfaces
        .iter()
        .flat_map(|iface| iface.0.get_addresses())
        .flatten();

    HashSet::<IpNet>::from_iter(addresses)
        .iter()
        .cloned()
        .collect()
}
