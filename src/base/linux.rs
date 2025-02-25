use crate::base::builder::{PingV4Builder, PingV6Builder};
use crate::base::error::{PingError, SharedError};
use crate::base::protocol::{IcmpDataForPing, IcmpFormat, Ipv4Header};
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

    BindFailed(libc::c_int),
    ConnectFailed(libc::c_int),
    SendFailed(libc::c_int),
    SendtoFailed(libc::c_int),
    SendMessageFailed(libc::c_int),
    RecvFailed(libc::c_int),

    ResolveRecvFailed,
    MissRespondAddr,
    NullPtr,
}

impl LinuxError {
    #[inline]
    fn convert_recv_failed(input: libc::c_int) -> PingError {
        match input {
            101 => PingError::SharedError(SharedError::Unreachable),
            11 => PingError::SharedError(SharedError::Timeout),
            _ => PingError::LinuxError(LinuxError::RecvFailed(input)),
        }
    }

    #[inline]
    fn convert_setup_failed(input: libc::c_int) -> PingError {
        match input {
            1 => PingError::SharedError(SharedError::NoElevatedPrivilege),
            _ => PingError::LinuxError(LinuxError::BindFailed(input)),
        }
    }

    #[inline]
    pub fn get_errno() -> libc::c_int {
        unsafe { *libc::__errno_location() }
    }

    pub fn errno_to_str(err: libc::c_int) -> Option<String> {
        unsafe {
            let mut ptr = libc::strerror(err) as *const u8;
            let mut offset = 0;
            let mut str = String::with_capacity(55);
            while (*ptr) != 0 {
                if offset > 55 || !(*ptr).is_ascii() {
                    return None;
                } else {
                    str.push(*ptr as char);
                    offset += 1;
                    ptr = ptr.wrapping_add(1);
                }
            }
            Some(str)
        }
    }
}

mod common {
    use super::*;

    #[inline]
    pub(super) fn set_timeout(sock: libc::c_int, millis: u32) -> Result<(), PingError> {
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
            return Err(LinuxError::SetSockOptError(LinuxError::get_errno()).into());
        }
        Ok(())
    }

    #[inline]
    pub(super) fn send(sock: libc::c_int, sent: &IcmpDataForPing) -> Result<(), PingError> {
        let err = unsafe {
            libc::send(
                sock,
                sent.get_inner().as_ptr() as *const _,
                IcmpDataForPing::DATA_SIZE,
                0,
            )
        };
        if err == -1 {
            return Err(LinuxError::SendFailed(LinuxError::get_errno()).into());
        }
        Ok(())
    }

    #[inline]
    pub(super) fn get_addr_v6(
        target: std::net::Ipv6Addr,
        sin6_scope_id: u32,
    ) -> libc::sockaddr_in6 {
        libc::sockaddr_in6 {
            sin6_family: libc::AF_INET6 as u16,
            sin6_port: 0,
            sin6_flowinfo: 0,
            sin6_addr: unsafe { std::mem::transmute(target) },
            sin6_scope_id,
        }
    }
}

impl PingV4 {
    #[inline]
    pub fn new(builder: PingV4Builder) -> Self {
        Self { builder }
    }

    fn precondition(&self) -> Result<libc::c_int, PingError> {
        #[cfg(not(feature = "DGRAM_SOCKET"))]
        let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_RAW, libc::IPPROTO_ICMP) };
        #[cfg(feature = "DGRAM_SOCKET")]
        let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_ICMP) };
        if sock == -1 {
            return Err(LinuxError::convert_setup_failed(LinuxError::get_errno()).into());
        }

        common::set_timeout(sock, self.builder.timeout)?;

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
                    return Err(LinuxError::BindFailed(LinuxError::get_errno()).into());
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
                    Err(LinuxError::SetSockOptError(LinuxError::get_errno()).into())
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
                return Err(LinuxError::ConnectFailed(LinuxError::get_errno()).into());
            }
        }
        let sent = IcmpDataForPing::new_ping_v4();
        common::send(sock, &sent)?;
        let start_time = std::time::Instant::now();

        let mut buff = std::mem::MaybeUninit::<[u8; IcmpDataForPing::DATA_SIZE]>::uninit();
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
                return Err(LinuxError::convert_recv_failed(LinuxError::get_errno()));
            }
            Ipv4Header::from_slice(&unsafe { buff.assume_init_ref() }[..len as usize])
                .and_then(|header| {
                    let format = IcmpFormat::from_header_v4(&header)?;
                    format.check_is_correspond_v4(&sent)?;
                    Some(duration)
                })
                .ok_or(LinuxError::ResolveRecvFailed.into())
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
                return Err(LinuxError::SendtoFailed(LinuxError::get_errno()).into());
            }
        }
        let start_time = std::time::Instant::now();

        const SIZE_OF_BUFF: usize = IcmpDataForPing::DATA_SIZE + 38;
        let mut buff = std::mem::MaybeUninit::<[u8; SIZE_OF_BUFF]>::uninit();
        // this buff size should depend on recv ttl exceeded message size, or you want to use libc::recvmsg instead
        // let us calculate the buff size: 4 as ICMP header fix size + 4 as unused size in ICMP data + 20 as Ipv4Header fix size + IcmpDataForPing::DATA_SIZE + 10 as Safety
        //          = IcmpDataForPing::DATA_SIZE + 38
        {
            let len = unsafe { libc::recv(sock, buff.as_mut_ptr() as *mut _, SIZE_OF_BUFF, 0) };
            let duration = std::time::Instant::now().duration_since(start_time);
            if len == -1 {
                return Err(LinuxError::convert_recv_failed(LinuxError::get_errno()));
            }
            Ipv4Header::from_slice(&unsafe { buff.assume_init_ref() }[..len as usize])
                .and_then(|header| {
                    let format = IcmpFormat::from_header_v4(&header)?;
                    format.check_is_correspond_v4(&sent)?;
                    Some(PingV4Result {
                        ip: header.get_source_address(),
                        duration,
                    })
                })
                .ok_or(LinuxError::ResolveRecvFailed.into())
        }
    }
}

