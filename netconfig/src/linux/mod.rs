use std::fs;

pub use handle::{InterfaceHandle, InterfaceHandleExt};
pub use metadata::Metadata;

pub mod handle;
mod ifreq;
pub mod metadata;

pub fn list_interfaces() -> Vec<crate::InterfaceHandle> {
    let mut result = vec![];

    for path in fs::read_dir("/sys/class/net").expect("Path is not available") {
        let handle = InterfaceHandle::from_name(path.unwrap().file_name().to_str().unwrap());
        result.push(crate::InterfaceHandle(handle));
    }
    result
}
