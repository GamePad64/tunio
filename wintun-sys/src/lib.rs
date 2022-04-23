#[allow(non_snake_case, non_camel_case_types)]
#[cfg(target_os = "windows")]
mod wintun_h;
#[cfg(target_os = "windows")]
pub use wintun_h::*;
