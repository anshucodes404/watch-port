// use crossterm::event;

mod app;
mod proc;
mod tui;

use std::{collections::HashMap, fs, time::Instant};

#[derive(Debug, Clone)]
pub struct PortSnapShot {
    time: Instant,
    entries: Vec<TcpPorts>,
}




pub enum PortEvent {
//     Opened(TcpPorts),
//     Closed(TcpPorts),
//     StateChanged { old: TcpPorts, new: TcpPorts },
// }


fn main() {
    let ports = get_tcp_ports().unwrap();

    println!(
        "{:<10} {:<10} {:<15} {:<10} {}",
        "PORT", "PROTOCOL", "STATE", "PID", "NAME"
    );

    println!("{}", "-".repeat(60));

    for port in ports {
        println!(
            "{:<10} {:<10?} {:<15?} {:<10} {}",
            port.port,
            port.protocol,
            port.state,
            port.pid.map_or("".to_string(), |p| p.to_string()),
            port.name.unwrap_or_default()
        );
    }
}
