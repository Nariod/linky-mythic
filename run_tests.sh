#!/bin/bash

# Script to run automated tests for linky-mythic

echo "Running automated tests for linky-mythic..."

# Build Rust implants
cd agent_code/links || { echo "Agent code directory not found"; exit 1; }

echo "Building Rust implants..."
cargo build --release || { echo "Failed to build Rust implants"; exit 1; }

# Run integration tests
cd ../.. || exit 1

echo "Running integration tests..."
./test_integration.sh || { echo "Integration tests failed"; exit 1; }

echo "All tests completed successfully."
