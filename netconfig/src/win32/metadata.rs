use crate::win32::handle::{InterfaceHandle, InterfaceHandleExt};
use crate::MetadataCommonT;
use windows::core::GUID;
use windows::Win32::NetworkManagement::IpHelper::NET_LUID_LH;

pub trait MetadataExt {
    fn luid(&self) -> NET_LUID_LH;
    fn guid(&self) -> GUID;
    fn index(&self) -> u32;
    fn alias(&self) -> String;
    fn description(&self) -> String;
}

impl MetadataExt for crate::Metadata {
    fn luid(&self) -> NET_LUID_LH {
        self.0.luid
    }

    fn guid(&self) -> GUID {
        self.0.guid
    }

    fn index(&self) -> u32 {
        self.0.index
    }

    fn alias(&self) -> String {
        self.0.alias.clone()
    }

    fn description(&self) -> String {
        self.0.description.clone()
    }
}

#[derive(Default)]
pub struct Metadata {
    pub(crate) luid: NET_LUID_LH,
    pub(crate) guid: GUID,
    pub(crate) index: u32,
    pub(crate) mtu: u32,
    pub(crate) name: String,
    pub(crate) alias: String,
    pub(crate) description: String,
}

impl MetadataCommonT for Metadata {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle(&self) -> crate::InterfaceHandle {
        crate::InterfaceHandle(InterfaceHandle::from_luid(self.luid))
    }

    fn mtu(&self) -> u32 {
        self.mtu
    }
}
