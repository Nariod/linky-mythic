#!/bin/bash

# Script to run automated tests for linky-mythic

set -euo pipefail

echo "Running automated tests for linky-mythic..."

# Run Rust unit tests with required compile-time env vars
cd agent_code || { echo "agent_code directory not found"; exit 1; }

echo "Running Rust unit tests..."
CALLBACK=x IMPLANT_SECRET=x PAYLOAD_UUID=x CALLBACK_URI=/ \
    cargo test --workspace || { echo "Rust unit tests failed"; exit 1; }

cd ..

echo "All unit tests passed."

# Integration tests require a live Mythic environment.
# Run them separately after ./setup_test_env.sh:
#   ./test_integration.sh
