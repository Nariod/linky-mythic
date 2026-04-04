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

**Beta — live-tested.** The codebase has been **validated against a live Mythic instance** with real implant callbacks (July 2026):

- ✅ Payload type registers and syncs with Mythic via RabbitMQ
- ✅ HTTP C2 profile integration (HTTPS recommended, HTTP supported)
- ✅ Linux payload builds successfully (release: **~1.9 MB** with ureq)
- ✅ Windows payload cross-compiles successfully (mingw-w64)
- ✅ Mythic-compatible encryption (AES-256-CBC + HMAC-SHA256)
- ✅ Chunked file transfer (download + upload) via Mythic file-store API
- ✅ **Live callback verified** — Linux implant checks in and executes commands
- ✅ OPSEC: string obfuscation (`obfstr`), path remapping, debuginfo stripped
- ✅ All unit tests pass (Go build + 9 Rust tests)

### Binary sizes

| Platform | Release | Notes |
|----------|---------|-------|
| Linux x86_64 | ~1.9 MB | musl, static, stripped, LTO, `opt-level=z`, `panic=abort` |
| Windows x86_64 | ~2 MB | mingw-w64, stripped |
| macOS x86_64 | N/A | requires osxcross (not in Dockerfile yet) |

### Live command test results (Linux)

| Command | Status | Notes |
|---------|--------|-------|
| whoami | ✅ | user@hostname |
| pwd | ✅ | current directory |
| ls | ✅ | sorted listing |
| cd | ✅ | returns `[+] /new/path` |
| pid | ✅ | process ID |
| info | ✅ | OS, arch, user, hostname, IP |
| ps | ✅ | process list |
| netstat | ✅ | network connections |
| shell | ✅ | shell command execution via `/bin/sh -c` |
| download | ✅ | Mythic chunked file transfer |
| upload | ✅ | Mythic pull-down protocol (requires UI modal) |
| sleep | ✅ | interval + jitter percentage |
| killdate | ✅ | agent expiration (epoch timestamp) |
| cp | ✅ | file/directory copy (recursive) |
| mv | ✅ | move/rename |
| rm | ✅ | file/directory removal (recursive) |
| mkdir | ✅ | recursive directory creation |
| execute | ✅ | direct binary execution (no shell) |
| exit | ✅ | clean agent termination |

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
| Indirect syscalls | N/A (no inject) | Planned ([syscalls-rs](https://github.com/Nariod/syscalls-rs)) |
| String obfuscation | Hash compile-time (ROL5) | `obfstr` compile-time encryption |
| Post-exploitation | HBIN dynamic modules | Not yet |
| Memory safety | Manual (C) | Compiler-enforced (Rust) |
| Unit tests | None | 9 tests (Go + Rust) |

---

## Known limitations

- macOS cross-compilation requires osxcross (not included in Dockerfile).
- No AMSI/ETW bypass, no indirect syscalls yet (see roadmap).
- `inject` uses Win32 APIs directly (detectable by EDR user-mode hooks).
- `upload` requires the Mythic web UI modal (not testable via GraphQL API alone).
- Binary size gap with pure-C agents like Hannibal (1.9 MB vs 25-45 KB).

---

## Build and container notes

The payload type container embeds the Rust toolchain and cross-compilation targets. In normal Mythic usage, no additional host-side Rust setup is necessary beyond Docker and Mythic.

Container prerequisites baked into the image:

```dockerfile
FROM golang:1.25 AS go-builder
# ... builds Go payload type service

FROM rust:latest
RUN apt-get install -y musl-tools mingw-w64 clang lld binutils
RUN rustup target add x86_64-unknown-linux-musl x86_64-pc-windows-gnu
```

### Running outside Docker (local development)

```bash
AGENT_CODE_DIR=/path/to/linky-mythic/agent_code \
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
├── main.go                       # Mythic container entry point
├── go.mod                        # Go 1.25, MythicContainer v1.6.4
├── Dockerfile                    # Multi-stage: Go builder + Rust toolchain
├── config.json                   # Mythic payload type configuration
├── agent_capabilities.json       # Capability summary
├── mythic/
│   └── agent_functions/
│       ├── builder.go            # Build orchestration + AES callback encryption
│       ├── shell.go ... exit.go  # 21 command definitions (Go ↔ Mythic)
│       └── utils.go              # Shared helpers (splitArgs)
└── agent_code/                   # Rust workspace
    ├── Cargo.toml                # Workspace: release profile (LTO, strip, opt-z)
    └── links/
        ├── common/
        │   ├── Cargo.toml        # ureq 3, obfstr, aes+cbc+hmac+sha2, zeroize
        │   └── src/
        │       ├── lib.rs        # C2 loop, crypto, file transfers, HTTP client
        │       └── dispatch.rs   # Cross-platform command dispatch
        ├── linux/                # Linux-specific: /proc parsing, hostname fallback
        ├── windows/              # Windows-specific: inject, integrity, tasklist
        └── osx/                  # macOS-specific: shell fallbacks for ps/netstat
```

---

## Development validation commands

```bash
# Go (from project root)
go build ./... && go vet ./...

# Rust workspace (from agent_code/)
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
| Indirect syscalls (Windows inject) | ⬜ Planned ([syscalls-rs](https://github.com/Nariod/syscalls-rs)) |
| Sleep obfuscation (Windows) | ⬜ Research |
| AMSI/ETW bypass (Windows) | ⬜ Planned |
| Conditional command compilation (Cargo features) | ⬜ Planned |

---

## Roadmap

See [TODO.md](TODO.md) for the detailed phase-by-phase plan.

### Near-term
- Mythic `process_browser` and `file_browser` structured JSON output
- Configurable User-Agent via build parameter
- `ipinfo` command (network interface info)
- Conditional command compilation via Cargo features (operator picks commands at build time)
- CI pipeline (replace stub test scripts with GitHub Actions)

### Medium-term
- Indirect syscalls for Windows inject via [syscalls-rs](https://github.com/Nariod/syscalls-rs)
- Sleep obfuscation research (Windows — Ekko-style)
- Full macOS support with osxcross in Dockerfile
- ARM64 targets (`aarch64-unknown-linux-musl`, `aarch64-apple-darwin`)

### Long-term
- Dynamic module loading (Rust equivalent of Hannibal's HBIN)
- SOCKS proxy for network pivoting
- AMSI/ETW bypass (Windows)
- Hugo documentation site
