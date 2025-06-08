#!/bin/bash

echo "=== Testing aigpt-rs CLI commands ==="
echo

echo "1. Testing configuration loading:"
cargo run --bin test-config
echo

echo "2. Testing fortune command:"
cargo run --bin aigpt-rs -- fortune
echo

echo "3. Testing chat with Ollama:"
cargo run --bin aigpt-rs -- chat test_user "Hello from Rust!" --provider ollama --model qwen2.5-coder:latest
echo

echo "4. Testing chat with OpenAI:"
cargo run --bin aigpt-rs -- chat test_user "What's the capital of Japan?" --provider openai --model gpt-4o-mini
echo

echo "5. Testing relationships command:"
cargo run --bin aigpt-rs -- relationships
echo

echo "=== All tests completed ==="