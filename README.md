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

**Beta.** The codebase has been **validated against a live Mythic instance** (April 2026):

- ✅ Payload type registers and syncs with Mythic via RabbitMQ
- ✅ HTTP C2 profile integration works
- ✅ Linux payload builds successfully through Mythic's build pipeline
- ✅ All unit tests pass (Go build + 7 Rust tests)

That means this repository should currently be treated as:

- functional for lab work, red team exercises, and experimentation
- promising, but still maturing — review before operational use
- `upload` and `download` still need Mythic file-store integration (see TODO.md Phase 7)

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
2. Use the `http` C2 profile.
3. Set the callback values carefully.

Recommended HTTP profile values:

- `callback_host`: `https://<your-server-ip-or-domain>`
- `callback_port`: `443` (or your chosen TLS port)
- `callback_interval`: `10`
- `callback_jitter`: `23`

**Important:** the implant supports **HTTPS only**. If `callback_host` does not start with `https://`, the payload will fail at runtime.

---

## Supported capabilities

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

## Known limitations

- `upload` is currently stubbed on all platforms (Mythic file-store API not yet integrated).
- `inject` and `integrity` are Windows-only.
- macOS cross-compilation requires osxcross (not included in the Dockerfile).
- No AMSI/ETW bypass, no indirect syscalls, no string obfuscation (see Phase 8 in TODO.md).

If you plan to use this project beyond a local lab, assume additional testing and review are required first.

---

## How it fits into Mythic

Mythic provides the core platform:

- web UI
- PostgreSQL
- RabbitMQ
- GraphQL / WebSocket services

`linky-mythic` runs as a separate payload type container connected to Mythic. Its main job is to define the agent, expose the commands, and build the Rust implant binaries on demand.

```text
Mythic Core                          linky-mythic
───────────────────────────          ─────────────────────────
Web UI                               Payload Type Container
PostgreSQL            ───────►       Defines the "linky" agent
RabbitMQ              ◄───────       Receives build requests
HTTP C2 profile                       Invokes cargo build
```

---

## linky-mythic vs original Linky

| Component | Original Linky | linky-mythic |
|-----------|----------------|--------------|
| Operator interface | Custom CLI | Mythic web UI |
| Backend / C2 server | Custom Rust server | Mythic + HTTP profile |
| Database | None | Mythic PostgreSQL |
| Implant language | Rust | Rust |
| Multi-operator support | No | Yes |
| Protocol | Custom | Mythic protocol |

In short: this project keeps the Rust implant approach from Linky, but adapts it to run as a Mythic agent instead of a standalone framework.

---

## Build and container notes

The payload type container embeds the Rust toolchain and cross-compilation targets it needs. In normal Mythic usage, no additional host-side Rust setup should be necessary beyond Docker and Mythic itself.

Build flow:

`Mythic UI` -> `payload build request` -> `builder.go` -> `cargo build` -> artifact returned to Mythic

Container prerequisites baked into the image:

```dockerfile
FROM rust:latest
RUN apt-get install -y musl-tools mingw-w64 binutils
RUN rustup target add x86_64-unknown-linux-musl x86_64-pc-windows-gnu
```

### Running outside Docker (local development)

When running the payload container outside Docker (e.g. for local testing), set the
`AGENT_CODE_DIR` environment variable to point to the `agent_code/` directory:

```bash
AGENT_CODE_DIR=/path/to/linky-mythic/agent_code \
RABBITMQ_HOST=127.0.0.1 \
RABBITMQ_PASSWORD=<from .env> \
MYTHIC_SERVER_HOST=127.0.0.1 \
./linky-container
```

The builder falls back to `/Mythic/agent_code` when `AGENT_CODE_DIR` is not set (the default path inside the Docker container).

---

## Project layout

```text
linky-mythic/
├── main.go                  # Mythic container entry point
├── Dockerfile               # Go + Rust build environment
├── mythic/                  # Payload type definition and command metadata
├── agent_code/              # Rust implant workspace
├── config.json              # Mythic payload type configuration
├── agent_capabilities.json  # Capability summary
└── README.md
```

Main directories:

- `mythic/`: Go code for payload registration, command definitions, and build integration
- `agent_code/`: Rust implant code for Linux, Windows, macOS, and shared protocol / dispatch logic

---

## Development validation commands

```bash
# Rust workspace (from agent_code/)
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ cargo check --workspace
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ cargo test --workspace
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ cargo clippy --workspace -- -D warnings
cargo fmt --check

# Go (from project root)
go mod tidy
go build ./...
go vet ./...
```

---

## Security notice

This repository should be treated as experimental offensive security software. Use it only in environments you own or where you have explicit written authorization.

Current OPSEC posture:

- No AMSI/ETW bypass
- No indirect syscalls
- No string obfuscation
- AES key derived from `PayloadUUID` is still a weak-entropy design
- Mythic provides multi-operator support, audit trail, and structured logging

---

## Roadmap

- Replace stubbed file transfer behavior with proper Mythic file-store integration
- Improve OPSEC hardening (string obfuscation, key zeroization, indirect syscalls)
- Implement Mythic `process_browser` and `file_browser` callbacks for structured output
- Expand documentation and operational guidance
- Build a functional CI pipeline with Docker-in-Docker and Mythic
