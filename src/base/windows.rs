use crate::base::builder::{PingV4Builder, PingV6Builder};
use crate::base::error::{PingError, SharedError};
use crate::base::utils::un_mut::UnMut;
use crate::{PingV4Result, PingV6Result};
use rand::Rng;
use std::ptr::null_mut;
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper;
#[cfg(target_pointer_width = "64")]
use windows::Win32::NetworkManagement::IpHelper::IP_OPTION_INFORMATION;
#[cfg(target_pointer_width = "32")]
use windows::Win32::NetworkManagement::IpHelper::IP_OPTION_INFORMATION32;
use windows::Win32::Networking::WinSock;

pub enum WindowsError {
    IcmpCreateFileError(String),
    IcmpCloseFileError(String),
    IcmpParseRepliesError(u32),
    InvalidParameter, //maybe reply_buffer too small
    UnknownError(u32),
}

pub struct PingV4 {
    builder: PingV4Builder,
    #[cfg(target_pointer_width = "32")]
    info: Option<UnMut<IP_OPTION_INFORMATION32>>,
    #[cfg(target_pointer_width = "64")]
    info: Option<UnMut<IP_OPTION_INFORMATION>>,
}

pub struct PingV6 {
    builder: PingV6Builder,
    #[cfg(target_pointer_width = "32")]
    info: Option<UnMut<IP_OPTION_INFORMATION32>>,
    #[cfg(target_pointer_width = "64")]
    info: Option<UnMut<IP_OPTION_INFORMATION>>,
}

impl PingV4 {
    #[inline]
    pub fn new(builder: PingV4Builder) -> PingV4 {
        match builder.ttl {
            Some(ttl) => PingV4 {
                builder,
                #[cfg(target_pointer_width = "32")]
                info: Some(UnMut::new(IP_OPTION_INFORMATION32 {
                    Ttl: ttl,
                    Tos: 0,
                    Flags: 0,
                    OptionsSize: 0,
                    OptionsData: null_mut(),
                })),
                #[cfg(target_pointer_width = "64")]
                info: Some(UnMut::new(IP_OPTION_INFORMATION {
                    Ttl: ttl,
                    Tos: 0,
                    Flags: 0,
                    OptionsSize: 0,
                    OptionsData: null_mut(),
                })),
            },
            None => PingV4 {
                builder,
                info: None,
            },
        }
    }

    const REPLY_BUFFER_SIZE: usize = size_of::<IpHelper::ICMP_ECHO_REPLY>() + size_of::<u128>() + 8;

    fn get_reply(
        &self,
        target: std::net::Ipv4Addr,
        buf: &mut [u8; Self::REPLY_BUFFER_SIZE],
    ) -> Result<std::time::Duration, PingError> {
        unsafe {
            let handler: windows::Win32::Foundation::HANDLE = match IpHelper::IcmpCreateFile() {
                Ok(v) => v,
                Err(e) => return Err(WindowsError::IcmpCreateFileError(e.message()).into()),
            };
            let des = target.to_bits();
            let request_data: u128 = rand::rng().random();
            let start_time = std::time::Instant::now();

            let request_options = match &self.info {
                None => None,
                Some(info) => Some(info.as_const_ref()),
            };

            let reply_count = match &self.builder.window_addition {
                // window_addition 确定第2-4项参数
                None => match self.builder.bind_addr {
                    // bind_addr确定调用方法是否有原地址
                    None => IpHelper::IcmpSendEcho2(
                        handler,
                        None,
                        None,
                        None,
                        des,
                        request_data.to_be_bytes().as_ptr() as *mut _,
                        size_of_val(&request_data) as _,
                        request_options,
                        buf.as_ptr() as *mut _,
                        buf.len() as _,
                        self.builder.timeout,
                    ),
                    Some(addr) => IpHelper::IcmpSendEcho2Ex(
                        handler,
                        None,
                        None,
                        None,
                        addr.to_bits(),
                        des,
                        request_data.to_be_bytes().as_ptr() as *mut _,
                        size_of_val(&request_data) as _,
                        request_options,
                        buf.as_ptr() as *mut _,
                        buf.len() as _,
                        self.builder.timeout,
                    ),
                },
                Some(addition) => match self.builder.bind_addr {
                    None => IpHelper::IcmpSendEcho2(
                        handler,
                        addition.event,
                        addition.apc_routine,
                        addition.apc_context,
                        des,
                        request_data.to_be_bytes().as_ptr() as *mut _,
                        size_of_val(&request_data) as _,
                        request_options,
                        buf.as_ptr() as *mut _,
                        buf.len() as _,
                        self.builder.timeout,
                    ),
                    Some(addr) => IpHelper::IcmpSendEcho2Ex(
                        handler,
                        addition.event,
                        addition.apc_routine,
                        addition.apc_context,
                        addr.to_bits(),
                        des,
                        request_data.to_be_bytes().as_ptr() as *mut _,
                        size_of_val(&request_data) as _,
                        request_options,
                        buf.as_ptr() as *mut _,
                        buf.len() as _,
                        self.builder.timeout,
                    ),
                },
            };

            if reply_count == 0 {
                let error = GetLastError();
                IpHelper::IcmpCloseHandle(handler)
                    .map_err(|e| WindowsError::IcmpCloseFileError(e.message()))?;
                return Err(solve_recv_error(error));
            }

            let reply_time = std::time::Instant::now().duration_since(start_time);

            let parse_error = IpHelper::IcmpParseReplies(buf.as_ptr() as *mut _, reply_count);

            if parse_error == 0 {
                IpHelper::IcmpCloseHandle(handler)
                    .map_err(|e| WindowsError::IcmpCloseFileError(e.message()))?;
                Ok(reply_time)
            } else {
                let error = GetLastError();
                IpHelper::IcmpCloseHandle(handler)
                    .map_err(|e| WindowsError::IcmpCloseFileError(e.message()))?;
                Err(WindowsError::IcmpParseRepliesError(error.0).into())
            }
        }
    }

