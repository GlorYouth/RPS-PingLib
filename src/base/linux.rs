use rustix::net;
use std::net::Ipv4Addr;
use rand::Rng;

struct PingICMP {
    data: [u8;20]
}

impl PingICMP {
    fn new() -> Self {
        let request_data: u128 = rand::rng().random();
        
        let mut data = [0_u8;20];
        data[0] = 8;
        data[1] = 0;
        data[4..].copy_from_slice(&request_data.to_be_bytes());
        
        const DATA_LEN: usize = 20;

        let mut sum: u32 = 0x800;
        let mut i = 4;
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

pub fn ping() {
    let sock = net::socket(
        net::AddressFamily::INET,
        net::SocketType::DGRAM,
        Some(net::ipproto::ICMP),
    )
        .unwrap();
    net::connect_v4(
        &sock,
        &net::SocketAddrV4::new(Ipv4Addr::new(1, 1, 1, 1), 0),
    )
        .unwrap();
    
    net::send(
        &sock,
        PingICMP::new().as_slice(),
        net::SendFlags::empty(),
    )
        .expect("TODO: panic message");
    let mut buff = [0_u8;20];
    net::recv(&sock, &mut buff,net::RecvFlags::WAITALL).unwrap();
    println!("{:?}", buff);
}

#[cfg(test)]
mod tests {
    use crate::base::linux::ping;

    #[test]
    fn test_ping() {
        ping()
    }
}
