use crate::base::utils::SliceReader;

pub struct Ipv4Header<'a> {
    fix_slice: &'a [u8],
    op_slice: &'a [u8], // 可选数据
    payload_slice: &'a [u8],
}

impl<'a> Ipv4Header<'a> {
    pub const FIXED_HEADER_SIZE: u16 = 20;

    pub fn from_reader<'b: 'a>(
        reader: &mut SliceReader<'b>,
        total_len: u16,
    ) -> Option<Ipv4Header<'a>> {
        if reader.len() < total_len as usize || total_len < Self::FIXED_HEADER_SIZE {
            return None;
        }
        let header_length = (reader.peek_u8() << 4) >> 4;
        let payload_length = u16::from_be_bytes(reader.as_ref()[2..4].try_into().unwrap());
        if payload_length != total_len - Self::FIXED_HEADER_SIZE - header_length as u16
            || reader.len() > (payload_length as usize + header_length as usize)
        {
            return None;
        }
        Some(Ipv4Header {
            fix_slice: reader.read_slice(Self::FIXED_HEADER_SIZE as usize),
            op_slice: reader.read_slice(header_length as usize - Self::FIXED_HEADER_SIZE as usize),
            payload_slice: reader.read_slice(payload_length as usize),
        })
    }

    pub fn get_type(&self) -> u8 {
        self.fix_slice[1]
    }

    fn get_payload_length(&self) -> u16 {
        u16::from_be_bytes(self.fix_slice[2..4].try_into().unwrap())
    }

    pub fn get_source_address(&self) -> std::net::Ipv4Addr {
        std::net::Ipv4Addr::from(
            <&[u8] as TryInto<[u8; 4]>>::try_into(&self.fix_slice[12..16]).unwrap(),
        )
    }

    pub fn get_destination_address(&self) -> std::net::Ipv4Addr {
        std::net::Ipv4Addr::from(
            <&[u8] as TryInto<[u8; 4]>>::try_into(&self.fix_slice[16..20]).unwrap(),
        )
    }

    pub fn get_payload(&self) -> &'a [u8] {
        self.payload_slice
    }
}

pub struct Ipv6Header<'a> {
    fix_slice: &'a [u8],
    payload_slice_vec: Vec<(u8, &'a [u8])>,
}

impl<'a> Ipv6Header<'a> {
    pub const FIXED_HEADER_SIZE: u16 = 40;

    pub fn from_reader<'b: 'a>(
        reader: &mut SliceReader<'b>,
        total_len: u16,
    ) -> Option<Ipv6Header<'a>> {
        if reader.len() < total_len as usize || total_len < Self::FIXED_HEADER_SIZE {
            return None;
        }
        let fix_slice = reader.read_slice(Self::FIXED_HEADER_SIZE as usize);
        let payload_length = u16::from_be_bytes(fix_slice[4..6].try_into().unwrap());
        if payload_length + Self::FIXED_HEADER_SIZE < total_len {
            return None;
        }
        let mut next_header_type = fix_slice[6];
        let mut payload_slice_vec = Vec::with_capacity(2);
        loop {
            match Ipv6HeaderType::new(next_header_type) {
                Ipv6HeaderType::Options(u) => {
                    next_header_type = reader.peek_u8();
                    let length = reader.as_ref()[reader.pos() + 1];
                    payload_slice_vec.push((u, reader.read_slice(length as usize)));
                    continue;
                }
                Ipv6HeaderType::Uppers(_) => {
                    payload_slice_vec.push((next_header_type, reader.remainder()));
                    return Some(Ipv6Header {
                        fix_slice,
                        payload_slice_vec,
                    });
                }
                Ipv6HeaderType::Unassigned(_)
                | Ipv6HeaderType::Experimental(_)
                | Ipv6HeaderType::Reserved(_) => return None,
            }
        }
    }
}

enum Ipv6HeaderType {
    Options(u8),
    Uppers(u8),
    Unassigned(u8),
    Experimental(u8),
    Reserved(u8),
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
                0..=145 => Ipv6HeaderType::Uppers(u),
                146..=252 => Ipv6HeaderType::Unassigned(u),
                253..=254 => Ipv6HeaderType::Experimental(u),
                255.. => Ipv6HeaderType::Reserved(u),
            },
        }
    }
}
