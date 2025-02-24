use crate::base::builder::{PingV4Builder, PingV6Builder};
use crate::base::error::{PingError, SharedError};
use crate::base::protocol::{IcmpDataForPing, IcmpFormat, Ipv4Header};
use crate::base::utils::SliceReader;
use crate::{PingV4Result, PingV6Result};
pub struct PingV4 {
    builder: PingV4Builder,
}

pub struct PingV6 {
    builder: PingV6Builder,
}

pub enum LinuxError {
    SocketSetupFailed(libc::c_int),
    SetSockOptError(libc::c_int),

    ConnectFailed(libc::c_int),
    SendFailed(libc::c_int),

    SendMessageFailed(libc::c_int),
    RecvFailed(libc::c_int),

    MissRespondAddr,
}

impl PingV4 {
    #[inline]
    pub fn new(builder: PingV4Builder) -> Self {
        Self { builder }
    }

    fn get_reply(
        &self,
        target: std::net::Ipv4Addr,
    ) -> Result<(std::time::Duration, std::net::Ipv4Addr), PingError> {
        unsafe {
            let sock = libc::socket(libc::AF_INET, libc::SOCK_RAW, libc::IPPROTO_ICMP);
            if sock == -1 {
                return Err(LinuxError::SocketSetupFailed(PingError::get_errno()).into());
            }

            {
                let millis = self.builder.timeout;
                let timeval = libc::timeval {
                    tv_sec: (millis / 1000) as libc::time_t,
                    tv_usec: ((millis % 1000) * 1000) as libc::suseconds_t,
                };
                let err = libc::setsockopt(
                    sock,
                    libc::SOL_SOCKET,
                    libc::SO_RCVTIMEO_NEW,
                    &timeval as *const _ as *const libc::c_void,
                    size_of::<libc::timeval>() as libc::socklen_t,
                );
                if err == -1 {
                    return Err(LinuxError::SetSockOptError(PingError::get_errno()).into());
                }
            }

            {
                let sock_addr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: 0,
                    sin_addr: libc::in_addr { s_addr: 0 },
                    sin_zero: Default::default(),
                };

                let err = libc::bind(
                    sock,
                    &sock_addr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in>() as libc::socklen_t,
                );
                if err == -1 {
                    return Err(SharedError::BindError(PingError::errno_to_str(
                        PingError::get_errno(),
                    ))
                    .into());
                }
            }

            {
                match self.builder.ttl {
                    None => {}
                    Some(ttl) => {
                        let err = libc::setsockopt(
                            sock,
                            libc::SOL_IP,
                            libc::IP_TTL,
                            &ttl as *const _ as *const libc::c_void,
                            size_of::<u8>() as libc::socklen_t,
                        );
                        if err == -1 {
                            return Err(LinuxError::SetSockOptError(PingError::get_errno()).into());
                        }
                    }
                }
            }

            let sent = IcmpDataForPing::new_ping_v4();
            {
                let addr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: 0,
                    sin_addr: std::mem::transmute(target),
                    sin_zero: Default::default(),
                };
                let err = libc::sendto(
                    sock,
                    sent.get_inner().as_ptr() as *const _,
                    IcmpDataForPing::DATA_SIZE,
                    0,
                    &addr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in>() as libc::socklen_t,
                );
                if err == -1 {
                    return Err(LinuxError::SendMessageFailed(PingError::get_errno()).into());
                }
            }
            let start_time = std::time::Instant::now();

            let mut buff = [0_u8; 100];
            {
                let len = libc::recv(sock, buff.as_mut_ptr() as *mut _, 100, 0);
                let duration = std::time::Instant::now().duration_since(start_time);
                if len == -1 {
                    return Err(LinuxError::RecvFailed(PingError::get_errno()).into());
                }
                let mut reader = SliceReader::from_slice(buff.as_ref());
                match Ipv4Header::from_reader(&mut reader, len as u16).and_then(|header| {
                    let format = IcmpFormat::from_header_v4(&header)?;
                    format.check_is_correspond_v4(&sent)?;
                    Some(header)
                }) {
                    Some(header) => Ok((duration, header.get_source_address())),
                    None => Err(LinuxError::MissRespondAddr.into()),
                }
            }
        }
    }

    #[inline]
    pub fn ping(&self, target: std::net::Ipv4Addr) -> Result<std::time::Duration, PingError> {
        Ok(self.get_reply(target)?.0)
    }

    #[inline]
    pub fn ping_in_detail(&self, target: std::net::Ipv4Addr) -> Result<PingV4Result, PingError> {
        let res = self.get_reply(target)?;
        Ok(PingV4Result {
            ip: res.1,
            duration: res.0,
        })
    }
}

