use crate::base::error::{PingError, SharedError};
use rand::Rng;
use rustix::net;

pub struct SinglePing {
    timeout: u32, //ms
}

pub enum LinuxError {
    SocketSetupFailed(String),
    SetSockOptError(String),
    ConnectFailed(String),
    SendFailed(String),
    RecvFailed(String),
}

impl SinglePing {
    #[inline]
    pub fn new(timeout: u32) -> SinglePing {
        SinglePing { timeout }
    }

    pub fn ping_v4(&self, addr: net::Ipv4Addr) -> Result<std::time::Duration, PingError> {
        let sock = net::socket(
            net::AddressFamily::INET,
            net::SocketType::DGRAM,
            Some(net::ipproto::ICMP),
        )
        .map_err(|e| LinuxError::SocketSetupFailed(e.to_string()))?;

        net::sockopt::set_socket_timeout(
            &sock,
            net::sockopt::Timeout::Recv,
            Some(std::time::Duration::from_millis(self.timeout.into())),
        )
        .map_err(|e| LinuxError::SetSockOptError(e.to_string()))?;

        net::connect_v4(&sock, &net::SocketAddrV4::new(addr, 0))
            .map_err(|e| LinuxError::ConnectFailed(e.to_string()))?;

        let start_time = std::time::Instant::now();
        
        net::send(&sock, PingICMP::new(8).as_slice(), net::SendFlags::empty())
            .map_err(|e| LinuxError::SendFailed(e.to_string()))?;
        
        let mut buff = [0_u8; 20];
        net::recv(&sock, &mut buff, net::RecvFlags::empty())
            .map_err(|e| solve_recv_error(e))?;

        Ok(std::time::Instant::now().duration_since(start_time))
    }

    pub fn ping_v6(&self, addr: net::Ipv6Addr) -> Result<std::time::Duration, PingError> {
        let sock = net::socket(
            net::AddressFamily::INET6,
            net::SocketType::DGRAM,
            Some(net::ipproto::ICMPV6),
        )
            .map_err(|e| LinuxError::SocketSetupFailed(e.to_string()))?;

        net::sockopt::set_socket_timeout(
            &sock,
            net::sockopt::Timeout::Recv,
            Some(std::time::Duration::from_millis(self.timeout.into())),
        )
            .map_err(|e| LinuxError::SetSockOptError(e.to_string()))?;

        net::connect_v6(&sock, &net::SocketAddrV6::new(addr, 0, 0, 0))
            .map_err(|e| LinuxError::ConnectFailed(e.to_string()))?;

        let start_time = std::time::Instant::now();

        net::send(&sock, PingICMP::new(128).as_slice(), net::SendFlags::empty())
            .map_err(|e| LinuxError::SendFailed(e.to_string()))?;

        let mut buff = [0_u8; 20];
        net::recv(&sock, &mut buff, net::RecvFlags::empty())
            .map_err(|e| solve_recv_error(e))?;

        Ok(std::time::Instant::now().duration_since(start_time))
    }
}

fn solve_recv_error(error: rustix::io::Errno) -> PingError {
    match error.to_owned().raw_os_error() {
        11 => SharedError::Timeout.into(),
        _ => LinuxError::RecvFailed(error.to_string()).into(),
    }
}

impl Default for SinglePing {
    fn default() -> SinglePing {
        SinglePing { timeout: 1000 }
    }
}

struct PingICMP {
    data: [u8; 20],
}

impl PingICMP {
    fn new(icmp_type: u8) -> Self {
        let request_data: u128 = rand::rng().random();

        let mut data = [0_u8; 20];
        data[0] = icmp_type;
        data[1] = 0;
        data[4..].copy_from_slice(&request_data.to_be_bytes());

        const DATA_LEN: usize = 20;

        let mut sum: u32 = 0;
        let mut i = 0;
        while i < DATA_LEN {
            // 取出每两个字节，拼接成16位
            let word = if i + 1 < DATA_LEN {
                // 如果有两个字节，拼接成一个16位字
                ((data[i] as u16) << 8) | (data[i + 1] as u16)
            } else {
                // 如果只剩一个字节，拼接成一个16位字，低8位为0
                (data[i] as u16) << 8
            };

            // 累加到sum中
            sum += word as u32;

            // 如果有溢出，进位加回
            if sum > 0xFFFF {
                sum = (sum & 0xFFFF) + 1;
            }

            i += 2;
        }
        data[2..4].copy_from_slice(&(!(sum as u16)).to_be_bytes());

        PingICMP { data }
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::base::linux::SinglePing;

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
