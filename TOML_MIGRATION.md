# TOML Configuration Migration Guide

## Overview

The ai.gpt Rust implementation (`aigpt-rs`) now uses TOML format for configuration instead of JSON. This provides better readability and is more idiomatic for Rust applications.

## Configuration Location

The configuration file is stored at:
- **macOS**: `~/Library/Application Support/syui/ai/gpt/config.toml`
- **Linux**: `~/.config/syui/ai/gpt/config.toml`
- **Windows**: `%APPDATA%\syui\ai\gpt\config.toml`

## Automatic Migration

When you run the Rust implementation for the first time, it will automatically:

1. Check if `config.toml` exists
2. If not, look for `config.json` in various locations:
   - `../config.json` (relative to aigpt-rs directory)
   - `config.json` (current directory)
   - `gpt/config.json` (from project root)
   - `/Users/syui/ai/ai/gpt/config.json` (absolute path)
3. If found, automatically convert the JSON to TOML format
4. Save the converted configuration to the appropriate location

## TOML Configuration Structure

```toml
# Default AI provider
default_provider = "openai"

# Provider configurations
[providers.openai]
default_model = "gpt-4o-mini"
api_key = "your-api-key-here"  # Optional, can use OPENAI_API_KEY env var
system_prompt = """
Multi-line system prompt
goes here
"""

[providers.ollama]
default_model = "qwen3"
host = "http://127.0.0.1:11434"

# AT Protocol configuration (optional)
[atproto]
host = "https://bsky.social"
handle = "your-handle.bsky.social"  # Optional
password = "your-app-password"      # Optional

# MCP (Model Context Protocol) configuration
[mcp]
enabled = true
auto_detect = true

# MCP Server definitions
[mcp.servers.ai_gpt]
base_url = "http://localhost:8001"
name = "ai.gpt MCP Server"
timeout = 10.0

# MCP endpoints
[mcp.servers.ai_gpt.endpoints]
get_memories = "/get_memories"
search_memories = "/search_memories"
# ... other endpoints ...
```

## Manual Migration

If automatic migration doesn't work, you can manually convert your `config.json`:

1. Copy the example configuration from `gpt/config.toml.example`
2. Fill in your specific values from `config.json`
3. Save it to the configuration location mentioned above

## Testing Configuration

To test if your configuration is working:

```bash
cd gpt/aigpt-rs
cargo run --bin test-config
```

This will show:
- Loaded configuration values
- Available providers
- MCP and ATProto settings
- Configuration file path

## Differences from JSON

Key differences in TOML format:
- Multi-line strings use triple quotes (`"""`)
- Comments start with `#`
- Tables (objects) use `[table.name]` syntax
- Arrays of tables use `[[array.name]]` syntax
- More readable for configuration files

## Backward Compatibility

The Python implementation still uses JSON format. Both implementations can coexist:
- Python: Uses `config.json`
- Rust: Uses `config.toml` (with automatic migration from JSON)

The Rust implementation will only perform the migration once. After `config.toml` is created, it will use that file exclusively.