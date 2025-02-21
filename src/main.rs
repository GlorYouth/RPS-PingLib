use multi_ping::PingV6Builder;

fn main() {
    let ping = PingV6Builder {
        timeout: 200,
        ttl: Some(100),
        bind_addr: None,
        scope_id_option: None,
    }
    .build();
    let result = ping
        .ping("2606:4700:4700::1111".parse().unwrap())
        .expect("ping_v6_in_detail error");
    println!("{}", result.as_micros() as f64 / 1000.0);
}
