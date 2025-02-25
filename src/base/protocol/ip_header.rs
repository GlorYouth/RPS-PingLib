#[derive(Debug)]
pub struct Ipv4Header<'a> {
    fix_slice: &'a [u8],
    // op_slice: &'a [u8], // 可选数据
    payload_slice: &'a [u8],
}

impl<'a> Ipv4Header<'a> {
    pub const FIXED_HEADER_SIZE: u16 = 20;

    pub fn from_slice<'b: 'a>(slice: &'b [u8]) -> Option<Ipv4Header<'a>> {
        if slice.len() < Ipv4Header::FIXED_HEADER_SIZE as usize {
            return None;
        }
        let header_length = (slice[0] << 4) >> 2;
        if slice.len() < header_length as usize {
            return None;
        }
        // we don't consider check the total size because many responses set this part wrongly
        Some(Ipv4Header {
            fix_slice: &slice[0..Self::FIXED_HEADER_SIZE as usize],
            // op_slice: &slice[Self::FIXED_HEADER_SIZE as usize..header_length as usize],
            payload_slice: &slice[header_length as usize..],
        })
    }

    // #[inline]
    // pub fn get_type(&self) -> u8 {
    //     self.fix_slice[1]
    // }

    // #[inline]
    // fn get_payload_length(&self) -> u16 {
    //     u16::from_be_bytes(self.fix_slice[2..4].try_into().unwrap())
    // }

    #[inline]
    pub fn get_source_address(&self) -> std::net::Ipv4Addr {
        std::net::Ipv4Addr::from(
            <&[u8] as TryInto<[u8; 4]>>::try_into(&self.fix_slice[12..16]).unwrap(),
        )
    }

    // #[inline]
    // pub fn get_destination_address(&self) -> std::net::Ipv4Addr {
    //     std::net::Ipv4Addr::from(
    //         <&[u8] as TryInto<[u8; 4]>>::try_into(&self.fix_slice[16..20]).unwrap(),
    //     )
    // }

    #[inline]
    pub fn get_payload(&self) -> &'a [u8] {
        self.payload_slice
    }
}

#[derive(Debug)]
pub struct Ipv6Header<'a> {
    // fix_slice: &'a [u8],
    payload_slice_vec: Vec<(u8, &'a [u8])>,
}

impl<'a> Ipv6Header<'a> {
    pub const FIXED_HEADER_SIZE: u16 = 40;

    pub fn from_slice(slice: &'a [u8]) -> Option<Ipv6Header<'a>> {
        let (fix_slice, mut other_slice) =
            slice.split_at_checked(Self::FIXED_HEADER_SIZE as usize)?;
        let payload_length = u16::from_be_bytes(fix_slice[4..6].try_into().unwrap());
        if payload_length + Self::FIXED_HEADER_SIZE < slice.len() as u16 {
            return None;
        }
        let mut next_header_type = fix_slice[6];
        let mut payload_slice_vec = Vec::with_capacity(2);
        loop {
            match Ipv6HeaderType::new(next_header_type) {
                Ipv6HeaderType::Options(u) => {
                    next_header_type = other_slice[0];
                    let length = other_slice[1];
                    let slice;
                    (slice, other_slice) =
                        other_slice.split_at_checked(Self::alignment_u8_size(length) as usize)?;
                    payload_slice_vec.push((u, slice));
                    continue;
                }
                Ipv6HeaderType::Uppers => {
                    payload_slice_vec.push((next_header_type, other_slice));
                    return Some(Ipv6Header {
                        // fix_slice,
                        payload_slice_vec,
                    });
                }
                Ipv6HeaderType::Unassigned
                | Ipv6HeaderType::Experimental
                | Ipv6HeaderType::Reserved => return None,
            }
        }
    }

    fn alignment_u8_size(u: u8) -> u8 {
        // option part slices' sizes are aligned to 8 bits, but I can't sure if it right
        if u == 0 {
            return 8;
        }
        let (divisor, is_reminder) = (u >> 3, u & 0b111 > 0);
        if is_reminder { (divisor + 1) << 3 } else { u }
    }

    // #[inline]
    // pub fn get_source_address(&self) -> std::net::Ipv6Addr {
    //     std::net::Ipv6Addr::from(
    //         <&[u8] as TryInto<[u8; 16]>>::try_into(&self.fix_slice[8..24]).unwrap(),
    //     )
    // }

    // #[inline]
    // pub fn get_destination_address(&self) -> std::net::Ipv6Addr {
    //     std::net::Ipv6Addr::from(
    //         <&[u8] as TryInto<[u8; 16]>>::try_into(&self.fix_slice[24..40]).unwrap(),
    //     )
    // }

    // #[inline]
    // pub fn get_type(&self) -> Option<u8> {
    //     Some(self.payload_slice_vec.last()?.0)
    // }
    #[inline]
    pub fn get_payload(&self) -> Option<&'a [u8]> {
        Some(self.payload_slice_vec.last()?.1)
    }
}

