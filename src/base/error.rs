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
                    write!(f, "SharedError::BindError({})", str)
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
                    write!(f, "PingError::LinuxError(SocketSetupFailed): {}", str)
                }
                LinuxError::SetSockOptError(str) => {
                    write!(f, "PingError::LinuxError(SetSockOptError): {}", str)
                }

                LinuxError::ConnectFailed(str) => {
                    write!(f, "PingError::LinuxError(ConnectFailed): {}", str)
                }
                LinuxError::SendFailed(str) => {
                    write!(f, "PingError::LinuxError(SendFailed): {}", str)
                }

                LinuxError::SendtoFailed(str) => {
                    write!(f, "PingError::LinuxError(SendFailed): {}", str)
                }
                LinuxError::RecvFailed(str) => {
                    write!(f, "PingError::LinuxError(RecvFailed): {}", str)
                }

                LinuxError::MissRespondAddr => {
                    write!(f, "PingError::LinuxError(MissRespondAddr)")
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
                    write!(f, "ping bind error: {}", str)
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
                    write!(f, "failed to setup socket: {}", str)
                }
                LinuxError::SetSockOptError(str) => {
                    write!(f, "failed to set socket option: {}", str)
                }

                LinuxError::ConnectFailed(str) => {
                    write!(f, "failed to connect socket: {}", str)
                }
                LinuxError::SendFailed(str) => {
                    write!(f, "failed to send message: {}", str)
                }

                LinuxError::SendtoFailed(str) => {
                    write!(f, "failed to send message to socket: {}", str)
                }
                LinuxError::RecvFailed(str) => {
                    write!(f, "failed to receive message from socket: {}", str)
                }

                LinuxError::MissRespondAddr => {
                    write!(f, "query target address failed")
                }
            },
        }
    }
}

pub enum SharedError {
    Timeout,
    Unreachable,
    BindError(String),
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
