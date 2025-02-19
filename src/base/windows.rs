use crate::base::builder::{PingV4Builder, PingV6Builder};
use crate::base::error::{PingError, SharedError};
use rand::Rng;
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper;
use windows::Win32::Networking::WinSock;

pub enum WindowsError {
    IcmpCreateFileError(String),
    IcmpCloseFileError(String),
    InvalidParameter, //maybe reply_buffer too small
    UnknownError(u32),
}

pub struct PingV4 {
    builder: PingV4Builder,
}


pub struct PingV6 {
    builder: PingV6Builder,
}


impl PingV4 {
    #[inline]
    pub fn new(builder: PingV4Builder) -> PingV4 {
        PingV4 { builder }
    }

    pub fn ping(&self, target: std::net::Ipv4Addr) -> Result<std::time::Duration, PingError> {
        unsafe {
            let handler: windows::Win32::Foundation::HANDLE = match IpHelper::IcmpCreateFile() {
                Ok(v) => v,
                Err(e) => return Err(WindowsError::IcmpCreateFileError(e.message()).into()),
            };
            let des = target.to_bits();
            let request_data: u128 = rand::rng().random();
            let start_time = std::time::Instant::now();

            const REPLY_BUFFER_SIZE: usize =
                size_of::<IpHelper::ICMP_ECHO_REPLY>() + size_of::<u128>() + 8;

            let reply_buffer = [0_u8; REPLY_BUFFER_SIZE];

            let reply_count = match &self.builder.window_addition {
                None => {
                    match self.builder.bind_addr {
                        None => {
                            IpHelper::IcmpSendEcho2(
                                handler,
                                None,
                                None,
                                None,
                                des,
                                request_data.to_be_bytes().as_ptr() as *mut _,
                                size_of_val(&request_data) as _,
                                None,
                                reply_buffer.as_ptr() as *mut _,
                                reply_buffer.len() as _,
                                self.builder.timeout,
                            )
                        }
                        Some(addr) => {
                            IpHelper::IcmpSendEcho2Ex(
                                handler,
                                None,
                                None,
                                None,
                                addr.to_bits(),
                                des,
                                request_data.to_be_bytes().as_ptr() as *mut _,
                                size_of_val(&request_data) as _,
                                None,
                                reply_buffer.as_ptr() as *mut _,
                                reply_buffer.len() as _,
                                self.builder.timeout,
                            )
                        }
                    }
                }
                Some(addition) => {
                    match self.builder.bind_addr {
                        None => {
                            IpHelper::IcmpSendEcho2(
                                handler,
                                addition.event,
                                addition.apc_routine,
                                addition.apc_context,
                                des,
                                request_data.to_be_bytes().as_ptr() as *mut _,
                                size_of_val(&request_data) as _,
                                None,
                                reply_buffer.as_ptr() as *mut _,
                                reply_buffer.len() as _,
                                self.builder.timeout,
                            )
                        }
                        Some(addr) => {
                            IpHelper::IcmpSendEcho2Ex(
                                handler,
                                addition.event,
                                addition.apc_routine,
                                addition.apc_context,
                                addr.to_bits(),
                                des,
                                request_data.to_be_bytes().as_ptr() as *mut _,
                                size_of_val(&request_data) as _,
                                None,
                                reply_buffer.as_ptr() as *mut _,
                                reply_buffer.len() as _,
                                self.builder.timeout,
                            )
                        }
                    }
                }
            };

            IpHelper::IcmpCloseHandle(handler)
                .map_err(|e| WindowsError::IcmpCloseFileError(e.message()))?;

            if reply_count != 0 {
                Ok(std::time::Instant::now().duration_since(start_time))
            } else {
                let error = GetLastError();
                Err(solve_recv_error(error))
            }
        }
    }
    
}

impl PingV6 {
    #[inline]
    pub fn new(builder: PingV6Builder) -> PingV6 {
        PingV6 { builder }
    }
    