impl PingV6 {
    #[inline]
    pub fn new(builder: PingV6Builder) -> Self {
        Self { builder }
    }

    fn precondition(&self) -> Result<libc::c_int, PingError> {
        unsafe {
            let sock = libc::socket(libc::AF_INET6, libc::SOCK_RAW, libc::IPPROTO_ICMPV6);
            if sock == -1 {
                return Err(LinuxError::SocketSetupFailed(PingError::get_errno()).into());
            }

            {
                let millis = self.builder.timeout;
                let timeval = libc::timeval {
                    tv_sec: (millis / 1000) as libc::time_t,
                    tv_usec: ((millis % 1000) * 1000) as libc::suseconds_t,
                };
                let err = libc::setsockopt(
                    sock,
                    libc::SOL_SOCKET,
                    libc::SO_RCVTIMEO_NEW,
                    &timeval as *const _ as *const libc::c_void,
                    size_of::<libc::timeval>() as libc::socklen_t,
                );
                if err == -1 {
                    return Err(LinuxError::SetSockOptError(PingError::get_errno()).into());
                }
            }

            {
                let sock_addr = libc::sockaddr_in6 {
                    sin6_family: libc::AF_INET6 as u16,
                    sin6_port: 0,
                    sin6_flowinfo: 0,
                    sin6_addr: libc::in6_addr {
                        s6_addr: Default::default(),
                    },
                    sin6_scope_id: self.builder.scope_id_option.unwrap_or(0),
                };

                let err = libc::bind(
                    sock,
                    &sock_addr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                );
                if err == -1 {
                    Err(
                        SharedError::BindError(PingError::errno_to_str(PingError::get_errno()))
                            .into(),
                    )
                } else {
                    Ok(sock)
                }
            }
        }
    }

    #[inline]
    pub fn ping(&self, target: std::net::Ipv6Addr) -> Result<std::time::Duration, PingError> {
        let sock = self.precondition()?;

        unsafe {
            {
                let addr = libc::sockaddr_in6 {
                    sin6_family: libc::AF_INET6 as u16,
                    sin6_port: 0,
                    sin6_flowinfo: 0,
                    sin6_addr: std::mem::transmute(target),
                    sin6_scope_id: self.builder.scope_id_option.unwrap_or(0),
                };
                let err = libc::connect(
                    sock,
                    &addr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                );
                if err == -1 {
                    return Err(LinuxError::ConnectFailed(PingError::get_errno()).into());
                }
            }

            let sent = IcmpDataForPing::new_ping_v6().into_inner();
            {
                let err = libc::send(
                    sock,
                    sent.as_ptr() as *const _,
                    IcmpDataForPing::DATA_SIZE,
                    0,
                );
                if err == -1 {
                    return Err(LinuxError::SendFailed(PingError::get_errno()).into());
                }
            }
            let start_time = std::time::Instant::now();

            let mut buff = [0_u8; IcmpDataForPing::DATA_SIZE];
            {
                let len = libc::recv(
                    sock,
                    buff.as_mut_ptr() as *mut _,
                    IcmpDataForPing::DATA_SIZE,
                    0,
                );
                let duration = std::time::Instant::now().duration_since(start_time);
                if len == -1 {
                    return Err(LinuxError::RecvFailed(PingError::get_errno()).into());
                }
                Ok(duration)
            }
        }
    }

