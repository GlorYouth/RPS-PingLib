use crate::base::protocol::{Ipv4Header, Ipv6Header};
use rand::Rng;

pub struct IcmpDataForPing {
    data: [u8; IcmpDataForPing::DATA_SIZE],
}

impl IcmpDataForPing {
    pub const DATA_SIZE: usize = 22;

    #[inline]
    pub fn new_ping_v4() -> Self {
        let request_data: u128 = rand::rng().random();

        let mut data = [0_u8; Self::DATA_SIZE];
        data[0] = 8;
        data[6..].copy_from_slice(&request_data.to_be_bytes());

        Self::process_check_sum(&mut data);

        IcmpDataForPing { data }
    }

    #[inline]
    pub fn new_ping_v6() -> Self {
        let request_data: u128 = rand::rng().random();

        let mut data = [0_u8; Self::DATA_SIZE];
        data[0] = 128;
        data[6..].copy_from_slice(&request_data.to_be_bytes());

        Self::process_check_sum(&mut data);

        IcmpDataForPing { data }
    }

    fn process_check_sum(data: &mut [u8; Self::DATA_SIZE]) {
        let mut sum: u32 = 0;
        let mut i = 0;
        while i < IcmpDataForPing::DATA_SIZE {
            // 取出每两个字节，拼接成16位
            let word = if i + 1 < IcmpDataForPing::DATA_SIZE {
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
    }

    fn icmp_type(&self) -> u8 {
        self.data[0]
    }

    #[inline]
    pub fn get_inner(&self) -> &[u8; IcmpDataForPing::DATA_SIZE] {
        &self.data
    }

    #[inline]
    pub fn get_inner_mut(&mut self) -> &mut [u8; IcmpDataForPing::DATA_SIZE] {
        &mut self.data
    }

    #[inline]
    pub fn into_inner(self) -> [u8; IcmpDataForPing::DATA_SIZE] {
        self.data
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }
}

pub struct IcmpFormat<'a> {
    icmp_type: u8,
    code: u8,
    checksum: u16,
    other_data: &'a [u8],
}

impl<'a> IcmpFormat<'a> {
    pub fn from_slice(slice: &[u8]) -> Option<IcmpFormat<'_>> {
        if slice.len() < 4 {
            None
        } else {
            Some(IcmpFormat {
                icmp_type: slice[0],
                code: slice[1],
                checksum: u16::from_be_bytes(slice[2..4].try_into().unwrap()),
                other_data: &slice[4..],
            })
        }
    }

    #[inline]
    pub fn from_header_v4(header: &Ipv4Header<'a>) -> Option<IcmpFormat<'a>> {
        IcmpFormat::from_slice(header.get_payload())
    }

    #[inline]
    pub fn icmp_type(&self) -> u8 {
        self.icmp_type
    }

    pub fn check_is_correspond_v4(&self, data: &IcmpDataForPing) -> Option<()> {
        match (data.icmp_type(), self.icmp_type) {
            (8, 0) => self.other_data[2..].eq(&data.data[6..]).then_some(()),
            (8, 11) => {
                // Time to live exceeded
                Ipv4Header::from_slice_uncheck(&self.other_data[4..]) // 使用uncheck的原因是部分Time to live exceeded响应并未传递ICMP请求的Data部分非序列号和识别部分
                    .and_then(|header| IcmpFormat::from_header_v4(&header))
                    .and_then(|icmp| {
                        // 直接比较checksum,因为有部分响应实现并未传递其余部分
                        icmp.checksum
                            .eq(&u16::from_be_bytes([data.data[2], data.data[3]]))
                            .then_some(())
                    })
            }
            _ => None,
        }
    }

    pub fn check_is_correspond_v6(&self, data: &IcmpDataForPing) -> Option<()> {
        match (data.icmp_type(), self.icmp_type) {
            (128, 129) => self.other_data[2..].eq(&data.data[6..]).then_some(()),
            (128, 3) => Ipv6Header::from_slice(&self.other_data[4..])
                .and_then(|header| IcmpFormat::from_slice(header.get_payload()?))
                .and_then(|format| format.other_data[2..].eq(&data.data[6..]).then_some(())),
            _ => None,
        }
    }
}
