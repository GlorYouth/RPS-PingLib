mod error;
#[cfg(not(target_os = "windows"))]
mod linux;
#[cfg(not(target_os = "windows"))]
pub use linux::PingV4;
#[cfg(not(target_os = "windows"))]
pub use linux::PingV6;
mod builder;
mod protocol;
mod result;
#[cfg(target_os = "windows")]
mod utils;
#[cfg(target_os = "windows")]
mod windows;

pub use result::*;

#[cfg(target_os = "windows")]
pub use windows::PingV4;
#[cfg(target_os = "windows")]
pub use windows::PingV6;

pub use builder::{PingV4Builder, PingV6Builder};
