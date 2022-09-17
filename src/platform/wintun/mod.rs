mod config;
mod driver;
mod event;
mod interface;
mod logger;
mod queue;
mod wrappers;

pub use config::{PlatformIfConfig, PlatformIfConfigBuilder};
pub use driver::Driver;
pub use interface::Interface;
pub use queue::Queue;

cfg_if::cfg_if! {
    if #[cfg(feature = "async-tokio")] {
        mod tokio_interface;
        mod tokio_queue;

        pub use tokio_interface::AsyncInterface;
        pub use tokio_queue::AsyncQueue;
    }
}
