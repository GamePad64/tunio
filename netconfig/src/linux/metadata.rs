use crate::linux::handle::InterfaceHandleExt;
use crate::MetadataCommonT;

#[derive(Default)]
pub struct Metadata {
    pub(crate) name: String,
    pub(crate) mtu: u32,
}

impl MetadataCommonT for Metadata {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle(&self) -> crate::InterfaceHandle {
        crate::InterfaceHandle::from_name(&*self.name())
    }

    fn mtu(&self) -> u32 {
        self.mtu
    }
}