    #[inline]
    pub fn ping_in_detail(&self, target: std::net::Ipv6Addr) -> Result<PingV6Result, PingError> {
        let sock = self.precondition()?;

        unsafe {
            let mut buff = IcmpDataForPing::new_ping_v6().into_inner();
            {
                let mut addr_v6 = libc::sockaddr_in6 {
                    sin6_family: libc::AF_INET6 as u16,
                    sin6_port: 0,
                    sin6_flowinfo: 0,
                    sin6_addr: std::mem::transmute(target),
                    sin6_scope_id: self.builder.scope_id_option.unwrap_or(0),
                };
                let mut iovec = [libc::iovec {
                    iov_base: buff.as_mut_ptr() as *mut _,
                    iov_len: IcmpDataForPing::DATA_SIZE,
                }];

                let msg = match self.builder.ttl {
                    None => libc::msghdr {
                        msg_name: &mut addr_v6 as *mut _ as *mut _,
                        msg_namelen: size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                        msg_iov: &mut iovec as *mut _ as *mut _,
                        msg_iovlen: 1,
                        msg_control: std::ptr::null_mut(),
                        msg_controllen: 0,
                        msg_flags: 0,
                    },
                    Some(ttl) => {
                        let ttl = ttl as u32;
                        const CONTROL_BUFF_LEN: usize =
                            unsafe { libc::CMSG_SPACE(size_of::<u32>() as _) as usize };
                        let mut control_buff = [0_u8; CONTROL_BUFF_LEN];
                        let msghdr = libc::msghdr {
                            msg_name: &mut addr_v6 as *mut _ as *mut _,
                            msg_namelen: size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                            msg_iov: &mut iovec as *mut _ as *mut _,
                            msg_iovlen: 1,
                            msg_control: &mut control_buff as *mut _ as *mut _,
                            msg_controllen: CONTROL_BUFF_LEN,
                            msg_flags: 0,
                        };
                        let ttl_cmsghdr: volatile::VolatilePtr<'_, libc::cmsghdr> =
                            volatile::VolatilePtr::new(
                                std::ptr::NonNull::new(libc::CMSG_FIRSTHDR(&msghdr)).unwrap(),
                            ); // use VolatilePtr to avoid 

                        ttl_cmsghdr.update(|mut cmsg| {
                            cmsg.cmsg_level = libc::SOL_IPV6;
                            cmsg.cmsg_type = libc::IPV6_HOPLIMIT;
                            cmsg.cmsg_len = libc::CMSG_LEN(size_of::<u32>() as _) as libc::size_t;
                            cmsg
                        });
                        libc::CMSG_DATA(ttl_cmsghdr.as_raw_ptr().as_ptr())
                            .copy_from_nonoverlapping(
                                &ttl as *const _ as *const _,
                                size_of::<u32>(),
                            );
                        msghdr
                    }
                };

                let err = libc::sendmsg(sock, &msg as *const _ as *const _, 0);
                if err == -1 {
                    return Err(LinuxError::SendMessageFailed(PingError::get_errno()).into());
                }
            }
            let start_time = std::time::Instant::now();

            {
                let mut addr_v6 = std::mem::MaybeUninit::<libc::sockaddr_in6>::uninit();
                let mut iovec = [libc::iovec {
                    iov_base: buff.as_mut_ptr() as *mut _,
                    iov_len: IcmpDataForPing::DATA_SIZE,
                }];
                let mut msg = libc::msghdr {
                    msg_name: addr_v6.as_mut_ptr() as *mut _,
                    msg_namelen: size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                    msg_iov: &mut iovec as *mut _ as *mut _,
                    msg_iovlen: 1,
                    msg_control: std::ptr::null_mut(),
                    msg_controllen: 0,
                    msg_flags: 0,
                };
                let len = libc::recvmsg(sock, &mut msg as *mut _ as *mut _, 0);
                let duration = std::time::Instant::now().duration_since(start_time);
                if len == -1 {
                    println!("{:?}", *libc::__errno_location());
                    return Err(LinuxError::RecvFailed(PingError::get_errno()).into());
                }
                if msg.msg_namelen == 0 {
                    return Err(LinuxError::MissRespondAddr.into());
                }
                Ok(PingV6Result {
                    ip: std::net::Ipv6Addr::from(addr_v6.assume_init().sin6_addr.s6_addr),
                    duration,
                })
            }
        }
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
    use crate::{PingV4, PingV6};

    #[test]
    fn test_ping_v4() {
        let ping: PingV4 = PingV4Builder {
            timeout: 1000,
            ttl: Some(5),
            bind_addr: None,
        }
        .into();
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
            ttl: Some(10),
            bind_addr: None,
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
        let ping: PingV6 = PingV6Builder::default().into();
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
            ttl: Some(5),
            bind_addr: None,
            scope_id_option: None,
        }
        .build();
        let result = ping
            .ping_in_detail("2606:4700:4700::1111".parse().unwrap())
            .expect("ping_v6_in_detail error");
        println!(
            "{},{}",
            result.ip,
            result.duration.as_micros() as f64 / 1000.0
        );
    }
}
