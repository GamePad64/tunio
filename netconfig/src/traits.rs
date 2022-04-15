use crate::{Error, Metadata};
use crate::{InterfaceHandle, IpNet};

pub trait MetadataCommonT {
    fn name(&self) -> String;
    fn handle(&self) -> InterfaceHandle;
    fn mtu(&self) -> u32;
}

pub trait InterfaceHandleCommonT {
    fn metadata(&self) -> Result<Metadata, Error>;
    fn add_ip(&self, network: IpNet);
    fn remove_ip(&self, network: IpNet);
    fn get_addresses(&self) -> Result<Vec<IpNet>, Error>;
    fn set_mtu(&self, mtu: u32) -> Result<(), Error>;
}
