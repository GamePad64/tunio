pub mod builder;
pub mod error;
#[cfg(target_os = "linux")]
mod linux;
pub mod traits;
#[cfg(target_os = "windows")]
pub mod wintun;

pub use builder::*;
pub use error::*;
pub use traits::*;

#[cfg(target_os = "windows")]
pub type DefaultDriver = wintun::driver::WinTunDriver;
#[cfg(target_os = "windows")]
pub type DefaultInterface = wintun::interface::WinTunInterface;

#[cfg(target_os = "linux")]
pub type DefaultDriver = linux::LinuxDriver;
#[cfg(target_os = "linux")]
pub type DefaultInterface = linux::interface::LinuxInterface;
