use crate::base::linux::LinuxError;

pub enum PingError {
    SharedError(SharedError),
    #[cfg(target_os = "windows")]
    WindowsError,
    #[cfg(not(target_os = "windows"))]
    LinuxError(LinuxError),
}

impl std::fmt::Debug for PingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PingError::SharedError(e) => {
                match e {
                    SharedError::Timeout => {
                        write!(f, "SharedError::Timeout")
                    }
                    SharedError::Unreachable => {
                        write!(f, "SharedError::Unreachable")
                    }
                }
            }
            #[cfg(target_os = "windows")]
            PingError::WindowsError => {}
            #[cfg(not(target_os = "windows"))]
            PingError::LinuxError(e) => {
                match e {
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
                    LinuxError::RecvFailed(str) => {
                        write!(f, "PingError::LinuxError(RecvFailed): {}", str)
                    }
                }
            }
        }
    }
}

impl std::fmt::Display for PingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PingError::SharedError(e) => {
                match e {
                    SharedError::Timeout => {
                        write!(f, "Ping timeout")
                    }
                    SharedError::Unreachable => {
                        write!(f, "Ping unreachable")
                    }
                }
            }
            #[cfg(target_os = "windows")]
            PingError::WindowsError => {}
            #[cfg(not(target_os = "windows"))]
            PingError::LinuxError(e) => {
                match e {
                    LinuxError::SocketSetupFailed(str) => {
                        write!(f, "failed to setup socket: {}", str)
                    }
                    LinuxError::SetSockOptError(str) => {
                        write!(f, "failed to set socket option: {}", str)
                    }
                    LinuxError::ConnectFailed(str) => {
                        write!(f, "failed to connect to socket: {}", str)
                    }
                    LinuxError::SendFailed(str) => {
                        write!(f, "failed to send message to socket: {}", str)
                    }
                    LinuxError::RecvFailed(str) => {
                        write!(f, "failed to receive message from socket: {}", str)
                    }
                }
            }
        }
    }
}

pub enum SharedError {
    Timeout,
    Unreachable,
}

impl From<SharedError> for PingError {
    fn from(error: SharedError) -> Self {
        Self::SharedError(error)
    }
}

impl From<LinuxError> for PingError {
    fn from(error: LinuxError) -> Self {
        Self::LinuxError(error)
    }
}
