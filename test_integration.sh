#!/bin/bash

# Script to perform integration tests for linky-mythic

echo "Running integration tests..."

# Define the Mythic server URL (adjust as needed)
MYTHIC_SERVER="http://localhost:8080"

# Test commands
COMMANDS=("ls" "cd" "pwd" "shell")

# Function to test a command
test_command() {
    local command=$1
    echo "Testing command: $command"
    
    # Simulate sending a command to Mythic and checking the response
    # Replace with actual curl command or test logic
    response=$(curl -s -X POST "$MYTHIC_SERVER/api/v1.4/tasks" \
        -H "Content-Type: application/json" \
        -d "{\"command\": \"$command\"}")
    
    if [ -z "$response" ]; then
        echo "Error: No response for command $command"
        return 1
    fi
    
    echo "Response for $command: $response"
    return 0
}

# Run tests for each command
for cmd in "${COMMANDS[@]}"; do
    test_command "$cmd" || { echo "Test failed for command: $cmd"; exit 1; }
done

echo "All integration tests passed."
