#!/bin/bash

echo "=== Testing aigpt-rs shell functionality ==="
echo

echo "1. Testing shell command with help:"
echo "help" | cargo run --bin aigpt-rs -- shell test_user --provider ollama --model qwen2.5-coder:latest
echo

echo "2. Testing basic commands:"
echo -e "!pwd\n!ls\nexit" | cargo run --bin aigpt-rs -- shell test_user --provider ollama --model qwen2.5-coder:latest
echo

echo "3. Testing AI commands:"
echo -e "/status\n/fortune\nexit" | cargo run --bin aigpt-rs -- shell test_user --provider ollama --model qwen2.5-coder:latest
echo

echo "=== Shell tests completed ==="