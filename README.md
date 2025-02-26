# RPS-PingLib

## Intro
RPS-PingLib is a rust ping lib which support windows and linux and designed for RPS-DNS project.

It is directly implemented using windows.rs and libc.rs.

## Usage
Cargo.toml:
```
[dependencies]
rps_ping_lib = { git = "https://github.com/GlorYouth/RPS-PingLib" }
```


example.rs:
```rust
fn main() {
    let ping = rps_ping_lib::PingV4Builder::new(1000).build();
    let duration = ping.ping("1.1.1.1".parse().unwrap()).unwrap();
    println!("{:?}", duration);

    let mut builder = rps_ping_lib::PingV4Builder::new(1000);
    { // optional
        builder.ttl = Some(5);
        builder.bind_addr = Some(std::net::Ipv4Addr::new(0, 0, 0, 0));
        #[cfg(target_os = "windows")]
        builder.window_addition = None;
    }
    let result = builder.build().ping_in_detail("1.1.1.1".parse().unwrap()).unwrap();
    println!("ip:{}, duration:{:?}", result.ip, result.duration);

    let ping = rps_ping_lib::PingV6Builder::new(1000).build();
    let duration = ping.ping("2606:4700:4700::1111".parse().unwrap()).unwrap();
    println!("{:?}", duration);

    let mut builder = rps_ping_lib::PingV6Builder::new(1000);
    { // optional
        builder.ttl = Some(5);
        builder.bind_addr = Some(std::net::Ipv6Addr::from_bits(0));
        builder.scope_id_option = Some(0);
        #[cfg(target_os = "windows")]
        builder.window_addition = None;
    }
    let result = builder.build().ping_in_detail("2606:4700:4700::1111".parse().unwrap()).unwrap();
    println!("ip:{}, duration:{:?}", result.ip, result.duration);
}
```