enum Ipv6HeaderType {
    Options(u8),
    Uppers,
    Unassigned,
    Experimental,
    Reserved,
}

impl Ipv6HeaderType {
    // Options:
    const HOPOPT: u8 = 0;
    const IPV6_OPTS: u8 = 60;
    const IPV6_ROUTE: u8 = 43;
    const IPV6_FRAG: u8 = 44;
    const AH: u8 = 51;
    const ESP: u8 = 50;

    pub fn new(u: u8) -> Ipv6HeaderType {
        match u {
            Self::HOPOPT
            | Self::IPV6_OPTS
            | Self::IPV6_ROUTE
            | Self::IPV6_FRAG
            | Self::AH
            | Self::ESP => Ipv6HeaderType::Options(u),
            u @ _ => match u {
                0..=145 => Ipv6HeaderType::Uppers,
                146..=252 => Ipv6HeaderType::Unassigned,
                253..=254 => Ipv6HeaderType::Experimental,
                255.. => Ipv6HeaderType::Reserved,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::protocol::Ipv4Header;
    use crate::base::protocol::ip_header::Ipv6Header;
    // use std::str::FromStr;

    #[test]
    fn test_ipv4_header() {
        let slice: &[u8] = &[
            69, 0, 0, 42, 133, 30, 0, 0, 55, 1, 58, 5, 1, 1, 1, 1, 192, 168, 2, 6, 0, 0, 136, 240,
            0, 0, 230, 74, 163, 38, 61, 106, 234, 34, 235, 11, 213, 222, 158, 115, 102, 178,
        ];
        let header = Ipv4Header::from_slice(&slice).unwrap();
        assert_eq!(
            header.fix_slice,
            [
                69, 0, 0, 42, 133, 30, 0, 0, 55, 1, 58, 5, 1, 1, 1, 1, 192, 168, 2, 6
            ]
        );
        // assert_eq!(header.op_slice, []);
        assert_eq!(
            header.payload_slice,
            [
                0, 0, 136, 240, 0, 0, 230, 74, 163, 38, 61, 106, 234, 34, 235, 11, 213, 222, 158,
                115, 102, 178
            ]
        );
    }

    #[test]
    fn test_ipv6_header() {
        let slice = [
            0x60, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x01, 0xfe, 0x80, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xff, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3a, 0x00,
            0x01, 0x00, 0x05, 0x02, 0x00, 0x00, 0x82, 0x00, 0x80, 0x1d, 0x00, 0x0a, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let header = Ipv6Header::from_slice(&slice).unwrap();
        // assert_eq!(
        //     header.fix_slice,
        //     &[
        //         96, 0, 0, 0, 0, 32, 0, 1, 254, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255,
        //         2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
        //     ]
        // );
        assert_eq!(
            header.payload_slice_vec[0],
            (0, &[58, 0, 1, 0, 5, 2, 0, 0][..])
        );
        assert_eq!(
            header.payload_slice_vec[1],
            (
                58,
                &[
                    130, 0, 128, 29, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ][..]
            )
        );
        // assert_eq!(
        //     header.get_source_address(),
        //     std::net::Ipv6Addr::from_str("fe80::1").unwrap()
        // );
        // assert_eq!(
        //     header.get_destination_address(),
        //     std::net::Ipv6Addr::from_str("ff02::1").unwrap()
        // );
    }
}
