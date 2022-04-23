pub mod config;
pub mod platform;
pub mod traits;

mod error;

pub use error::Error;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub type DefaultDriver = platform::wintun::Driver;
        pub type DefaultInterface = platform::wintun::Interface;
    } else if #[cfg(target_os = "linux")] {
        pub type DefaultDriver = platform::linux::Driver;
        pub type DefaultInterface = platform::linux::interface::LinuxInterface;
    }
}
