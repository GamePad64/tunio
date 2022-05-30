#[cfg(target_os = "linux")]
pub mod linux;
mod util;
#[cfg(target_os = "macos")]
pub mod utun;
#[cfg(target_os = "windows")]
pub mod wintun;
