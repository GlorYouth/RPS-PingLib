use multi_ping::{PingV4Builder, PingV6Builder};

fn main() {
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
        .ping_in_detail("2606:4700:4700::1111".parse().unwrap())
        .expect("ping_v6_in_detail error");
    println!(
        "{},{}",
        result.ip,
        result.duration.as_micros() as f64 / 1000.0
    );
}
