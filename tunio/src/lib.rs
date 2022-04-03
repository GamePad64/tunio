pub mod config;
pub mod platform;
pub mod traits;

mod error;

pub use error::Error;

#[cfg(target_os = "windows")]
pub type DefaultDriver = platform::wintun::Driver;
#[cfg(target_os = "windows")]
pub type DefaultInterface = platform::wintun::Interface;

#[cfg(target_os = "linux")]
pub type DefaultDriver = platform::linux::LinuxDriver;
#[cfg(target_os = "linux")]
pub type DefaultInterface = platform::linux::interface::LinuxInterface;
