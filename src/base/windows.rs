use rand::Rng;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::NetworkManagement::IpHelper;
use windows::Win32::Networking::WinSock;
use crate::base::error::PingError;

pub enum WindowsError {
    IcmpCreateFileError,
    UnknownError(u32),
}

pub struct SinglePing {
    event: Option<windows::Win32::Foundation::HANDLE>,
    apc_routine: Option<windows::Win32::System::IO::PIO_APC_ROUTINE>,
    apc_context: Option<*const core::ffi::c_void>,
    request_option: Option<*const IpHelper::IP_OPTION_INFORMATION>,
    // if you want to use above variables, please read
    // https://learn.microsoft.com/en-us/windows/win32/api/icmpapi/nf-icmpapi-icmpsendecho2 for ipv4
    // https://learn.microsoft.com/en-us/windows/win32/api/icmpapi/nf-icmpapi-icmp6sendecho2 for ipv6
    timeout: u32, //ms
}

impl Default for SinglePing {
    fn default() -> Self {
        Self {
            event: None,
            apc_routine: None,
            apc_context: None,
            request_option: None,
            timeout: 1000,
        }
    }
}

impl SinglePing {
    #[inline]
    pub fn new(timeout: u32) -> Self {
        Self {
            event: None,
            apc_routine: None,
            apc_context: None,
            request_option: None,
            timeout,
        }
    }

    pub fn ping_v4(&self, addr: std::net::Ipv4Addr) -> Result<std::time::Duration, PingError> {
        unsafe {
            let handler: windows::Win32::Foundation::HANDLE = match IpHelper::IcmpCreateFile() {
                Ok(v) => v,
                Err(_e) => return Err(WindowsError::IcmpCreateFileError.into()),
            };
            let des = addr.to_bits();
            let request_data: u128 = rand::rng().random();
            let start_time = std::time::Instant::now();

            const REPLY_BUFFER_SIZE: usize =
                size_of::<IpHelper::ICMP_ECHO_REPLY>() + size_of::<u128>() + 8;

            let reply_buffer = [0_u8; REPLY_BUFFER_SIZE];

            let reply_count = IpHelper::IcmpSendEcho2(
                handler,
                self.event,
                self.apc_routine,
                self.apc_context,
                des,
                request_data.to_be_bytes().as_ptr() as *mut _,
                size_of_val(&request_data) as _,
                self.request_option,
                reply_buffer.as_ptr() as *mut _,
                reply_buffer.len() as _,
                self.timeout,
            );

            if reply_count != 0 {
                Ok(std::time::Instant::now().duration_since(start_time))
            } else {
                let error = GetLastError();
                Err(WindowsError::UnknownError(error.0).into())
            }
        }
    }

    pub fn ping_v6(&self, addr: std::net::Ipv6Addr) -> Result<std::time::Duration, PingError> {
        unsafe {
            let handler: windows::Win32::Foundation::HANDLE = match IpHelper::Icmp6CreateFile() {
                Ok(v) => v,
                Err(_e) => return Err(WindowsError::IcmpCreateFileError.into()),
            };
            let request_data: u128 = rand::rng().random();
            let start_time = std::time::Instant::now();

            const REPLY_BUFFER_SIZE: usize =
                size_of::<IpHelper::ICMP_ECHO_REPLY>() + size_of::<u128>() + 8;

            let reply_buffer = [0_u8; REPLY_BUFFER_SIZE];

            let reply_count = IpHelper::Icmp6SendEcho2(
                handler,
                self.event,
                self.apc_routine,
                self.apc_context,
                &WinSock::SOCKADDR_IN6 {
                    sin6_family: WinSock::AF_INET6,
                    sin6_port: 0,
                    sin6_flowinfo: 0,
                    sin6_addr: WinSock::IN6_ADDR {
                        u: WinSock::IN6_ADDR_0 {
                            Byte: std::mem::zeroed(),
                        },
                    },
                    Anonymous: Default::default(),
                },
                &WinSock::SOCKADDR_IN6 {
                    sin6_family: WinSock::AF_INET6,
                    sin6_port: 0,
                    sin6_flowinfo: 0,
                    sin6_addr: WinSock::IN6_ADDR {
                        u: WinSock::IN6_ADDR_0 {
                            Byte: std::mem::transmute(addr),
                        },
                    },
                    Anonymous: Default::default(),
                },
                request_data.to_be_bytes().as_ptr() as *mut _,
                size_of_val(&request_data) as _,
                self.request_option,
                reply_buffer.as_ptr() as *mut _,
                reply_buffer.len() as _,
                self.timeout,
            );

            if reply_count != 0 {
                Ok(std::time::Instant::now().duration_since(start_time))
            } else {
                let error = GetLastError();
                Err(WindowsError::UnknownError(error.0).into())
            }
        }
    }

    #[inline]
    pub fn set_event(&mut self, event: windows::Win32::Foundation::HANDLE) {
        self.event = Some(event)
    }

    #[inline]
    pub fn set_apc_routine(&mut self, apc_routine: windows::Win32::System::IO::PIO_APC_ROUTINE) {
        self.apc_routine = Some(apc_routine)
    }

    #[inline]
    pub fn set_apc_context(&mut self, apc_context: *const core::ffi::c_void) {
        self.apc_context = Some(apc_context)
    }

    #[inline]
    pub fn set_request_option(&mut self, request_option: *const IpHelper::IP_OPTION_INFORMATION) {
        self.request_option = Some(request_option)
    }

    #[inline]
    pub fn set_timeout(&mut self, timeout: u32) {
        self.timeout = timeout
    }
}

#[cfg(test)]
mod tests {
    use crate::base::windows::SinglePing;

    #[test]
    fn test_ping_v4() {
        let ping = SinglePing::default();
        println!(
            "{} ms",
            ping.ping_v4(std::net::Ipv4Addr::new(1, 1, 1, 1))
                .expect("ping_v4 error")
                .as_micros() as f64
                / 1000.0
        );
    }

    #[test]
    fn test_ping_v6() {
        let ping = SinglePing::default();
        println!(
            "{} ms",
            ping.ping_v6("2408:8756:c52:1aec:0:ff:b013:5a11".parse().unwrap())
                .expect("ping_v6 error")
                .as_micros() as f64
                / 1000.0
        );
    }
}
