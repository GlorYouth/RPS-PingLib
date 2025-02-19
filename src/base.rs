mod error;
#[cfg(not(target_os = "windows"))]
mod linux;
#[cfg(not(target_os = "windows"))]
pub use linux::PingV4;
#[cfg(not(target_os = "windows"))]
pub use linux::PingV6;
#[cfg(target_os = "windows")]
mod windows;
mod builder;

#[cfg(target_os = "windows")]
pub use windows::PingV4;
#[cfg(target_os = "windows")]
pub use windows::PingV6;

pub use builder::{ PingV4Builder,PingV6Builder };