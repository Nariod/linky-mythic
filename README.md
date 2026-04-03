# linky-mythic

A Mythic C2 payload type providing Rust-native implants for Linux, Windows, and macOS.

> **Linky reimagined as a Mythic agent** — same Rust implants, Mythic handles GUI / backend / DB.

---

## What this project is

`linky-mythic` is a **Mythic Payload Type**: it installs into an existing Mythic instance and adds the ability to generate, deploy, and control Rust implants across platforms.

This project does NOT provide a C2 server, a GUI, or a database. **Mythic provides all of that.** This project provides only the implants and their definition container.

```bash
sudo ./mythic-cli install github https://github.com/Nariod/linky-mythic
```

---

## Current status

**Alpha — not yet tested against a live Mythic instance.**

Phases 0–5c (project structure, Rust implant migration, wire format, Go builder, command
definitions, HTTPS configuration, and dispatch unification) are complete. The code compiles and passes all tests, but has not been validated against a live Mythic instance.

Key completed fixes:
- ✅ `build_client()` now uses `.danger_accept_invalid_certs(true)` and includes User-Agent and timeout.
- ✅ `run_c2_loop()` defensively handles URL schemes to avoid double `https://` prefixes.
- ✅ Go builder correctly parses `PayloadUUID` and handles debug build paths.
- ✅ Dispatch architecture unified across Linux/Windows/macOS using `dispatch_common(command, parameters)`.
- ✅ `TaskResponse` sets `status: "error"` when a command fails (output starts with "[-]").
- ✅ Unused dependencies (`obfstr`) removed from Cargo.toml files.
- ✅ `extract_param` fallback returns `""` instead of raw JSON when the key is absent.
- ✅ `derive_key` simplified using `.into()`.

Key blocking issues (to be validated in Phase 6):
- `MythicEncryptsData` setting may cause double encryption — requires live Mythic testing.
- End-to-end command testing pending.

---

## Mythic architecture — overview

Mythic is a modular C2 framework. The core server provides:
- React Web UI (multi-operator)
- PostgreSQL (persistence, audit logs)
- RabbitMQ (message bus between containers)
- GraphQL API + WebSockets

Agents and C2 profiles are **separate Docker containers** that connect to the core via RabbitMQ.

```
Mythic Core (docker-compose)           linky-mythic (this repo)
─────────────────────────────          ──────────────────────────
React UI                               Payload Type Container (Go)
   │                                     ├── defines "linky" as an agent
PostgreSQL ──── RabbitMQ ◄──────────────┤── receives build requests
   │                │                    └── invokes cargo build
GraphQL API     gRPC server
```

---

## This project vs original Linky

| Component | Linky (original) | linky-mythic |
|-----------|-----------------|--------------|
| GUI / CLI | Custom rustyline CLI | Mythic React UI |
| C2 server | actix-web (Rust) | Mythic HTTP profile |
| Database | None (in-memory) | PostgreSQL (Mythic) |
| Multi-operator | No | Yes |
| Wire protocol | Custom 3-stage | Mythic standard |
| Implants | Rust | Rust (adapted) |
| On-the-fly build | cargo in container | cargo in container |
| Encryption | Custom AES-256-GCM | AES-256-GCM (Mythic format) |

**What we keep**: all Rust implant code (`links/linux`, `links/windows`, `links/osx`, `links/common`), dispatch logic, and all capabilities (shell, download, upload, injection...).

**What we rewrite**: `run_c2_loop` in `links/common/src/lib.rs` — to speak the Mythic protocol instead of the custom 3-stage protocol.

**What we drop**: the entire `server/` crate (actix-web, routes, CLI, generate, UI). Mythic replaces all of it.

---

## Project structure

```
linky-mythic/
├── main.go                       # Entry point: StartAndRunForever
├── go.mod                        # module linky, go 1.21
├── Dockerfile                    # Multi-stage: Go builder + Rust toolchain
├── mythic/
│   ├── payload_type.go           # Agent metadata (OS, arch, build params)
│   └── agent_functions/          # Command definitions (Go)
│       ├── builder.go            # build() → invokes cargo build
│       ├── shell.go              # Canonical command template
│       ├── ls.go, cd.go, ...     # One file per command
│       └── exit.go               # ✅ Implemented
├── agent_code/                   # Rust implants (Mythic-adapted)
│   ├── Cargo.toml                # Workspace (linux, windows, osx, common)
│   └── links/
│       ├── common/               # Mythic protocol, crypto, dispatch, helpers
│       │   └── src/
│       │       ├── lib.rs        # run_c2_loop, AES-256-GCM, wire format
│       │       └── dispatch.rs   # Cross-platform command dispatch
│       ├── linux/                # Linux implant (native ps/netstat parsing)
│       ├── windows/              # Windows implant (injection, integrity level)
│       └── osx/                  # macOS implant (shell fallback for ps/netstat)
├── agent_capabilities.json       # mythicmeta.github.io/overview matrix
├── config.json                   # mythic-cli config
├── TODO.md                       # Detailed migration plan + audit
└── README.md
```

