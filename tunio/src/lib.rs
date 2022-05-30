#![doc = include_str!("../../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
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
        #[cfg(feature = "async-tokio")]
        pub type DefaultTokioInterface = platform::wintun::AsyncTokioInterface;
    }else if #[cfg(target_os = "linux")] {
        pub type DefaultDriver = platform::linux::Driver;
        pub type DefaultInterface = platform::linux::Interface;
        #[cfg(feature = "async-tokio")]
        pub type DefaultTokioInterface = platform::linux::AsyncTokioInterface;
    }else if #[cfg(target_os = "macos")] {
        pub type DefaultDriver = platform::utun::Driver;
        pub type DefaultInterface = platform::utun::Interface;
    }
}
