use anyhow::Result;
use std::{collections::HashMap, fs, net::Ipv4Addr};

#[derive(Debug, Clone)]
pub struct TcpPorts {
    pub local_add: String,
    pub remote_add: String,
    pub protocol: Protocol,
    pub state: State,
    pub pid: Option<u32>,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Protocol {
    TCP,
}

#[derive(Debug, Clone)]
pub enum State {
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    Unknown(String),
}

pub fn get_tcp_ports() -> Result<Vec<TcpPorts>> {
    let content = fs::read_to_string("/proc/net/tcp")?;
    let inode_map = build_pid_inode_map().unwrap();
    let mut entries: Vec<TcpPorts> = vec![];
    for line in content.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        // println!("{:?}", cols);

        if cols.len() < 10 {
            continue;
        }

        let state_hex = cols[3];
        let inode = cols[9].parse::<u64>().unwrap_or(0);

        let local_add = match decode_address(cols[1]) {
            Some(str) => str,
            None => "No Port".into(),
        };

        let remote_add = match decode_address(cols[2]) {
            Some(str) => str,
            None => "No Port".into(),
        };

        let state = parse_state(state_hex);

        let pid = inode_map.get(&inode).copied();
        let name = pid.and_then(|p| get_process_name(p).ok());

        entries.push(TcpPorts {
            local_add,
            remote_add,
            protocol: Protocol::TCP,
            state,
            pid,
            name,
        });
    }

    entries.sort_by_key(|e| {
        e.local_add
            .split(":")
            .nth(1)
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(0)
    });
    Ok(entries)
}

fn build_pid_inode_map() -> Result<HashMap<u64, u32>> {
    let mut map = HashMap::new();

    for entry in fs::read_dir("/proc")? {
        // getting the entry and its name
        let entry = entry?;
        let name = entry.file_name();

        // extracting the pid from the name, if it fails we skip this entry
        let pid = match name.to_string_lossy().parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        // getting the fd directory for this process, if it fails we skip this entry
        let fd_dir = entry.path().join("fd");
        let Ok(fds) = fs::read_dir(&fd_dir) else {
            continue;
        };

        for fd in fds {
            let Ok(fd) = fd else {
                continue;
            };
            let Ok(socket_link) = fs::read_link(fd.path()) else {
                continue;
            };

            let socket_str = socket_link.to_string_lossy();

            if let Some(inode_str) = socket_str
                .strip_prefix("socket:[")
                .and_then(|s| s.strip_suffix("]"))
            {
                if let Ok(inode_no) = inode_str.parse::<u64>() {
                    map.insert(inode_no, pid);
                }
            }
        }
    }
    // println!("PID to Inode Map: {:?}", map);
    Ok(map)
}

fn get_process_name(pid: u32) -> anyhow::Result<String> {
    let comm_path = format!("/proc/{}/comm", pid);
    let name = fs::read_to_string(comm_path)?.trim().to_string();
    Ok(name)
}

// fn parse_port(loc_add: &str) -> u16 {
//     let parts: Vec<&str> = loc_add.split(":").collect();
//     if parts.len() != 2 {
//         return 0;
//     }
//     u16::from_str_radix(parts[1], 16).unwrap_or(0)
// }

fn decode_address(addr: &str) -> Option<String> {
    let (ip_hex, port_hex) = addr.split_once(':')?;

    let ip = u32::from_str_radix(ip_hex, 16).ok()?;
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    let ip = Ipv4Addr::from(u32::from_be(ip));

    Some(format!("{}:{}", ip, port))
}

fn parse_state(state_hex: &str) -> State {
    match state_hex {
        "01" => State::Established,
        "02" => State::SynSent,
        "03" => State::SynRecv,
        "04" => State::FinWait1,
        "05" => State::FinWait2,
        "06" => State::TimeWait,
        "07" => State::Close,
        "08" => State::CloseWait,
        "09" => State::LastAck,
        "0A" => State::Listen,
        "0B" => State::Closing,
        "0C" => State::SynRecv,
        _ => State::Unknown(state_hex.into()),
    }
    // .to_string()
}
