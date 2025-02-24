#[cfg(not(target_os = "windows"))]
use crate::base::linux::LinuxError;
#[cfg(target_os = "windows")]
use crate::base::windows::WindowsError;

pub enum PingError {
    SharedError(SharedError),
    #[cfg(target_os = "windows")]
    WindowsError(WindowsError),
    #[cfg(not(target_os = "windows"))]
    LinuxError(LinuxError),
}

impl std::fmt::Debug for PingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PingError::SharedError(e) => match e {
                SharedError::Timeout => {
                    write!(f, "SharedError::Timeout")
                }
                SharedError::Unreachable => {
                    write!(f, "SharedError::Unreachable")
                }
                SharedError::BindError(str) => {
                    write!(f, "SharedError::BindError({:?})", str)
                }
            },
            #[cfg(target_os = "windows")]
            PingError::WindowsError(e) => match e {
                WindowsError::IcmpCreateFileError(str) => {
                    write!(f, "PingError::WindowsError(IcmpCreateFileError): {}", str)
                }
                WindowsError::IcmpCloseFileError(str) => {
                    write!(f, "PingError::WindowsError(IcmpCloseFileError): {}", str)
                }
                WindowsError::IcmpParseRepliesError(u) => {
                    write!(f, "PingError::WindowsError(IcmpParseRepliesError): {}", u)
                }
                WindowsError::InvalidParameter => {
                    write!(f, "PingError::InvalidParameter")
                }
                WindowsError::UnknownError(i) => {
                    write!(f, "PingError::WindowsError(UnknownError({}))", i)
                }
            },
            #[cfg(not(target_os = "windows"))]
            PingError::LinuxError(e) => match e {
                LinuxError::SocketSetupFailed(str) => {
                    write!(
                        f,
                        "PingError::LinuxError(SocketSetupFailed): Errno({str}) {:?}",
                        Self::errno_to_str(*str)
                    )
                }
                LinuxError::SetSockOptError(str) => {
                    write!(
                        f,
                        "PingError::LinuxError(SetSockOptError): Errno({str}) {:?}",
                        Self::errno_to_str(*str)
                    )
                }

                LinuxError::ConnectFailed(str) => {
                    write!(
                        f,
                        "PingError::LinuxError(ConnectFailed): Errno({str}) {:?}",
                        Self::errno_to_str(*str)
                    )
                }
                LinuxError::SendFailed(str) => {
                    write!(
                        f,
                        "PingError::LinuxError(SendFailed): Errno({str}) {:?}",
                        Self::errno_to_str(*str)
                    )
                }

                LinuxError::SendMessageFailed(str) => {
                    write!(
                        f,
                        "PingError::LinuxError(SendMessageFailed): Errno({str}) {:?}",
                        Self::errno_to_str(*str)
                    )
                }
                LinuxError::RecvFailed(str) => {
                    write!(
                        f,
                        "PingError::LinuxError(RecvFailed): Errno({str}) {:?}",
                        Self::errno_to_str(*str)
                    )
                }

                LinuxError::MissRespondAddr => {
                    write!(f, "PingError::LinuxError(MissRespondAddr)")
                }
                LinuxError::NullPtr => {
                    write!(f, "PingError::LinuxError(NullPtr)")
                }
            },
        }
    }
}

impl std::fmt::Display for PingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PingError::SharedError(e) => match e {
                SharedError::Timeout => {
                    write!(f, "ping timeout")
                }
                SharedError::Unreachable => {
                    write!(f, "ping unreachable")
                }
                SharedError::BindError(str) => {
                    write!(f, "ping bind error: {:?}", str)
                }
            },
            #[cfg(target_os = "windows")]
            PingError::WindowsError(e) => match e {
                WindowsError::IcmpCreateFileError(str) => {
                    write!(f, "icmp create file error: {}", str)
                }
                WindowsError::IcmpCloseFileError(str) => {
                    write!(f, "icmp close file error: {}", str)
                }
                WindowsError::IcmpParseRepliesError(u) => {
                    write!(f, "icmp parse replies error: {}", u)
                }
                WindowsError::InvalidParameter => {
                    write!(f, "invalid parameter")
                }
                WindowsError::UnknownError(i) => {
                    write!(f, "Windows Unknown Error: {}", i)
                }
            },
            #[cfg(not(target_os = "windows"))]
            PingError::LinuxError(e) => match e {
                LinuxError::SocketSetupFailed(str) => {
                    write!(f, "failed to setup socket: {:?}", Self::errno_to_str(*str))
                }
                LinuxError::SetSockOptError(str) => {
                    write!(
                        f,
                        "failed to set socket option: {:?}",
                        Self::errno_to_str(*str)
                    )
                }

                LinuxError::ConnectFailed(str) => {
                    write!(
                        f,
                        "failed to connect socket: {:?}",
                        Self::errno_to_str(*str)
                    )
                }
                LinuxError::SendFailed(str) => {
                    write!(f, "failed to send message: {:?}", Self::errno_to_str(*str))
                }

                LinuxError::SendMessageFailed(str) => {
                    write!(
                        f,
                        "failed to send message to socket: {:?}",
                        Self::errno_to_str(*str)
                    )
                }
                LinuxError::RecvFailed(str) => {
                    write!(
                        f,
                        "failed to receive message from socket: {:?}",
                        Self::errno_to_str(*str)
                    )
                }

                LinuxError::MissRespondAddr => {
                    write!(f, "query target address failed")
                }
                LinuxError::NullPtr => {
                    write!(f, "query target null pointer")
                }
            },
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl PingError {
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

pub enum SharedError {
    Timeout,
    Unreachable,
    BindError(Option<String>),
}

impl From<SharedError> for PingError {
    fn from(error: SharedError) -> Self {
        Self::SharedError(error)
    }
}

#[cfg(not(target_os = "windows"))]
impl From<LinuxError> for PingError {
    fn from(error: LinuxError) -> Self {
        Self::LinuxError(error)
    }
}

#[cfg(target_os = "windows")]
impl From<WindowsError> for PingError {
    fn from(error: WindowsError) -> Self {
        Self::WindowsError(error)
    }
}
