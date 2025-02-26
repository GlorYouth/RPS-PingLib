use rps_ping_lib::PingV6Builder;

#[test]
fn test_ping_v6() {
    let ping = PingV6Builder {
        timeout: 150,
        ttl: None,
        bind_addr: None,
        scope_id_option: None,
        #[cfg(target_os = "windows")]
        window_addition: None,
    }
    .build();
    println!(
        "{} ms",
        ping.ping("2408:8756:c52:1aec:0:ff:b013:5a11".parse().unwrap())
            .expect("ping_v6 error")
            .as_micros() as f64
            / 1000.0
    );
}

#[test]
fn test_ping_v6_in_detail() {
    let ping = PingV6Builder {
        timeout: 200,
        ttl: Some(100),
        bind_addr: None,
        scope_id_option: None,
        #[cfg(target_os = "windows")]
        window_addition: None,
    }
    .build();
    let result = ping
        .ping_in_detail("2408:8756:c52:1aec:0:ff:b013:5a11".parse().unwrap())
        .expect("ping_v6_in_detail error");
    println!(
        "{},{}",
        result.ip,
        result.duration.as_micros() as f64 / 1000.0
    );
}
