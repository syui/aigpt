#!/bin/bash

echo "=== Testing aigpt-rs shell manually ==="
echo

# Test with echo to simulate input
echo "Testing with simple command..."
echo "/status" | timeout 10 cargo run --bin aigpt-rs -- shell syui --provider ollama --model qwen2.5-coder:latest
echo "Exit code: $?"
echo

echo "Testing with help command..."
echo "help" | timeout 10 cargo run --bin aigpt-rs -- shell syui --provider ollama --model qwen2.5-coder:latest
echo "Exit code: $?"
echo

echo "Testing with AI message..."
echo "Hello AI" | timeout 10 cargo run --bin aigpt-rs -- shell syui --provider ollama --model qwen2.5-coder:latest
echo "Exit code: $?"
echo

echo "=== Manual shell tests completed ==="