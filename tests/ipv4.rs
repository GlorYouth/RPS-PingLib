use rps_ping_lib::PingV4Builder;

#[test]
fn test_ping_v4() {
    let ping = PingV4Builder {
        timeout: 200,
        ttl: Some(50),
        bind_addr: None,
        #[cfg(target_os = "windows")]
        window_addition: None,
    }
    .build();
    println!(
        "{} ms",
        ping.ping(std::net::Ipv4Addr::new(1, 1, 1, 1))
            .expect("ping_v4 error")
            .as_micros() as f64
            / 1000.0
    );
}

#[test]
fn test_ping_in_detail() {
    let ping = PingV4Builder {
        timeout: 200,
        ttl: Some(5),
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
