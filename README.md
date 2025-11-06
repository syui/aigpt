# aigpt

AI memory system with psychological analysis for Claude via MCP.

**Current: Layers 1-4 Complete** - Memory storage, AI interpretation, personality analysis, integrated profile, and relationship inference.

## Features

### Layer 1: Pure Memory Storage
- 🗄️ **SQLite Storage**: Reliable database with ACID guarantees
- 🔖 **ULID IDs**: Time-sortable, 26-character unique identifiers
- 🔍 **Search**: Fast content-based search
- 📝 **CRUD Operations**: Complete memory management

### Layer 2: AI Memory
- 🧠 **AI Interpretation**: Claude interprets and evaluates memories
- 📊 **Priority Scoring**: Importance ratings (0.0-1.0)
- 🎯 **Smart Storage**: Memory + evaluation in one step

### Layer 3: Personality Analysis
- 🔬 **Big Five Model**: Scientifically validated personality assessment
- 📈 **Pattern Recognition**: Analyzes memory patterns to build user profile
- 💾 **Historical Tracking**: Save and compare analyses over time

### Layer 3.5: Integrated Profile
- 🎯 **Essential Summary**: Unified view of personality, interests, and values
- 🤖 **AI-Optimized**: Primary tool for AI to understand the user
- ⚡ **Smart Caching**: Auto-updates only when necessary
- 🔍 **Flexible Access**: Detailed data still accessible when needed

### Layer 4: Relationship Inference (Optional)
- 🤝 **Relationship Tracking**: Track interactions with entities (people, characters, etc.)
- 📊 **Bond Strength**: Infer relationship strength from memory patterns
- 🎮 **Game Ready**: Foundation for companion apps, games, VTubers
- 🔒 **Opt-in**: Enable only when needed with `--enable-layer4` flag

### General
- 🛠️ **MCP Integration**: Works seamlessly with Claude Code
- 🧪 **Well-tested**: Comprehensive test coverage
- 🚀 **Simple & Fast**: Minimal dependencies, pure Rust

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

## MCP Tools

### Layer 1: Basic Memory (6 tools)
- `create_memory` - Simple memory creation
- `get_memory` - Retrieve by ID
- `list_memories` - List all memories
- `search_memories` - Content-based search
- `update_memory` - Update existing memory
- `delete_memory` - Remove memory

### Layer 2: AI Memory (1 tool)
- `create_ai_memory` - Create with AI interpretation and priority score

### Layer 3: Personality Analysis (2 tools)
- `save_user_analysis` - Save Big Five personality analysis
- `get_user_analysis` - Retrieve latest personality profile

### Layer 3.5: Integrated Profile (1 tool)
- `get_profile` - **Primary tool**: Get integrated user profile with essential summary

### Layer 4: Relationship Inference (2 tools, requires `--enable-layer4`)
- `get_relationship` - Get inferred relationship with specific entity
- `list_relationships` - List all relationships sorted by bond strength

## Usage Examples in Claude Code

### Layer 1: Simple Memory
```
Remember that the project deadline is next Friday.
```
Claude will use `create_memory` automatically.

### Layer 2: AI Memory with Evaluation
```
create_ai_memory({
  content: "Designed a new microservices architecture",
  ai_interpretation: "Shows technical creativity and strategic thinking",
  priority_score: 0.85
})
```

### Layer 3: Personality Analysis
```
# After accumulating memories, analyze personality
save_user_analysis({
  openness: 0.8,
  conscientiousness: 0.7,
  extraversion: 0.4,
  agreeableness: 0.65,
  neuroticism: 0.3,
  summary: "High creativity and planning ability, introverted personality"
})

# Retrieve analysis
get_user_analysis()
```

### Layer 3.5: Integrated Profile (Recommended)
```
# Get essential user profile - AI's primary tool
get_profile()

# Returns:
{
  "dominant_traits": [
    {"name": "openness", "score": 0.8},
    {"name": "conscientiousness", "score": 0.7},
    {"name": "extraversion", "score": 0.4}
  ],
  "core_interests": ["Rust", "architecture", "design", "system", "memory"],
  "core_values": ["simplicity", "efficiency", "maintainability"],
  "key_memory_ids": ["01H...", "01H...", ...],
  "data_quality": 0.85
}
```

**Usage Pattern:**
- AI normally uses `get_profile()` to understand the user
- For specific details, AI can call `get_memory(id)`, `list_memories()`, etc.
- Profile auto-updates when needed (10+ memories, new analysis, or 7+ days)

### Layer 4: Relationship Inference (Optional, requires `--enable-layer4`)
```
# Create memories with entity tracking
Memory::new_with_entities({
  content: "Had lunch with Alice",
  ai_interpretation: "Pleasant social interaction",
  priority_score: 0.7,
  related_entities: ["alice"]
})

# Get relationship inference
get_relationship({ entity_id: "alice" })

# Returns:
{
  "entity_id": "alice",
  "interaction_count": 15,
  "avg_priority": 0.75,
  "days_since_last": 2,
  "bond_strength": 0.82,
  "relationship_type": "close_friend",
  "confidence": 0.80
}

# List all relationships
list_relationships({ limit: 5 })
```

**Relationship Types:**
- `close_friend` (0.8+): Very strong bond
- `friend` (0.6-0.8): Strong connection
- `valued_acquaintance` (0.4-0.6, high priority): Important but not close
- `acquaintance` (0.4-0.6): Regular contact
- `regular_contact` (0.2-0.4): Occasional interaction
- `distant` (<0.2): Minimal connection

**Starting the Server:**
```bash
# Normal mode (Layer 1-3.5 only)
aigpt server

# With relationship features (Layer 1-4)
aigpt server --enable-layer4
```

## Big Five Personality Traits

- **Openness**: Creativity, curiosity, openness to new experiences
- **Conscientiousness**: Organization, planning, reliability
- **Extraversion**: Social energy, assertiveness, outgoingness
- **Agreeableness**: Cooperation, empathy, kindness
- **Neuroticism**: Emotional stability (low = stable, high = sensitive)

Scores range from 0.0 to 1.0, where higher scores indicate stronger trait expression.

## Storage Location

All data stored in: `~/.config/syui/ai/gpt/memory.db`

## Architecture

Multi-layer system design:

- **Layer 1** ✅ Complete: Pure memory storage (with entity tracking)
- **Layer 2** ✅ Complete: AI interpretation with priority scoring
- **Layer 3** ✅ Complete: Big Five personality analysis
- **Layer 3.5** ✅ Complete: Integrated profile (unified summary)
- **Layer 4** ✅ Complete: Relationship inference (optional, `--enable-layer4`)
- **Layer 4+** 🔵 Future: Extended game/companion features
- **Layer 5** 🔵 Future: Distribution and sharing

**Design Philosophy**:
- **"Internal complexity, external simplicity"**: Simple API, complex internals
- **"AI judges, tool records"**: AI makes decisions, tool stores data
- **Layered architecture**: Each layer independent but interconnected
- **Optional features**: Core layers always active, advanced layers opt-in

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - Multi-layer system design
- [Layer 1 Details](docs/LAYER1.md) - Technical details of memory storage
- [Old Versions](docs/archive/old-versions/) - Previous documentation

## Development

```bash
# Run tests
cargo test

# Build for release
cargo build --release

# Run with verbose logging
RUST_LOG=debug aigpt server
```

## Design Philosophy

**"AI evolves, tools don't"** - This tool provides simple, reliable storage while AI (Claude) handles interpretation, evaluation, and analysis. The tool focuses on being maintainable and stable.

## License

MIT

## Author

syui
