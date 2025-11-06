# aigpt

Simple memory storage for Claude with MCP support.

**Layer 1: Pure Memory Storage** - A clean, SQLite-based memory system with ULID identifiers.

## Features

- 🗄️ **SQLite Storage**: Reliable database with ACID guarantees
- 🔖 **ULID IDs**: Time-sortable, 26-character unique identifiers
- 🔍 **Search**: Fast content-based search
- 🛠️ **MCP Integration**: Works seamlessly with Claude Code
- 🧪 **Well-tested**: Comprehensive test coverage

## Quick Start

### Installation

```bash
# Build
cargo build --release

# Install (optional)
cp target/release/aigpt ~/.cargo/bin/
```

### CLI Usage

```bash
# Create a memory
aigpt create "Remember this information"

# List all memories
aigpt list

# Search memories
aigpt search "keyword"

# Show statistics
aigpt stats
```

### MCP Integration with Claude Code

```bash
# Add to Claude Code
claude mcp add aigpt /path/to/aigpt/target/release/aigpt server
```

Then use in Claude Code:
- "Remember that tomorrow will be sunny"
- "Search for weather information"
- "Show all my memories"

## Storage Location

Memories are stored in: `~/.config/syui/ai/gpt/memory.db`

## Architecture

This is **Layer 1** of a planned multi-layer system:

- **Layer 1** (Current): Pure memory storage
- **Layer 2** (Planned): AI interpretation with priority scoring
- **Layer 3** (Planned): User evaluation and diagnosis
- **Layer 4** (Planned): Game systems and companion features

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Documentation

- [Layer 1 Details](docs/LAYER1.md) - Technical details of current implementation
- [Architecture](docs/ARCHITECTURE.md) - Multi-layer system design

## Development

```bash
# Run tests
cargo test

# Build for release
cargo build --release

# Run with verbose logging
RUST_LOG=debug aigpt server
```

## License

MIT

## Author

syui
