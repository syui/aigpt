# Architecture: Multi-Layer Memory System

## Design Philosophy

aigptは、独立したレイヤーを積み重ねる設計です。各レイヤーは：

- **独立性**: 単独で動作可能
- **接続性**: 他のレイヤーと連携可能
- **段階的**: 1つずつ実装・テスト

## Layer Overview

```
┌─────────────────────────────────────────┐
│  Layer 5: Distribution & Sharing       │  Future
│  (Game streaming, public/private)      │
├─────────────────────────────────────────┤
│  Layer 4b: AI Companion                │  Future
│  (Romance system, personality growth)   │
├─────────────────────────────────────────┤
│  Layer 4a: Game Systems                │  Future
│  (Ranking, rarity, XP, visualization)   │
├─────────────────────────────────────────┤
│  Layer 3: User Evaluation              │  Future
│  (Personality diagnosis from patterns)  │
├─────────────────────────────────────────┤
│  Layer 2: AI Memory                    │  Future
│  (Claude interpretation, priority_score)│
├─────────────────────────────────────────┤
│  Layer 1: Pure Memory Storage          │  ✅ Current
│  (SQLite, ULID, CRUD operations)       │
└─────────────────────────────────────────┘
```

## Layer 1: Pure Memory Storage (Current)

**Status**: ✅ **Implemented & Tested**

### Purpose
正確なデータの保存と参照。シンプルで信頼できる基盤。

### Technology Stack
- **Database**: SQLite with ACID guarantees
- **IDs**: ULID (time-sortable, 26 chars)
- **Language**: Rust with thiserror/anyhow
- **Protocol**: MCP (Model Context Protocol) via stdio

### Data Model
```rust
pub struct Memory {
    pub id: String,              // ULID
    pub content: String,         // User content
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Operations
- `create()` - Insert new memory
- `get(id)` - Retrieve by ID
- `update()` - Update existing memory
- `delete(id)` - Remove memory
- `list()` - List all (sorted by created_at DESC)
- `search(query)` - Content-based search
- `count()` - Total count

### File Structure
```
src/
├── core/
│   ├── error.rs    - Error types (thiserror)
│   ├── memory.rs   - Memory struct
│   ├── store.rs    - SQLite operations
│   └── mod.rs      - Module exports
├── mcp/
│   ├── base.rs     - MCP server
│   └── mod.rs      - Module exports
├── lib.rs          - Library root
└── main.rs         - CLI application
```

### Storage
- Location: `~/.config/syui/ai/gpt/memory.db`
- Schema: Single table with indexes on timestamps
- No migrations (fresh start for Layer 1)

---

## Layer 2: AI Memory (Planned)

**Status**: 🔵 **Planned**

### Purpose
Claudeが記憶内容を解釈し、重要度を評価。

### Extended Data Model
```rust
pub struct AIMemory {
    // Layer 1 fields
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Layer 2 additions
    pub interpreted_content: String,  // Claude's interpretation
    pub priority_score: f32,          // 0.0 - 1.0
    pub psychological_factors: PsychologicalFactors,
}