    #[inline]
    pub fn ping(&self, target: std::net::Ipv4Addr) -> Result<std::time::Duration, PingError> {
        let mut buf = [0u8; Self::REPLY_BUFFER_SIZE];
        self.get_reply(target, &mut buf)
    }
    #[inline]
    pub fn ping_in_detail(&self, target: std::net::Ipv4Addr) -> Result<PingV4Result, PingError> {
        let mut buf = [0u8; Self::REPLY_BUFFER_SIZE];
        let duration = self.get_reply(target, &mut buf)?;
        Ok(PingV4Result {
            ip: std::net::Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]),
            duration,
        })
    }
}

impl PingV6 {
    #[inline]
    pub fn new(builder: PingV6Builder) -> PingV6 {
        match builder.ttl {
            Some(ttl) => PingV6 {
                builder,
                #[cfg(target_pointer_width = "32")]
                info: Some(UnMut::new(IP_OPTION_INFORMATION32 {
                    Ttl: ttl,
                    Tos: 0,
                    Flags: 0,
                    OptionsSize: 0,
                    OptionsData: null_mut(),
                })),
                #[cfg(target_pointer_width = "64")]
                info: Some(UnMut::new(IP_OPTION_INFORMATION {
                    Ttl: ttl,
                    Tos: 0,
                    Flags: 0,
                    OptionsSize: 0,
                    OptionsData: null_mut(),
                })),
            },
            None => PingV6 {
                builder,
                info: None,
            },
        }
    }

    const REPLY_BUFFER_SIZE: usize =
        size_of::<IpHelper::ICMPV6_ECHO_REPLY_LH>() + size_of::<u128>() + 8;

