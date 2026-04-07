#!/bin/bash

# Script to run automated tests for linky-mythic

set -euo pipefail

echo "Running automated tests for linky-mythic..."

# ── Rust ──────────────────────────────────────────────────────────────────────

cd Payload_Type/linky/agent_code || { echo "agent_code directory not found"; exit 1; }

echo "Checking Rust formatting..."
cargo fmt --check --all || { echo "Rust formatting check failed (run 'cargo fmt --all')"; exit 1; }

echo "Running Rust unit tests..."
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ \
    cargo test --workspace || { echo "Rust unit tests failed"; exit 1; }

cd ../.. || exit 1

# ── Go ────────────────────────────────────────────────────────────────────────

cd .. || exit 1

echo "Building Go payload type..."
go build ./... || { echo "Go build failed"; exit 1; }

echo "Running go vet..."
go vet ./... || { echo "Go vet failed"; exit 1; }

cd ../.. || exit 1

echo "All tests passed."

# Integration tests require a live Mythic environment.
# Run them separately after ./setup_test_env.sh:
#   ./test_integration.sh
