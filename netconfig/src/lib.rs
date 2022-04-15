mod error;
mod traits;
pub use crate::traits::{InterfaceHandleCommonT, MetadataCommonT};
pub use error::Error;
pub use ipnet::IpNet;
use std::collections::HashSet;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub mod win;
        pub struct InterfaceHandle(win::InterfaceHandle);
        pub struct Metadata(win::Metadata);
    } else if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub struct InterfaceHandle(linux::InterfaceHandle);
        pub struct Metadata(linux::Metadata);
    }
}

impl MetadataCommonT for Metadata {
    fn name(&self) -> String {
        self.0.name()
    }

    fn handle(&self) -> InterfaceHandle {
        self.0.handle()
    }

    fn mtu(&self) -> u32 {
        self.0.mtu()
    }
}

impl InterfaceHandleCommonT for InterfaceHandle {
    fn metadata(&self) -> Result<Metadata, Error> {
        self.0.metadata()
    }

    fn add_ip(&self, network: IpNet) {
        self.0.add_ip(network)
    }

    fn remove_ip(&self, network: IpNet) {
        self.0.remove_ip(network)
    }

    fn get_addresses(&self) -> Result<Vec<IpNet>, Error> {
        self.0.get_addresses()
    }

    fn set_mtu(&self, mtu: u32) -> Result<(), Error> {
        self.0.set_mtu(mtu)
    }
}

pub fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            win::list_interfaces()
        } else if #[cfg(target_os = "linux")] {
            linux::list_interfaces()
        }
    }
}

pub fn list_addresses() -> Vec<IpNet> {
    let interfaces = list_interfaces();

    let addresses = interfaces
        .iter()
        .flat_map(|iface| iface.get_addresses())
        .flatten();

    HashSet::<IpNet>::from_iter(addresses)
        .iter()
        .cloned()
        .collect()
}
