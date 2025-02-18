use multi_ping::SinglePing;

fn main() {
    let ping = SinglePing::default();
    println!(
        "{} ms",
        ping.ping_v4(std::net::Ipv4Addr::new(1, 1, 1, 1))
            .expect("ping_v4 error")
            .as_micros() as f64
            / 1000.0
    );
}
