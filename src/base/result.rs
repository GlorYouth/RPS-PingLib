
#[derive(Debug)]
pub struct PingV4Result {
    pub ip: std::net::Ipv4Addr,
    pub duration: std::time::Duration,
}


#[derive(Debug)]
pub struct PingV6Result {
    pub ip: std::net::Ipv6Addr,
    pub duration: std::time::Duration,
}