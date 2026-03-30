#!/bin/bash

# Script to set up the Mythic test environment using Docker

echo "Setting up Mythic test environment..."

# Navigate to the Mythic directory (adjust path as needed)
cd mythic || { echo "Mythic directory not found"; exit 1; }

# Start Mythic containers
echo "Starting Mythic containers..."
docker-compose up -d

# Wait for containers to be ready
sleep 30

echo "Mythic test environment is ready."
