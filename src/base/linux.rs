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
    SendtoFailed(libc::c_int),
    SendMessageFailed(libc::c_int),
    RecvFailed(libc::c_int),

    ResolveRecvFailed,
    MissRespondAddr,
    NullPtr,
}

impl PingV4 {
    #[inline]
    pub fn new(builder: PingV4Builder) -> Self {
        Self { builder }
    }

    fn precondition(&self) -> Result<libc::c_int, PingError> {
        let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_RAW, libc::IPPROTO_ICMP) };
        if sock == -1 {
            return Err(LinuxError::SocketSetupFailed(PingError::get_errno()).into());
        }

        {
            let millis = self.builder.timeout;
            let timeval = libc::timeval {
                tv_sec: (millis / 1000) as libc::time_t,
                tv_usec: ((millis % 1000) * 1000) as libc::suseconds_t,
            };
            let err = unsafe {
                libc::setsockopt(
                    sock,
                    libc::SOL_SOCKET,
                    libc::SO_RCVTIMEO_NEW,
                    &timeval as *const _ as *const libc::c_void,
                    size_of::<libc::timeval>() as libc::socklen_t,
                )
            };
            if err == -1 {
                return Err(LinuxError::SetSockOptError(PingError::get_errno()).into());
            }
        }

        match self.builder.bind_addr {
            None => {}
            Some(addr) => {
                let sock_addr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: 0,
                    sin_addr: libc::in_addr {
                        s_addr: addr.to_bits(),
                    },
                    sin_zero: Default::default(),
                };

                let err = unsafe {
                    libc::bind(
                        sock,
                        &sock_addr as *const _ as *const libc::sockaddr,
                        size_of::<libc::sockaddr_in>() as libc::socklen_t,
                    )
                };
                if err == -1 {
                    return Err(SharedError::BindError(PingError::errno_to_str(
                        PingError::get_errno(),
                    ))
                    .into());
                }
            }
        }

        match self.builder.ttl {
            None => Ok(sock),
            Some(ttl) => {
                let err = unsafe {
                    libc::setsockopt(
                        sock,
                        libc::SOL_IP,
                        libc::IP_TTL,
                        &ttl as *const _ as *const libc::c_void,
                        size_of::<u8>() as libc::socklen_t,
                    )
                };
                if err == -1 {
                    Err(LinuxError::SetSockOptError(PingError::get_errno()).into())
                } else {
                    Ok(sock)
                }
            }
        }
    }

    #[inline]
    pub fn ping(&self, target: std::net::Ipv4Addr) -> Result<std::time::Duration, PingError> {
        let sock = self.precondition()?;
        {
            let addr = libc::sockaddr_in {
                sin_family: libc::AF_INET as u16,
                sin_port: 0,
                sin_addr: unsafe { std::mem::transmute(target) },
                sin_zero: Default::default(),
            };
            let err = unsafe {
                libc::connect(
                    sock,
                    &addr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in>() as libc::socklen_t,
                )
            };
            if err == -1 {
                return Err(LinuxError::ConnectFailed(PingError::get_errno()).into());
            }
        }
        let sent = IcmpDataForPing::new_ping_v4();
        {
            let err = unsafe {
                libc::send(
                    sock,
                    sent.get_inner().as_ptr() as *const _,
                    IcmpDataForPing::DATA_SIZE,
                    0,
                )
            };
            if err == -1 {
                return Err(LinuxError::SendFailed(PingError::get_errno()).into());
            }
        }
        let start_time = std::time::Instant::now();

        let mut buff = [0_u8; IcmpDataForPing::DATA_SIZE];
        {
            let len = unsafe {
                libc::recv(
                    sock,
                    buff.as_mut_ptr() as *mut _,
                    IcmpDataForPing::DATA_SIZE,
                    0,
                )
            };
            let duration = std::time::Instant::now().duration_since(start_time);
            if len == -1 {
                return Err(LinuxError::RecvFailed(PingError::get_errno()).into());
            }
            let mut reader = SliceReader::from_slice(buff.as_ref());
            match Ipv4Header::from_reader(&mut reader, len as u16).and_then(|header| {
                let format = IcmpFormat::from_header_v4(&header)?;
                format.check_is_correspond_v4(&sent)?;
                Some(())
            }) {
                Some(_) => Ok(duration),
                None => Err(LinuxError::ResolveRecvFailed.into()),
            }
        }
    }

    #[inline]
    pub fn ping_in_detail(&self, target: std::net::Ipv4Addr) -> Result<PingV4Result, PingError> {
        let sock = self.precondition()?;
        let sent = IcmpDataForPing::new_ping_v4();
        {
            let addr = libc::sockaddr_in {
                sin_family: libc::AF_INET as u16,
                sin_port: 0,
                sin_addr: unsafe { std::mem::transmute(target) },
                sin_zero: Default::default(),
            };
            let err = unsafe {
                libc::sendto(
                    sock,
                    sent.get_inner().as_ptr() as *const _,
                    IcmpDataForPing::DATA_SIZE,
                    0,
                    &addr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in>() as libc::socklen_t,
                )
            };
            if err == -1 {
                return Err(LinuxError::SendtoFailed(PingError::get_errno()).into());
            }
        }
        let start_time = std::time::Instant::now();

        let mut buff = [0_u8; 100]; // this buff size should depend on recv ttl exceeded message size, or you want to use libc::recvmsg instead
        {
            let len = unsafe { libc::recv(sock, buff.as_mut_ptr() as *mut _, 100, 0) };
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
                Some(header) => Ok(PingV4Result {
                    ip: header.get_source_address(),
                    duration,
                }),
                None => Err(LinuxError::ResolveRecvFailed.into()),
            }
        }
    }
}