pub struct PsychologicalFactors {
    pub emotional_weight: f32,    // 0.0 - 1.0
    pub personal_relevance: f32,  // 0.0 - 1.0
    pub novelty: f32,             // 0.0 - 1.0
    pub utility: f32,             // 0.0 - 1.0
}
```

### MCP Tools (Additional)
- `create_memory_with_ai` - Create with Claude interpretation
- `reinterpret_memory` - Re-evaluate existing memory
- `get_high_priority` - Get memories above threshold

### Implementation Strategy
- Feature flag: `--features ai-memory`
- Backward compatible with Layer 1
- Claude Code does interpretation (no external API)

---

## Layer 3: User Evaluation (Planned)

**Status**: 🔵 **Planned**

### Purpose
メモリパターンからユーザーの性格を診断。

### Diagnosis Types
```rust
pub enum DiagnosisType {
    Innovator,      // 革新者
    Philosopher,    // 哲学者
    Pragmatist,     // 実用主義者
    Explorer,       // 探検家
    Protector,      // 保護者
    Visionary,      // 未来志向
}
```

### Analysis
- Memory content patterns
- Priority score distribution
- Creation frequency
- Topic diversity

### MCP Tools (Additional)
- `diagnose_user` - Run personality diagnosis
- `get_user_profile` - Get analysis summary

---

## Layer 4a: Game Systems (Planned)

**Status**: 🔵 **Planned**

### Purpose
ゲーム的要素で記憶管理を楽しく。

### Features
- **Rarity Levels**: Common → Uncommon → Rare → Epic → Legendary
- **XP System**: Memory creation earns XP
- **Rankings**: Based on total priority score
- **Visualization**: Game-style output formatting

### Data Additions
```rust
pub struct GameMemory {
    // Previous layers...
    pub rarity: RarityLevel,
    pub xp_value: u32,
    pub discovered_at: DateTime<Utc>,
}
```

---

## Layer 4b: AI Companion (Planned)

**Status**: 🔵 **Planned**

### Purpose
育成可能な恋愛コンパニオン。

### Features
- Personality types (Tsundere, Kuudere, Genki, etc.)
- Relationship level (0-100)
- Memory-based interactions
- Growth through conversations

### Data Model
```rust
pub struct Companion {
    pub id: String,
    pub name: String,
    pub personality: CompanionPersonality,
    pub relationship_level: u8,  // 0-100
    pub memories_shared: Vec<String>,
    pub last_interaction: DateTime<Utc>,
}
```

---

## Layer 5: Distribution (Future)

**Status**: 🔵 **Future Consideration**

### Purpose
ゲーム配信や共有機能。

### Ideas
- Share memory rankings
- Export as shareable format
- Public/private memory modes
- Integration with streaming platforms

---

## Implementation Strategy

### Phase 1: Layer 1 ✅ (Complete)
- [x] Core memory storage
- [x] SQLite integration
- [x] MCP server
- [x] CLI interface
- [x] Tests
- [x] Documentation

### Phase 2: Layer 2 (Next)
- [ ] Add AI interpretation fields to schema
- [ ] Implement priority scoring logic
- [ ] Create `create_memory_with_ai` tool
- [ ] Update MCP server
- [ ] Write tests for AI features

### Phase 3: Layers 3-4 (Future)
- [ ] User diagnosis system
- [ ] Game mechanics
- [ ] Companion system

### Phase 4: Layer 5 (Future)
- [ ] Sharing mechanisms
- [ ] Public/private modes

## Design Principles

1. **Simplicity First**: Each layer adds complexity incrementally
2. **Backward Compatibility**: New layers don't break old ones
3. **Feature Flags**: Optional features via Cargo features
4. **Independent Testing**: Each layer has its own test suite
5. **Clear Boundaries**: Layers communicate through defined interfaces

## Technology Choices

### Why SQLite?
- ACID guarantees
- Better querying than JSON
- Built-in indexes
- Single-file deployment
- No server needed

### Why ULID?
- Time-sortable (unlike UUID v4)
- Lexicographically sortable
- 26 characters (compact)
- No collision concerns

### Why Rust?
- Memory safety
- Performance
- Excellent error handling
- Strong type system
- Great tooling (cargo, clippy)

### Why MCP?
- Standard protocol for AI tools
- Works with Claude Code/Desktop
- Simple stdio-based communication
- No complex networking

## Future Considerations

### Potential Enhancements
- Full-text search (SQLite FTS5)
- Tag system
- Memory relationships/links
- Export/import functionality
- Multiple databases
- Encryption for sensitive data

### Scalability
- Layer 1: Handles 10K+ memories easily
- Consider pagination for Layer 4 (UI display)
- Indexing strategy for search performance

## Development Guidelines

### Adding a New Layer

1. **Design**: Document data model and operations
2. **Feature Flag**: Add to Cargo.toml
3. **Schema**: Extend database schema (migrations)
4. **Implementation**: Write code in new module
5. **Tests**: Comprehensive test coverage
6. **MCP Tools**: Add new MCP tools if needed
7. **Documentation**: Update this file

### Code Organization

```
src/
├── core/           # Layer 1: Pure storage
├── ai/             # Layer 2: AI features (future)
├── evaluation/     # Layer 3: User diagnosis (future)
├── game/           # Layer 4a: Game systems (future)
├── companion/      # Layer 4b: Companion (future)
└── mcp/            # MCP server (all layers)
```

---

**Version**: 0.2.0
**Last Updated**: 2025-11-05
**Current Layer**: 1
