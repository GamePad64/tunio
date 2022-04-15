// use netconfig::win::MetadataExt;
use netconfig::{list_addresses, list_interfaces};
use netconfig::{InterfaceHandleCommonT, MetadataCommonT};

fn main() {
    env_logger::init();

    for handle in list_interfaces().iter() {
        let metadata = handle.metadata().unwrap();
        println!("Name: {}", metadata.name());
        // println!("Alias: {}", metadata.alias());
        // println!("GUID: {:?}", metadata.guid());
        // println!("LUID: {:?}", unsafe { metadata.luid().Value });
        println!("MTU: {}", metadata.mtu());
        //
        for address in handle.get_addresses().unwrap() {
            println!("Address: {:?}", address);
        }
        println!();
    }

    // println!("Addresses: {:?}", list_addresses())
}