    pub fn ping(&self, target: std::net::Ipv6Addr) -> Result<std::time::Duration, PingError> {
        unsafe {
            let handler: windows::Win32::Foundation::HANDLE = match IpHelper::Icmp6CreateFile() {
                Ok(v) => v,
                Err(e) => return Err(WindowsError::IcmpCreateFileError(e.message()).into()),
            };
            let request_data: u128 = rand::rng().random();
            let start_time = std::time::Instant::now();

            const REPLY_BUFFER_SIZE: usize =
                size_of::<IpHelper::ICMP_ECHO_REPLY>() + size_of::<u128>() + 8;

            let reply_buffer = [0_u8; REPLY_BUFFER_SIZE];
            
            let bind_addr = match self.builder.bind_addr {
                None => {std::mem::zeroed()}
                Some(addr) => {
                    std::mem::transmute(addr)
                }
            };

            let reply_count = match &self.builder.window_addition {
                None => {
                    IpHelper::Icmp6SendEcho2(
                        handler,
                        None,
                        None,
                        None,
                        &WinSock::SOCKADDR_IN6 {
                            sin6_family: WinSock::AF_INET6,
                            sin6_port: 0,
                            sin6_flowinfo: 0,
                            sin6_addr: WinSock::IN6_ADDR {
                                u: WinSock::IN6_ADDR_0 {
                                    Byte: bind_addr,
                                },
                            },
                            Anonymous: match self.builder.scope_id_option {
                                None => Default::default(),
                                Some(id) => WinSock::SOCKADDR_IN6_0 {
                                    sin6_scope_id: id,
                                }
                            },
                        },
                        &WinSock::SOCKADDR_IN6 {
                            sin6_family: WinSock::AF_INET6,
                            sin6_port: 0,
                            sin6_flowinfo: 0,
                            sin6_addr: WinSock::IN6_ADDR {
                                u: WinSock::IN6_ADDR_0 {
                                    Byte: std::mem::transmute(target),
                                },
                            },
                            Anonymous: match self.builder.scope_id_option {
                                None => Default::default(),
                                Some(id) => WinSock::SOCKADDR_IN6_0 {
                                    sin6_scope_id: id,
                                }
                            },
                        },
                        request_data.to_be_bytes().as_ptr() as *mut _,
                        size_of_val(&request_data) as _,
                        None,
                        reply_buffer.as_ptr() as *mut _,
                        reply_buffer.len() as _,
                        self.builder.timeout,
                    )
                }
                Some(addition) => {
                    IpHelper::Icmp6SendEcho2(
                        handler,
                        addition.event,
                        addition.apc_routine,
                        addition.apc_context,
                        &WinSock::SOCKADDR_IN6 {
                            sin6_family: WinSock::AF_INET6,
                            sin6_port: 0,
                            sin6_flowinfo: 0,
                            sin6_addr: WinSock::IN6_ADDR {
                                u: WinSock::IN6_ADDR_0 {
                                    Byte: bind_addr,
                                },
                            },
                            Anonymous: match self.builder.scope_id_option {
                                None => Default::default(),
                                Some(id) => WinSock::SOCKADDR_IN6_0 {
                                    sin6_scope_id: id,
                                }
                            },
                        },
                        &WinSock::SOCKADDR_IN6 {
                            sin6_family: WinSock::AF_INET6,
                            sin6_port: 0,
                            sin6_flowinfo: 0,
                            sin6_addr: WinSock::IN6_ADDR {
                                u: WinSock::IN6_ADDR_0 {
                                    Byte: std::mem::transmute(target),
                                },
                            },
                            Anonymous: match self.builder.scope_id_option {
                                None => Default::default(),
                                Some(id) => WinSock::SOCKADDR_IN6_0 {
                                    sin6_scope_id: id,
                                }
                            },
                        },
                        request_data.to_be_bytes().as_ptr() as *mut _,
                        size_of_val(&request_data) as _,
                        None,
                        reply_buffer.as_ptr() as *mut _,
                        reply_buffer.len() as _,
                        self.builder.timeout,
                    )
                }
            };

            IpHelper::IcmpCloseHandle(handler)
                .map_err(|e| WindowsError::IcmpCloseFileError(e.message()))?;

            if reply_count != 0 {
                Ok(std::time::Instant::now().duration_since(start_time))
            } else {
                let error = GetLastError();
                Err(solve_recv_error(error))
            }
        }
    }
}

fn solve_recv_error(error: WIN32_ERROR) -> PingError {
    match error {
        WIN32_ERROR(11010) => SharedError::Timeout.into(),
        windows::Win32::Foundation::ERROR_NETWORK_UNREACHABLE => SharedError::Unreachable.into(),
        windows::Win32::Foundation::ERROR_INVALID_PARAMETER => {
            WindowsError::InvalidParameter.into()
        }
        WIN32_ERROR(_) => WindowsError::UnknownError(error.0).into(),
    }
}

impl Into<PingV4> for PingV4Builder {
    #[inline]
    fn into(self) -> PingV4 {
        PingV4 { builder: self }
    }
}

impl Into<PingV6> for PingV6Builder {
    #[inline]
    fn into(self) -> PingV6 {
        PingV6 { builder: self }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::builder::{PingV4Builder, PingV6Builder};
    use crate::base::windows::{PingV4, PingV6};

    #[test]
    fn test_ping_v4() {
        let ping: PingV4 = PingV4Builder::default().into();
        println!(
            "{} ms",
            ping.ping(std::net::Ipv4Addr::new(1, 1, 1, 1))
                .expect("ping_v4 error")
                .as_micros() as f64
                / 1000.0
        );
    }

    #[test]
    fn test_ping_v6() {
        let ping: PingV6 = PingV6Builder::default().into();
        println!(
            "{} ms",
            ping.ping("2408:8756:c52:1aec:0:ff:b013:5a11".parse().unwrap())
                .expect("ping_v6 error")
                .as_micros() as f64
                / 1000.0
        );
    }
}
