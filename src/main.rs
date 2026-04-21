// use crossterm::event;
use std::{collections::HashMap, fs};

#[derive(Debug, Clone)]
pub struct TcpPorts {
    pub port : u16,
    pub protocol: String,
    pub state: String,
    pub pid: Option<u32>,
    pub name: Option<String>
}

fn get_tcp_ports()-> Result<Vec<TcpPorts>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("/proc/net/tcp").unwrap();
    let inode_map = build_pid_inode_map().unwrap();
    let mut entries: Vec<TcpPorts> = vec![];
    for line in content.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        println!("{:?}", cols);

        if cols.len() < 10 {
            continue;
        }

        let loc_add = cols[1];
        let state_hex = cols[3];
        let inode = cols[9].parse::<u64>().unwrap_or(0);

        let port = parse_port(loc_add);
        let state = parse_state(state_hex);

        let pid = inode_map.get(&inode).copied();
        let name = pid.and_then(|p| get_process_name(p).ok());

        entries.push(TcpPorts {
            port,
            protocol: "tcp".into(),
            state,
            pid,
            name
        });



    }

    entries.sort_by_key(|e| e.port);
    Ok(entries)
}


fn build_pid_inode_map() -> Result<HashMap<u64, u32>, Box<dyn std::error::Error>> {

    let mut map = HashMap::new();

    for entry in fs::read_dir("/proc")? {

        // getting the entry and its name
        let entry = entry?;
        let name = entry.file_name();

        // extracting the pid from the name, if it fails we skip this entry
        let pid= match name.to_string_lossy().parse() {
            Ok(n) => n,
            Err(_) => continue
        };

        // getting the fd directory for this process, if it fails we skip this entry
        let fd_dir = entry.path().join("fd");
        let Ok(fds) = fs::read_dir(&fd_dir) else { continue };

        for fd in fds {
            let Ok(fd) = fd else {continue;};
            let Ok(socket_link) = fs::read_link(fd.path()) else {continue;};

            let socket_str = socket_link.to_string_lossy();

            if let Some(inode_str) = socket_str.strip_prefix("socket:[").and_then(|s| s.strip_suffix("]")) {
                if let Ok(inode_no) = inode_str.parse::<u64>() {
                    map.insert(inode_no, pid);
                }
            }
        }

    }
    println!("PID to Inode Map: {:?}", map);
    Ok(map)
}


fn get_process_name(pid: u32) -> Result<String, Box<dyn std::error::Error>> {
    let comm_path = format!("/proc/{}/comm", pid);
    let name = fs::read_to_string(comm_path)?.trim().to_string();
    Ok(name)
}

fn parse_port(loc_add: &str) -> u16 {
    let parts: Vec<&str> = loc_add.split(":").collect();
    if parts.len() != 2 {
        return 0;
    }
    u16::from_str_radix(parts[1],16).unwrap_or(0)
}

fn parse_state(state_hex: &str) -> String {
    match state_hex {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        "0C" => "SYN_RECV",
        _ => "UNKNOWN"
    }.to_string()
}

fn main() {
    if let Err(e) = get_tcp_ports() {
        eprintln!("Error: {}", e);
    } else {
        println!("TCP ports retrieved successfully.");
    }
}
