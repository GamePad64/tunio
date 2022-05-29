#![doc = include_str!("../../README.md")]
pub(crate) mod config;
pub mod platform;
pub mod traits;

mod error;

pub use config::*;
pub use error::Error;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub type DefaultDriver = platform::wintun::Driver;
        pub type DefaultInterface = platform::wintun::Interface;
    }else if #[cfg(target_os = "linux")] {
        pub type DefaultDriver = platform::linux::Driver;
        pub type DefaultInterface = platform::linux::Interface;
    }else if #[cfg(target_os = "macos")] {
        pub type DefaultDriver = platform::utun::Driver;
        pub type DefaultInterface = platform::utun::Interface;
    }
}
