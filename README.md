# linky-mythic

A Mythic payload type providing Rust-native implants for Linux, Windows, and macOS.

> **Linky reimagined as a Mythic agent** — same Rust implants, Mythic handles GUI / backend / DB.
>
> **AI-assisted project notice** — this repository was written with the help of AI. Treat the code, documentation, and operational behavior with caution, and review everything carefully before building, deploying, or using it in a real environment.
>
> **Authorized-use only** — this project is provided strictly for testing, research, training, and explicitly authorized security exercises. Do not use it against systems, networks, or data without clear prior authorization. Any illegal or abusive use is prohibited.

---

## Overview

`linky-mythic` is a **Mythic payload type**. It installs into an existing Mythic instance and adds the ability to build and control Rust implants across multiple platforms.

This repository does **not** provide:

- a standalone C2 server
- a web UI
- a database
- a replacement for Mythic itself

Those pieces are provided by **Mythic**. This project provides the payload type container and the implant code.

Quick install:

```bash
sudo ./mythic-cli install github https://github.com/Nariod/linky-mythic
```

---

## Current status

**Beta — live-tested.** The codebase has been **validated against a live Mythic v3.4.32 instance** with real implant callbacks (April 2026):

- ✅ Payload type registers and syncs with Mythic via RabbitMQ
- ✅ HTTP C2 profile integration (HTTPS recommended, HTTP supported)
- ✅ Linux payload builds successfully (release: **~1.9 MB** with ureq)
- ✅ Windows payload cross-compiles successfully (mingw-w64)
- ✅ Windows + indirect syscalls build: ✅
- ✅ Linux shellcode export: ✅
- ❌ macOS build: expected failure (osxcross not installed)
- ✅ Mythic-compatible encryption (AES-256-CBC + HMAC-SHA256)
- ✅ Chunked file transfer (download + upload) via Mythic file-store API
- ✅ **Live callback verified** — Linux implant checks in and executes **16/16 commands**
- ✅ **Windows callbacks verified** — both standard and indirect syscalls variants: **21/21 commands pass**
- ✅ OPSEC: string obfuscation (`obfstr`), path remapping, debuginfo stripped
- ✅ **Indirect syscalls** (Windows): optional `inject` via [syscalls-rs](https://github.com/Nariod/syscalls-rs) — NtAPI calls bypass user-mode hooks
- ✅ All unit tests pass (Go build + 9 Rust tests)

### Binary sizes

| Platform | Release | Notes |
|----------|---------|-------|
| Linux x86_64 | ~1.9 MB | musl, static, stripped, LTO, `opt-level=z`, `panic=abort` |
| Windows x86_64 | ~2 MB | mingw-w64, stripped |
| macOS x86_64 | N/A | requires osxcross (not in Dockerfile yet) |

### Live command test results (Linux — April 2026, Mythic v3.4.0.52)

| Command | Status | Notes |
|---------|--------|-------|
| whoami | ✅ | `fedora@` (user@hostname) |
| pwd | ✅ | `/home/fedora/Documents/linky-mythic` |
| ls | ✅ | sorted listing with directory indicators |
| cd | ✅ | returns `[+] /new/path` |
| pid | ✅ | `58542` (actual PID) |
| info | ✅ | OS Version, arch, user, hostname, IP addresses |
| ps | ✅ | PID/PPID/USER/COMMAND table |
| netstat | ✅ | Proto/Local/Remote/State/PID table |
| shell | ✅ | `echo hello_from_linky` → `hello_from_linky` |
| download | ✅ | Mythic chunked file transfer (verified in Phase 6) |
| upload | ✅ | Mythic pull-down protocol (requires UI modal) |
| sleep | ✅ | `[+] sleep: 10s, jitter: 23%` |
| killdate | ✅ | `no killdate set` / set/clear working |
| cp | ✅ | file/directory copy (recursive) |
| mv | ✅ | move/rename |
| rm | ✅ | file/directory removal (recursive) |
| mkdir | ✅ | recursive directory creation |
| execute | ✅ | `/usr/bin/uname -a` → full kernel info |
| exit | ✅ | clean agent termination |

### Live command test results (Windows — April 2026, Mythic v3.4.32)

Both build variants tested: **standard** and **indirect syscalls**. All 21 commands pass identically on both.

| Command | Standard | Indirect Syscalls | Notes |
|---------|----------|-------------------|-------|
| whoami | ✅ | ✅ | `Nariod@WIN-KKTTS06FQCO` |
| pwd | ✅ | ✅ | `C:\Users\Nariod\Desktop\Dev` |
| pid | ✅ | ✅ | Returns actual PID |
| info | ✅ | ✅ | OS, arch, user, hostname, IPs |
| integrity | ✅ | ✅ | Integrity level 3 (High) |
| ls | ✅ | ✅ | Sorted directory listing |
| cd | ✅ | ✅ | Changes working directory |
| shell | ✅ | ✅ | `cmd.exe /C` execution |
| ps | ✅ | ✅ | Process list with PID/name/user |
| netstat | ✅ | ✅ | Network connections table |
| mkdir | ✅ | ✅ | Creates directories recursively |
| cp | ✅ | ✅ | File copy (plain text: `source dest`) |
| mv | ✅ | ✅ | Move/rename (plain text: `source dest`) |
| rm | ✅ | ✅ | File/directory removal |
| execute | ✅ | ✅ | Direct binary execution (plain text: `path args`) |
| sleep | ✅ | ✅ | `seconds jitter%` format |
| killdate | ✅ | ✅ | Unix timestamp or `clear` |
| download | ✅ | ✅ | Mythic chunked file transfer |
| upload | ✅ | ✅ | File upload via Mythic file store |
| inject | ✅ | ✅ | Shellcode injection (JSON or `pid base64` format) |
| exit | ✅ | ✅ | Clean agent termination |

**Bugs found and fixed during Windows testing:**

| Bug | Root Cause | Fix |
|-----|-----------|-----|
| `upload` — "Required arg, file, was not specified" | `TaskFunctionParseArgString` didn't handle JSON input; only set `remote_path` | Added JSON detection via `LoadArgsFromJSONString` fallback |
| `inject` — "Required arg, pid, was not specified" | No `TaskFunctionParseArgString` defined at all | Added parser supporting both JSON dict and `<pid> <base64>` string formats |

---

## Quick start

1. Install and start Mythic.
2. Install the Mythic HTTP C2 profile.
3. Install `linky-mythic`.
4. Open the Mythic UI and generate a payload using `linky`.

```bash
# 1. Install and start Mythic
git clone https://github.com/its-a-feature/Mythic
cd Mythic && make
sudo ./mythic-cli start

# 2. Install the HTTP C2 profile
sudo ./mythic-cli install github https://github.com/MythicC2Profiles/http

# 3. Install linky-mythic
sudo ./mythic-cli install github https://github.com/Nariod/linky-mythic

# 4. Open the Mythic UI
# https://localhost:7443
```

---

## Payload generation notes

When creating a payload in Mythic:

1. Select the `linky` payload type.
2. Use the `http` C2 profile with `AESPSK` set to `aes256_hmac`.
3. Set the callback values carefully.

Recommended HTTP profile values:

- `callback_host`: `https://<your-server-ip-or-domain>`
- `callback_port`: `443` (or your chosen TLS port)
- `callback_interval`: `10`
- `callback_jitter`: `23`

The implant preserves the scheme from `callback_host`. HTTPS is strongly recommended. HTTP is supported but offers no transport encryption.

> **Important**: Agent traffic goes to the **HTTP C2 profile container** (port 443 by default), NOT to the Mythic nginx frontend (port 7443). The `callback_uri` must match the C2 profile's `post_uri` (e.g., `/data`). Using `/` as callback_uri may cause a 301 redirect.

---

## Known issues (audit April 2026)

| ID | Severity | Description |
|----|----------|-------------|
| GO-01 | ✅ Fixed | ~~`encryptCallback` returns plaintext on crypto failure~~ — returns error |
| GO-02 | ✅ Fixed | ~~Dockerfile hardcodes RabbitMQ credentials~~ — defaults removed |
| RS-01 | ✅ Fixed | ~~Non-constant-time HMAC comparison~~ — uses `hmac::Mac::verify_slice()` |
| RS-02 | ✅ Fixed | ~~`handle_sleep_command` can panic on whitespace-only input~~ |
| GO-06 | ✅ Fixed | ~~Default `callback_uri` of `/` conflicts with nginx~~ — changed to `/data` |
| GO-08 | ✅ Fixed | ~~`upload.go` `ParseArgString` didn't handle JSON~~ — added `LoadArgsFromJSONString` |
| GO-09 | ✅ Fixed | ~~`inject.go` missing `ParseArgString`~~ — added JSON + string parser |

See [TODO.md](TODO.md) Phase 17 for the complete audit report.

---

## Supported capabilities

### Cross-platform commands (Linux, Windows, macOS)

| Command | Description |
|---------|-------------|
| shell | Execute a command via the OS shell (`/bin/sh`, `cmd.exe`) |
| execute | Execute a binary directly (no shell wrapper) |
| ls | List directory contents (sorted) |
| cd | Change working directory |
| pwd | Print working directory |
| whoami | Current user and hostname |
| pid | Process ID |
| info | OS, arch, user, hostname, IP, uptime |
| ps | Process list |
| netstat | Network connections |
| download | Download a file from the target (Mythic chunked transfer) |
| upload | Upload a file to the target (Mythic pull-down protocol) |
| sleep | Set callback interval and jitter percentage |
| killdate | Set agent expiration (epoch timestamp) |
| exit | Terminate the implant |
| cp | Copy file or directory (recursive) |
| mv | Move / rename |
| rm | Remove file or directory (recursive) |
| mkdir | Create directory (recursive) |

### Windows-only commands

| Command | Description |
|---------|-------------|
| inject | Remote process shellcode injection (VirtualAllocEx + CreateRemoteThread) |
| integrity | Query process integrity level (Low/Medium/High/System) |
| cmd | Execute via `cmd.exe /C` |
| powershell | Execute via `powershell.exe -noP -sta -w 1 -c` |

---

## Architecture

### How it fits into Mythic

```text
┌─────────────────────────────┐     ┌──────────────────────────────┐
│       Mythic Core           │     │    linky-mythic container    │
│                             │     │                              │
│  Web UI                     │     │  Go payload type service     │
│  PostgreSQL                 │◄───►│  ├── builder.go (cargo build)│
│  RabbitMQ                   │     │  └── 21 command definitions  │
│  HTTP C2 profile (TLS)      │     │                              │
│  GraphQL / WebSocket        │     │  Rust implant workspace      │
│                             │     │  ├── common/ (protocol, C2)  │
└─────────────────────────────┘     │  ├── linux/                  │
         ▲                          │  ├── windows/                │
         │ HTTPS (AES-256-CBC       │  └── osx/                   │
         │  + HMAC-SHA256)          └──────────────────────────────┘
         │
    ┌────┴────┐
    │ Implant │  (~1.9 MB Rust binary)
    └─────────┘
```

Build flow: `Mythic UI` → `build request` → `builder.go` → `cargo build --release` → binary returned to Mythic

### Mythic wire format (AES256_HMAC)

```
base64( UUID(36 bytes) + IV(16 bytes) + AES-256-CBC(PKCS7(JSON)) + HMAC-SHA256(32 bytes) )
```

The encryption key is the 32-byte AESPSK from the HTTP C2 profile (base64-decoded at runtime).

### Key technology choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| HTTP client | `ureq 3` (not reqwest) | No tokio/hyper — binary ~1.9 MB vs ~4.5 MB |
| TLS | `rustls` via `ring` | No OpenSSL, no `aws-lc-sys` — simpler cross-compilation |
| Crypto | `aes` + `cbc` + `hmac` + `sha2` | Mythic AES256_HMAC compatible |
| String obfuscation | `obfstr 0.4` | Compile-time string encryption for protocol strings |
| Memory safety | `zeroize 1` | AES keys zeroed after use |

---

## linky-mythic vs Hannibal

Competitive reference: [silentwarble/Hannibal](https://github.com/silentwarble/Hannibal) — Mythic agent in C, Windows x64 only.

| Aspect | Hannibal | linky-mythic |
|--------|----------|-------------|
| Binary size | 25-45 KB (shellcode) | ~1.9 MB |
| Platforms | Windows x64 only | Linux + Windows + macOS |
| Language | C (PIC, custom linker) | Rust (safe, idiomatic) |
| Sleep obfuscation | Ekko (RC4 .text encryption) | Not yet |
| Indirect syscalls | N/A (no inject) | ✅ Optional ([syscalls-rs](https://github.com/Nariod/syscalls-rs)) |
| String obfuscation | Hash compile-time (ROL5) | `obfstr` compile-time encryption |
| Post-exploitation | HBIN dynamic modules | Not yet |
| Memory safety | Manual (C) | Compiler-enforced (Rust) |
| Unit tests | None | 9 tests (Go + Rust) |

---

## Known limitations

- macOS cross-compilation requires osxcross (not included in Dockerfile).
- No AMSI/ETW bypass yet (see roadmap).
- `inject` uses Win32 APIs by default; enable `indirect-syscalls` feature for NT API path via syscalls-rs.
- Binary size gap with pure-C agents like Hannibal (1.9 MB vs 25-45 KB).
- **SELinux (Fedora/RHEL)**: requires `chcon` relabeling after `mythic-cli install` (see Build notes).

---

## Build and container notes

The payload type container embeds the Rust toolchain and cross-compilation targets. In normal Mythic usage, no additional host-side Rust setup is necessary beyond Docker and Mythic.

Container prerequisites baked into the image:

```dockerfile
FROM golang:1.25 AS go-builder
# ... builds Go payload type service (CGO_ENABLED=0 for static binary)

FROM rust:latest
RUN apt-get install -y musl-tools mingw-w64 clang lld binutils
RUN rustup target add x86_64-unknown-linux-musl x86_64-pc-windows-gnu
```

> **Important**: The Go binary is installed to `/usr/local/bin/` (not `/Mythic/`).
> Mythic bind-mounts `InstalledServices/linky/` onto `/Mythic/` at runtime,
> which would hide any binary placed there.

### SELinux (Fedora / RHEL)

On systems with SELinux enforcing, Docker bind mounts are blocked by default.
After installing linky, relabel the files:

```bash
sudo chcon -Rt svirt_sandbox_file_t /path/to/Mythic/InstalledServices/linky/
```

Without this, `cargo build` inside the container will fail with permission errors.

### Running outside Docker (local development)

```bash
AGENT_CODE_DIR=/path/to/linky-mythic/Payload_Type/linky/agent_code \
RABBITMQ_HOST=127.0.0.1 \
RABBITMQ_PASSWORD=<from Mythic .env> \
MYTHIC_SERVER_HOST=127.0.0.1 \
./linky-container
```

The builder falls back to `/Mythic/agent_code` when `AGENT_CODE_DIR` is not set (the default path inside the Docker container).

---

## Project layout

```text
linky-mythic/
├── config.json                             # Mythic payload type configuration
├── agent_capabilities.json                 # Capability summary
├── Payload_Type/
│   └── linky/
│       ├── Dockerfile                      # Multi-stage: Go builder + Rust toolchain
│       ├── main.go                         # Mythic container entry point
│       ├── go.mod                          # Go 1.25, MythicContainer v1.6.4
│       ├── mythic/
│       │   └── agent_functions/
│       │       ├── builder.go              # Build orchestration + AES callback encryption
│       │       ├── shell.go ... exit.go    # 21 command definitions (Go ↔ Mythic)
│       │       └── utils.go               # Shared helpers (splitArgs)
│       └── agent_code/                     # Rust workspace
│           ├── Cargo.toml                  # Workspace: release profile (LTO, strip, opt-z)
│           └── links/
│               ├── common/
│               │   ├── Cargo.toml          # ureq 3, obfstr, aes+cbc+hmac+sha2, zeroize
│               │   └── src/
│               │       ├── lib.rs          # C2 loop, crypto, file transfers, HTTP client
│               │       └── dispatch.rs     # Cross-platform command dispatch
│               ├── linux/                  # Linux-specific: /proc parsing, hostname fallback
│               ├── windows/                # Windows-specific: inject, integrity, tasklist
│               └── osx/                    # macOS-specific: shell fallbacks for ps/netstat
└── ...
```

---

## Development validation commands

```bash
# Go (from Payload_Type/linky/)
cd Payload_Type/linky
go build ./... && go vet ./...

# Rust workspace (from Payload_Type/linky/agent_code/)
cd Payload_Type/linky/agent_code
CALLBACK=x \
  IMPLANT_SECRET=$(python3 -c "import base64,os;print(base64.b64encode(os.urandom(32)).decode())") \
  PAYLOAD_UUID=x CALLBACK_URI=/ \
  cargo test --workspace

# Format check
cargo fmt --check
```

---

## OPSEC posture

| Feature | Status |
|---------|--------|
| AES-256-CBC + HMAC-SHA256 encryption | ✅ Done |
| 32-byte random AESPSK from Mythic C2 profile | ✅ Done |
| Encryption keys zeroized after use | ✅ Done |
| String obfuscation (`obfstr`) on protocol strings | ✅ Done |
| Cargo path remapping (`--remap-path-prefix`) | ✅ Done |
| Debug info stripped (`-C debuginfo=0`) | ✅ Done |
| Release binary stripped + LTO + `panic=abort` | ✅ Done |
| Configurable User-Agent | ⬜ Planned |
| Indirect syscalls (Windows inject) | ✅ Optional (feature flag `indirect-syscalls`) |
| Constant-time HMAC verification | ✅ Done (verify_slice) |
| Sleep obfuscation (Windows) | ⬜ Research |
| AMSI/ETW bypass (Windows) | ⬜ Planned |
| Conditional command compilation (Cargo features) | ⬜ Planned |

---

## Roadmap

See [TODO.md](TODO.md) for the detailed phase-by-phase plan.

### Near-term (bug fixes from audit)
- ~~Fix `encryptCallback` plaintext fallback (GO-01)~~ ✅
- ~~Use env vars for RabbitMQ credentials in Dockerfile (GO-02)~~ ✅
- ~~Constant-time HMAC comparison (RS-01)~~ ✅
- ~~Fix `handle_sleep_command` panic on whitespace input (RS-02)~~ ✅
- ~~Pin Rust version in Dockerfile for reproducible builds (GO-07)~~ ✅

### Near-term (features)
- Mythic `process_browser` and `file_browser` structured JSON output
- Configurable User-Agent via build parameter
- `ipinfo` command (network interface info)
- Conditional command compilation via Cargo features (operator picks commands at build time)

### Medium-term
- Sleep obfuscation research (Windows — Ekko-style)
- Full macOS support with osxcross in Dockerfile
- ARM64 targets (`aarch64-unknown-linux-musl`, `aarch64-apple-darwin`)

### Long-term
- Dynamic module loading (Rust equivalent of Hannibal's HBIN)
- SOCKS proxy for network pivoting
- AMSI/ETW bypass (Windows)
- Hugo documentation site