    fn get_reply(
        &self,
        target: std::net::Ipv6Addr,
        buf: &mut [u8; Self::REPLY_BUFFER_SIZE],
    ) -> Result<std::time::Duration, PingError> {
        unsafe {
            let handler: windows::Win32::Foundation::HANDLE = match IpHelper::Icmp6CreateFile() {
                Ok(v) => v,
                Err(e) => return Err(WindowsError::IcmpCreateFileError(e.message()).into()),
            };
            let request_data: u128 = rand::rng().random();
            let start_time = std::time::Instant::now();

            let request_options = match &self.info {
                None => None,
                Some(info) => Some(info.as_const_ref()),
            };

            let bind_addr = match self.builder.bind_addr {
                None => std::mem::zeroed(),
                Some(addr) => std::mem::transmute(addr),
            };

            let reply_count = match &self.builder.window_addition {
                None => IpHelper::Icmp6SendEcho2(
                    handler,
                    None,
                    None,
                    None,
                    &WinSock::SOCKADDR_IN6 {
                        sin6_family: WinSock::AF_INET6,
                        sin6_port: 0,
                        sin6_flowinfo: 0,
                        sin6_addr: WinSock::IN6_ADDR {
                            u: WinSock::IN6_ADDR_0 { Byte: bind_addr },
                        },
                        Anonymous: match self.builder.scope_id_option {
                            None => Default::default(),
                            Some(id) => WinSock::SOCKADDR_IN6_0 { sin6_scope_id: id },
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
                            Some(id) => WinSock::SOCKADDR_IN6_0 { sin6_scope_id: id },
                        },
                    },
                    request_data.to_be_bytes().as_ptr() as *mut _,
                    size_of_val(&request_data) as _,
                    request_options,
                    buf.as_ptr() as *mut _,
                    buf.len() as _,
                    self.builder.timeout,
                ),
                Some(addition) => IpHelper::Icmp6SendEcho2(
                    handler,
                    addition.event,
                    addition.apc_routine,
                    addition.apc_context,
                    &WinSock::SOCKADDR_IN6 {
                        sin6_family: WinSock::AF_INET6,
                        sin6_port: 0,
                        sin6_flowinfo: 0,
                        sin6_addr: WinSock::IN6_ADDR {
                            u: WinSock::IN6_ADDR_0 { Byte: bind_addr },
                        },
                        Anonymous: match self.builder.scope_id_option {
                            None => Default::default(),
                            Some(id) => WinSock::SOCKADDR_IN6_0 { sin6_scope_id: id },
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
                            Some(id) => WinSock::SOCKADDR_IN6_0 { sin6_scope_id: id },
                        },
                    },
                    request_data.to_be_bytes().as_ptr() as *mut _,
                    size_of_val(&request_data) as _,
                    request_options,
                    buf.as_ptr() as *mut _,
                    buf.len() as _,
                    self.builder.timeout,
                ),
            };

            if reply_count == 0 {
                let error = GetLastError();
                IpHelper::IcmpCloseHandle(handler)
                    .map_err(|e| WindowsError::IcmpCloseFileError(e.message()))?;
                return Err(solve_recv_error(error));
            }

            let reply_time = std::time::Instant::now().duration_since(start_time);

            let parse_error = IpHelper::Icmp6ParseReplies(buf.as_ptr() as *mut _, reply_count);

            if parse_error == 1 {
                IpHelper::IcmpCloseHandle(handler)
                    .map_err(|e| WindowsError::IcmpCloseFileError(e.message()))?;
                Ok(reply_time)
            } else {
                let error = GetLastError();
                IpHelper::IcmpCloseHandle(handler)
                    .map_err(|e| WindowsError::IcmpCloseFileError(e.message()))?;
                Err(WindowsError::IcmpParseRepliesError(error.0).into())
            }
        }
    }

    #[inline]
    pub fn ping(&self, target: std::net::Ipv6Addr) -> Result<std::time::Duration, PingError> {
        let mut buf = [0u8; Self::REPLY_BUFFER_SIZE];
        self.get_reply(target, &mut buf)
    }

    #[inline]
    pub fn ping_in_detail(&self, target: std::net::Ipv6Addr) -> Result<PingV6Result, PingError> {
        let mut buf = [0u8; Self::REPLY_BUFFER_SIZE];
        let duration = self.get_reply(target, &mut buf)?;
        Ok(PingV6Result {
            ip: std::net::Ipv6Addr::from(<[u8; 16]>::try_from(&buf[6..22]).unwrap()),
            duration,
        })
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
        PingV4::new(self)
    }
}

impl Into<PingV6> for PingV6Builder {
    #[inline]
    fn into(self) -> PingV6 {
        PingV6::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::base::builder::{PingV4Builder, PingV6Builder};

    #[test]
    fn test_ping_v4() {
        let ping = PingV4Builder {
            timeout: 200,
            ttl: Some(50),
            bind_addr: None,
            window_addition: None,
        }
        .build();
        println!(
            "{} ms",
            ping.ping(std::net::Ipv4Addr::new(1, 1, 1, 1))
                .expect("ping_v4 error")
                .as_micros() as f64
                / 1000.0
        );
    }

    #[test]
    fn test_ping_in_detail() {
        let ping = PingV4Builder {
            timeout: 200,
            ttl: Some(5),
            bind_addr: None,
            window_addition: None,
        }
        .build();
        let result = ping
            .ping_in_detail(std::net::Ipv4Addr::new(1, 1, 1, 1))
            .expect("ping_v4_in_detail error");
        println!(
            "{},{}",
            result.ip,
            result.duration.as_micros() as f64 / 1000.0
        );
    }

    #[test]
    fn test_ping_v6() {
        let ping = PingV6Builder {
            timeout: 150,
            ttl: None,
            bind_addr: None,
            scope_id_option: None,
            window_addition: None,
        }
        .build();
        println!(
            "{} ms",
            ping.ping("2408:8756:c52:1aec:0:ff:b013:5a11".parse().unwrap())
                .expect("ping_v6 error")
                .as_micros() as f64
                / 1000.0
        );
    }

    #[test]
    fn test_ping_v6_in_detail() {
        let ping = PingV6Builder {
            timeout: 200,
            ttl: Some(100),
            bind_addr: None,
            scope_id_option: None,
            window_addition: None,
        }
        .build();
        let result = ping
            .ping_in_detail("2408:8756:c52:1aec:0:ff:b013:5a11".parse().unwrap())
            .expect("ping_v6_in_detail error");
        println!(
            "{},{}",
            result.ip,
            result.duration.as_micros() as f64 / 1000.0
        );
    }
}
