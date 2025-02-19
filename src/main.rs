use multi_ping::{PingV4, PingV4Builder};

fn main() {
    let ping: PingV4 = PingV4Builder {
        timeout: 1000,
        ttl: Some(5),
        bind_addr: None,
    }
    .into();
    println!(
        "{} ms",
        ping.ping(std::net::Ipv4Addr::new(1, 1, 1, 1))
            .expect("ping_v4 error")
            .as_micros() as f64
            / 1000.0
    );
}
