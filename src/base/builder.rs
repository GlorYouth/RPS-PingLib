use crate::base::{PingV4, PingV6};

pub struct PingV4Builder {
    pub timeout: u32, //ms
    pub ttl: Option<u8>,
    pub bind_addr: Option<std::net::Ipv4Addr>,
    #[cfg(target_os = "windows")]
    pub window_addition: Option<WindowAddition>,
}

impl Default for PingV4Builder {
    fn default() -> Self {
        Self {
            timeout: 1000,
            ttl: None,
            bind_addr: None,
            #[cfg(target_os = "windows")]
            window_addition: None,
        }
    }
}

impl PingV4Builder {
    pub fn new(timeout: u32) -> Self {
        Self {
            timeout,
            ttl: None,
            bind_addr: None,
            #[cfg(target_os = "windows")]
            window_addition: None,
        }
    }

    #[inline]
    pub fn build(self) -> PingV4 {
        self.into()
    }
}

pub struct PingV6Builder {
    pub timeout: u32, //ms
    pub ttl: Option<u8>,
    pub bind_addr: Option<std::net::Ipv6Addr>,
    pub scope_id_option: Option<u32>,
    #[cfg(target_os = "windows")]
    pub window_addition: Option<WindowAddition>,
}

impl Default for PingV6Builder {
    fn default() -> Self {
        Self {
            timeout: 1000,
            ttl: None,
            bind_addr: None,
            scope_id_option: None,
            #[cfg(target_os = "windows")]
            window_addition: None,
        }
    }
}

impl PingV6Builder {
    pub fn new(timeout: u32) -> Self {
        Self {
            timeout,
            ttl: None,
            bind_addr: None,
            scope_id_option: None,
            #[cfg(target_os = "windows")]
            window_addition: None,
        }
    }

    pub fn build(self) -> PingV6 {
        self.into()
    }
}

#[cfg(target_os = "windows")]
pub struct WindowAddition {
    pub event: Option<windows::Win32::Foundation::HANDLE>,
    pub apc_routine: windows::Win32::System::IO::PIO_APC_ROUTINE,
    pub apc_context: Option<*const core::ffi::c_void>,
    // if you want to use above variables, please read
    // https://learn.microsoft.com/en-us/windows/win32/api/icmpapi/nf-icmpapi-icmpsendecho2ex for ipv4
    // https://learn.microsoft.com/en-us/windows/win32/api/icmpapi/nf-icmpapi-icmp6sendecho2 for ipv6
}
