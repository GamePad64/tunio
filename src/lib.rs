#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub mod platform;

pub use tunio_core::config::*;
pub use tunio_core::Error;

pub use tunio_core::config;
pub use tunio_core::traits;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub type DefaultDriver = platform::wintun::Driver;
        pub type DefaultInterface = platform::wintun::Interface;
        pub type DefaultAsyncInterface = platform::wintun::AsyncInterface;
    }else if #[cfg(target_os = "linux")] {
        pub type DefaultDriver = platform::linux::Driver;
        pub type DefaultInterface = platform::linux::Interface;
        #[cfg(feature = "tokio")]
        pub type DefaultAsyncInterface = platform::linux::TokioInterface;
    }else if #[cfg(target_os = "macos")] {
        pub type DefaultDriver = platform::utun::Driver;
        pub type DefaultInterface = platform::utun::Interface;
        #[cfg(feature = "tokio")]
        pub type DefaultAsyncInterface = platform::utun::TokioInterface;
    }
}
