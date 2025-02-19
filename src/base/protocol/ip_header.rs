use crate::base::utils::SliceReader;

pub struct Ipv4Header<'a> {
    fix_slice: &'a [u8],
    op_slice: &'a [u8],
}

impl<'a> Ipv4Header<'a> {
    pub const FIXED_HEADER_SIZE: usize = 20;
    
    pub fn from_reader<'b: 'a>(reader: & mut SliceReader<'b>) -> Ipv4Header<'a> {
        let header_length = (reader.peek_u8() << 4) >> 4;
        if header_length > 20 {
            reader.skip(header_length as usize);
            Ipv4Header {
                fix_slice: reader.read_slice(20),
                op_slice: reader.read_slice(header_length as usize - 20),
            }
        } else {
            Ipv4Header {
                fix_slice: reader.read_slice(20),
                op_slice: &[],
            }
        }
    }
    
    pub fn get_type(&self) -> u8 {
        self.fix_slice[1]
    }
    
    pub fn get_payload_length(&self) -> u16 {
        u16::from_be_bytes(self.fix_slice[2..4].try_into().unwrap())
    }
    
    pub fn get_source_address(&self) -> std::net::Ipv4Addr {
        std::net::Ipv4Addr::from(<&[u8] as TryInto<[u8;4]>>::try_into(&self.fix_slice[12..16]).unwrap())
    }
    
    pub fn get_destination_address(&self) -> std::net::Ipv4Addr {
        std::net::Ipv4Addr::from(<&[u8] as TryInto<[u8;4]>>::try_into(&self.fix_slice[16..20]).unwrap())
    }
}