impl PingV6 {
    #[inline]
    pub fn new(builder: PingV6Builder) -> Self {
        Self { builder }
    }

    fn precondition(&self) -> Result<libc::c_int, PingError> {
        let sock = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_RAW, libc::IPPROTO_ICMPV6) };
        if sock == -1 {
            return Err(LinuxError::SocketSetupFailed(PingError::get_errno()).into());
        }

        {
            let millis = self.builder.timeout;
            let timeval = libc::timeval {
                tv_sec: (millis / 1000) as libc::time_t,
                tv_usec: ((millis % 1000) * 1000) as libc::suseconds_t,
            };
            let err = unsafe {
                libc::setsockopt(
                    sock,
                    libc::SOL_SOCKET,
                    libc::SO_RCVTIMEO_NEW,
                    &timeval as *const _ as *const libc::c_void,
                    size_of::<libc::timeval>() as libc::socklen_t,
                )
            };
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

            let err = unsafe {
                libc::bind(
                    sock,
                    &sock_addr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                )
            };
            if err == -1 {
                Err(SharedError::BindError(PingError::errno_to_str(PingError::get_errno())).into())
            } else {
                Ok(sock)
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

        let mut buff = IcmpDataForPing::new_ping_v6().into_inner();
        {
            let mut addr_v6 = libc::sockaddr_in6 {
                sin6_family: libc::AF_INET6 as u16,
                sin6_port: 0,
                sin6_flowinfo: 0,
                sin6_addr: unsafe { std::mem::transmute(target) },
                sin6_scope_id: self.builder.scope_id_option.unwrap_or(0),
            };

            match self.builder.ttl {
                // 没错, ipv6设置ttl(HopLimit)就是这么繁琐
                None => {
                    let err = unsafe {
                        libc::sendto(
                            sock,
                            buff.as_mut_ptr() as *mut _,
                            IcmpDataForPing::DATA_SIZE,
                            0,
                            &mut addr_v6 as *mut _ as *mut _,
                            size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                        )
                    };
                    if err == -1 {
                        return Err(LinuxError::SendtoFailed(PingError::get_errno()).into());
                    }
                }
                Some(ttl) => {
                    let ttl = ttl as TTL; // use u32, instead you will have to deal with problem in CMSG_LEN API
                    type TTL = u32;

                    let mut iovec = [libc::iovec {
                        iov_base: buff.as_mut_ptr() as *mut _,
                        iov_len: IcmpDataForPing::DATA_SIZE,
                    }];

                    const CONTROL_BUFF_LEN: usize =
                        unsafe { libc::CMSG_SPACE(size_of::<TTL>() as _) as usize };
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
                    let ttl_cmsghdr: volatile::VolatilePtr<libc::cmsghdr> = unsafe {
                        volatile::VolatilePtr::new(
                            std::ptr::NonNull::new(libc::CMSG_FIRSTHDR(&msghdr))
                                .ok_or(LinuxError::NullPtr)?,
                        ) // use VolatilePtr to avoid being optimized
                    };

                    ttl_cmsghdr.update(|mut cmsg| {
                        cmsg.cmsg_level = libc::SOL_IPV6;
                        cmsg.cmsg_type = libc::IPV6_HOPLIMIT;
                        cmsg.cmsg_len =
                            unsafe { libc::CMSG_LEN(size_of::<TTL>() as _) } as libc::size_t;
                        cmsg
                    });
                    let _ = unsafe {
                        volatile::VolatilePtr::new(
                            std::ptr::NonNull::new(libc::CMSG_DATA(
                                ttl_cmsghdr.as_raw_ptr().as_ptr(),
                            ))
                            .ok_or(LinuxError::NullPtr)?,
                        )
                        .map(|data_ptr| {
                            data_ptr.as_ptr().copy_from_nonoverlapping(
                                &ttl as *const _ as *const _,
                                size_of::<TTL>(),
                            );
                            data_ptr
                        })
                    };
                    let err = unsafe { libc::sendmsg(sock, &msghdr as *const _ as *const _, 0) };
                    if err == -1 {
                        return Err(LinuxError::SendMessageFailed(PingError::get_errno()).into());
                    }
                }
            };
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
            let len = unsafe { libc::recvmsg(sock, &mut msg as *mut _ as *mut _, 0) };
            let duration = std::time::Instant::now().duration_since(start_time);
            if len == -1 {
                return Err(LinuxError::RecvFailed(PingError::get_errno()).into());
            }
            if msg.msg_namelen == 0 {
                return Err(LinuxError::MissRespondAddr.into());
            }
            // todo check_is_correspond_v6
            Ok(PingV6Result {
                ip: std::net::Ipv6Addr::from(unsafe { addr_v6.assume_init() }.sin6_addr.s6_addr),
                duration,
            })
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