---

## Installation

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

## Build prerequisites (inside the container)

The Payload Type `Dockerfile` embeds the full Rust toolchain and cross-compilation targets. No host dependencies are needed beyond Docker.

```dockerfile
FROM rust:latest
RUN apt-get install -y musl-tools mingw-w64 binutils
RUN rustup target add x86_64-unknown-linux-musl x86_64-pc-windows-gnu
```

Mythic calls `builder.go` → `cargo build` → the binary is delivered to the operator via the UI.

---

## Agent capabilities

| Command | Linux | Windows | macOS |
|---------|-------|---------|-------|
| shell | ✅ | ✅ | ✅ |
| ls / cd / pwd | ✅ | ✅ | ✅ |
| whoami / pid | ✅ | ✅ | ✅ |
| info | ✅ | ✅ | ✅ |
| ps | ✅ | ✅ | ✅ (shell fallback) |
| netstat | ✅ | ✅ | ✅ (shell fallback) |
| download | ✅ | ✅ | ✅ |
| upload | ⬜ stub | ⬜ stub | ⬜ stub |
| sleep / jitter | ✅ | ✅ | ✅ |
| killdate | ✅ | ✅ | ✅ |
| exit | ✅ (Rust) | ✅ (Rust) | ✅ (Rust) |
| inject | — | ✅ | — |
| integrity | — | ✅ | — |

---

## Roadmap

| Phase | Focus | Status |
|-------|-------|--------|
| 0 | Project structure, config.json, Dockerfile, Go stubs | ✅ Done |
| 1 | Rust implants migration (dispatch, linux, windows, osx, common) | ✅ Done |
| 2 | Wire format cleanup (hex → direct AES-GCM, unit tests) | ✅ Done |
| 3 | Go builder: `encryptCallback` AES-GCM + `PAYLOAD_UUID` wiring | ✅ Done |
| 4 | Go command definitions: individual files, `extract_param` in Rust | ✅ Done |
| 5 | **Critical bugfixes** (TLS, URL, reqwest feature, CALLBACK_URI) | ✅ Done |
| 5b | **Go builder bugfixes** (PayloadUUID type, debug path, params) | ✅ Done |
| 5c | **Dispatch unification** (refactor dispatch_common, error status) | ✅ Done |
| 6 | End-to-end test against a live Mythic instance | ⬜ Planned |
| 7 | Download/Upload via Mythic file store (native file transfer) | ⬜ Planned |
| 8 | OPSEC hardening (obfstr, key strength, zeroize, anti-panic) | ⬜ Planned |
| 9 | Process/file browser, Hugo docs, CI pipeline | ⬜ Planned |

---

## Mythic HTTP profile configuration

When generating a payload in the Mythic UI, the operator must configure the `http` C2 profile:

- `callback_host`: `https://<your-server-ip-or-domain>` (must start with `https://`)
- `callback_port`: `443` (or your chosen port)
- `callback_interval`: default `10` (seconds)
- `callback_jitter`: default `23` (percent)

The implant only supports HTTPS. Any HTTP (non-TLS) callback host will fail at runtime.

---

## Validation commands

```bash
# Rust workspace (from agent_code/)
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x cargo check --workspace
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x cargo test --workspace
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x cargo clippy --workspace -- -D warnings
cargo fmt --check

# Go (from project root)
go mod tidy
go build ./...
go vet ./...
```

---

## Security notice

This tool is for **authorized** penetration testing engagements only. Do not use it against systems without explicit written permission.

Current OPSEC posture (alpha):
- No AMSI/ETW bypass
- No indirect syscalls
- No string obfuscation (planned Phase 8)
- AES key derived from PayloadUUID (weak entropy — planned Phase 8)
- Mythic provides multi-operator support, audit trail, and structured logging