impl PingV6 {
    #[inline]
    pub fn new(builder: PingV6Builder) -> Self {
        Self { builder }
    }

    // APIs are so different between Ipv4 socket and Ipv6 socket, so many codes are different

    fn precondition(&self) -> Result<libc::c_int, PingError> {
        #[cfg(not(feature = "DGRAM_SOCKET"))]
        let sock = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_RAW, libc::IPPROTO_ICMPV6) };
        #[cfg(feature = "DGRAM_SOCKET")]
        let sock = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, libc::IPPROTO_ICMPV6) };

        if sock == -1 {
            return Err(LinuxError::convert_setup_failed(LinuxError::get_errno()).into());
        }

        common::set_timeout(sock, self.builder.timeout)?;

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
                Err(LinuxError::BindFailed(LinuxError::get_errno()).into())
            } else {
                Ok(sock)
            }
        }
    }

    #[inline]
    pub fn ping(&self, target: std::net::Ipv6Addr) -> Result<std::time::Duration, PingError> {
        let sock = self.precondition()?;

        {
            let addr = common::get_addr_v6(target, self.builder.scope_id_option.unwrap_or(0));
            let err = unsafe {
                libc::connect(
                    sock,
                    &addr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                )
            };
            if err == -1 {
                return Err(LinuxError::ConnectFailed(LinuxError::get_errno()).into());
            }
        }

        let sent = IcmpDataForPing::new_ping_v6();
        common::send(sock, &sent)?;
        let start_time = std::time::Instant::now();

        let mut buff = std::mem::MaybeUninit::<[u8; IcmpDataForPing::DATA_SIZE]>::uninit();
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
                return Err(LinuxError::convert_recv_failed(LinuxError::get_errno()));
            }
            IcmpFormat::from_slice(unsafe { buff.assume_init_ref() })
                .and_then(|format| format.check_is_correspond_v6(&sent))
                .map(|_| duration)
                .ok_or(LinuxError::ResolveRecvFailed.into())
        }
    }

    #[inline]
    pub fn ping_in_detail(&self, target: std::net::Ipv6Addr) -> Result<PingV6Result, PingError> {
        let sock = self.precondition()?;

        let mut sent = IcmpDataForPing::new_ping_v6();
        {
            let mut addr_v6 =
                common::get_addr_v6(target, self.builder.scope_id_option.unwrap_or(0));

            match self.builder.ttl {
                // 没错, ipv6设置ttl(HopLimit)就是这么繁琐
                None => {
                    let err = unsafe {
                        libc::sendto(
                            sock,
                            sent.get_inner_mut().as_mut_ptr() as *mut _,
                            IcmpDataForPing::DATA_SIZE,
                            0,
                            &mut addr_v6 as *mut _ as *mut _,
                            size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                        )
                    };
                    if err == -1 {
                        return Err(LinuxError::SendtoFailed(LinuxError::get_errno()).into());
                    }
                }
                Some(ttl) => {
                    let ttl = ttl as TTL; // use u32, instead you will have to deal with problem in CMSG_LEN API
                    type TTL = u32;

                    let mut iovec = [libc::iovec {
                        iov_base: sent.get_inner_mut().as_mut_ptr() as *mut _,
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
                        return Err(LinuxError::SendMessageFailed(LinuxError::get_errno()).into());
                    }
                }
            };
        }
        let start_time = std::time::Instant::now();

        // let us calculate the buff size: 4 as ICMPv6 header fix size + 4 as unused size in ICMPv6 data + 40 as Ipv6Header fix size + IcmpDataForPing::DATA_SIZE + 10 as Safety
        //          = IcmpDataForPing::DATA_SIZE + 58
        const SIZE_OF_BUFF: usize = IcmpDataForPing::DATA_SIZE + 58;
        let mut buff: std::mem::MaybeUninit<[u8; SIZE_OF_BUFF]> = std::mem::MaybeUninit::uninit();
        {
            let mut addr_v6 = std::mem::MaybeUninit::<libc::sockaddr_in6>::uninit();
            let mut iovec = [libc::iovec {
                iov_base: buff.as_mut_ptr() as *mut _,
                iov_len: SIZE_OF_BUFF,
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

            // if you don't use recvmsg, you can't get source socketaddr
            let len = unsafe { libc::recvmsg(sock, &mut msg as *mut _ as *mut _, 0) };
            let duration = std::time::Instant::now().duration_since(start_time);
            if len == -1 {
                return Err(LinuxError::convert_recv_failed(LinuxError::get_errno()));
            }
            if msg.msg_namelen == 0 {
                return Err(LinuxError::MissRespondAddr.into());
            }
            let len = if len > SIZE_OF_BUFF as isize {
                SIZE_OF_BUFF
            } else {
                len as usize
            };
            IcmpFormat::from_slice(&unsafe { buff.assume_init_ref() }[..len])
                .and_then(|format| format.check_is_correspond_v6(&sent))
                .map(|_| PingV6Result {
                    ip: std::net::Ipv6Addr::from(
                        unsafe { addr_v6.assume_init() }.sin6_addr.s6_addr,
                    ),
                    duration,
                })
                .ok_or(LinuxError::ResolveRecvFailed.into())
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
