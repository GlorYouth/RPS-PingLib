use multi_ping::{PingV4Builder, PingV6Builder};

fn main() {
    let ping = PingV4Builder {
        timeout: 200,
        ttl: Some(6),
        bind_addr: None,
        #[cfg(target_os = "windows")]
        window_addition: None,
    }
    .build();
    let result = ping
        .ping_in_detail(std::net::Ipv4Addr::new(1, 1, 1, 1))
        .expect("ping_v4_in_detail error");
    println!(
        "{},{}",
        result.ip,
        result.duration.as_micros() as f64 / 1000.0
    );
}
