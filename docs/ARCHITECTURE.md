# Architecture: Multi-Layer Memory System

## Design Philosophy

aigptは、独立したレイヤーを積み重ねる設計です。各レイヤーは：

- **独立性**: 単独で動作可能
- **接続性**: 他のレイヤーと連携可能
- **段階的**: 1つずつ実装・テスト

## Layer Overview

```
┌─────────────────────────────────────────┐
│  Layer 5: Distribution & Sharing       │  🔵 Future
│  (Game streaming, public/private)      │
├─────────────────────────────────────────┤
│  Layer 4b: AI Companion                │  🔵 Planned
│  (Romance system, personality growth)   │
├─────────────────────────────────────────┤
│  Layer 4a: Game Systems                │  🔵 Planned
│  (Ranking, rarity, XP, visualization)   │
├─────────────────────────────────────────┤
│  Layer 3.5: Integrated Profile         │  ✅ Complete
│  (Unified summary for AI consumption)   │
├─────────────────────────────────────────┤
│  Layer 3: User Evaluation              │  ✅ Complete
│  (Big Five personality analysis)        │
├─────────────────────────────────────────┤
│  Layer 2: AI Memory                    │  ✅ Complete
│  (Claude interpretation, priority_score)│
├─────────────────────────────────────────┤
│  Layer 1: Pure Memory Storage          │  ✅ Complete
│  (SQLite, ULID, CRUD operations)       │
└─────────────────────────────────────────┘
```

## Layer 1: Pure Memory Storage

**Status**: ✅ **Complete**

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

## Layer 2: AI Memory

**Status**: ✅ **Complete**

### Purpose
Claudeが記憶内容を解釈し、重要度を評価。人間の記憶プロセス（記憶と同時に評価）を模倣。

### Extended Data Model
```rust
pub struct Memory {
    // Layer 1 fields
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Layer 2 additions
    pub ai_interpretation: Option<String>,  // Claude's interpretation
    pub priority_score: Option<f32>,        // 0.0 - 1.0
}
```

### MCP Tools
- `create_ai_memory` - Create memory with AI interpretation and priority score
  - `content`: Memory content
  - `ai_interpretation`: Optional AI interpretation
  - `priority_score`: Optional priority (0.0-1.0)

### Philosophy
"AIは進化しますが、ツールは進化しません" - AIが判断し、ツールは記録のみ。

### Implementation
- Backward compatible with Layer 1 (Optional fields)
- Automatic schema migration from Layer 1
- Claude Code does interpretation (no external API)

---

## Layer 3: User Evaluation

**Status**: ✅ **Complete**

### Purpose
Layer 2のメモリパターンからユーザーの性格を分析。Big Five心理学モデルを使用。

### Data Model
```rust
pub struct UserAnalysis {
    pub id: String,
    pub openness: f32,              // 0.0-1.0: 創造性、好奇心
    pub conscientiousness: f32,      // 0.0-1.0: 計画性、信頼性
    pub extraversion: f32,           // 0.0-1.0: 外向性、社交性
    pub agreeableness: f32,          // 0.0-1.0: 協調性、共感性
    pub neuroticism: f32,            // 0.0-1.0: 神経質さ（低い=安定）
    pub summary: String,             // 分析サマリー
    pub analyzed_at: DateTime<Utc>,
}
```

### Big Five Model
心理学で最も信頼性の高い性格モデル（OCEAN）：
- **O**penness: 新しい経験への開かれさ
- **C**onscientiousness: 誠実性、計画性
- **E**xtraversion: 外向性
- **A**greeableness: 協調性
- **N**euroticism: 神経質さ

### Analysis Process
1. Layer 2メモリを蓄積
2. AIがパターンを分析（活動の種類、優先度の傾向など）
3. Big Fiveスコアを推測
4. 分析結果を保存

### MCP Tools
- `save_user_analysis` - Save Big Five personality analysis
  - All 5 traits (0.0-1.0) + summary
- `get_user_analysis` - Get latest personality profile

### Storage
- SQLite table: `user_analyses`
- Historical tracking: Compare analyses over time
- Helper methods: `dominant_trait()`, `is_high()`

---

## Layer 3.5: Integrated Profile

**Status**: ✅ **Complete**

### Purpose
Layer 1-3のデータを統合し、本質のみを抽出した統一プロファイル。「内部は複雑、表面はシンプル」の設計哲学を実現。

### Problem Solved
Layer 1-3は独立して動作するが、バラバラのデータをAIが毎回解釈する必要があった。Layer 3.5は統合された1つの答えを提供し、効率性とシンプルさを両立。

### Data Model
```rust
pub struct UserProfile {
    // 性格の本質（Big Five上位3特性）
    pub dominant_traits: Vec<TraitScore>,

    // 関心の核心（最頻出トピック5個）
    pub core_interests: Vec<String>,

    // 価値観の核心（高priority メモリから抽出、5個）
    pub core_values: Vec<String>,

    // 重要メモリID（証拠、上位10個）
    pub key_memory_ids: Vec<String>,

    // データ品質（0.0-1.0、メモリ数と分析有無で算出）
    pub data_quality: f32,

    pub last_updated: DateTime<Utc>,
}

pub struct TraitScore {
    pub name: String,    // "openness", "conscientiousness", etc.
    pub score: f32,      // 0.0-1.0
}
```

### Integration Logic

**1. Dominant Traits Extraction**
- Big Fiveから上位3特性を自動選択
- スコアでソート

**2. Core Interests Extraction**
- メモリコンテンツから頻度分析
- AI interpretationは2倍の重み
- 上位5個を抽出

**3. Core Values Extraction**
- priority_score >= 0.7 のメモリから抽出
- 価値関連キーワードをフィルタリング
- 上位5個を抽出

**4. Key Memories**
- priority_scoreでソート
- 上位10個のIDを保持（証拠として）

**5. Data Quality Score**
- メモリ数: 50個で1.0（それ以下は比例）
- 性格分析あり: +0.5
- 加重平均で算出

### Caching Strategy

**Storage**: SQLite `user_profiles` テーブル（1行のみ）

**Update Triggers**:
1. 10個以上の新しいメモリ追加
2. 新しい性格分析の保存
3. 7日以上経過

**Flow**:
```
get_profile()
  ↓
キャッシュ確認
  ↓
更新必要？ → No → キャッシュを返す
  ↓ Yes
Layer 1-3から再生成
  ↓
キャッシュ更新
  ↓
新しいプロファイルを返す
```

### MCP Tools
- `get_profile` - **Primary tool**: Get integrated profile

### Usage Pattern

**通常使用（効率的）**:
```
AI: get_profile()を呼ぶ
→ ユーザーの本質を理解
→ 適切な応答を生成
```

**詳細確認（必要時）**:
```
AI: get_profile()で概要を把握
→ 疑問がある
→ get_memory(id)で詳細確認
→ list_memories()で全体確認
```

### Design Philosophy

**"Internal complexity, external simplicity"**
- 内部: 複雑な分析、頻度計算、重み付け
- 表面: シンプルな1つのJSON
- AIは基本的にget_profile()のみ参照
- 柔軟性: 詳細データへのアクセスも可能

**Efficiency**:
- 頻繁な再計算を避ける（キャッシング）
- 必要時のみ更新（スマートトリガー）
- AI が迷わない（1つの明確な答え）

---

## Layer 4a: Game Systems

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

## Layer 4b: AI Companion

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

### Phase 2: Layer 2 ✅ (Complete)
- [x] Add AI interpretation fields to schema
- [x] Implement priority scoring logic
- [x] Create `create_ai_memory` tool
- [x] Update MCP server
- [x] Automatic schema migration
- [x] Backward compatibility

### Phase 3: Layer 3 ✅ (Complete)
- [x] Big Five personality model
- [x] UserAnalysis data structure
- [x] user_analyses table
- [x] `save_user_analysis` tool
- [x] `get_user_analysis` tool
- [x] Historical tracking support

### Phase 3.5: Layer 3.5 ✅ (Complete)
- [x] UserProfile data structure
- [x] Integration logic (traits, interests, values)
- [x] Frequency analysis for topic extraction
- [x] Value keyword extraction
- [x] Data quality scoring
- [x] Caching mechanism (user_profiles table)
- [x] Smart update triggers
- [x] `get_profile` MCP tool

### Phase 4: Layers 4-5 (Next)
- [ ] Game mechanics (Layer 4a)
- [ ] Companion system (Layer 4b)
- [ ] Sharing mechanisms (Layer 5)
- [ ] Public/private modes (Layer 5)

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
├── core/
│   ├── memory.rs      # Layer 1: Memory struct
│   ├── store.rs       # Layer 1-3.5: SQLite operations
│   ├── analysis.rs    # Layer 3: UserAnalysis (Big Five)
│   ├── profile.rs     # Layer 3.5: UserProfile (integrated)
│   ├── error.rs       # Error types
│   └── mod.rs         # Module exports
├── mcp/
│   ├── base.rs        # MCP server (all layers)
│   └── mod.rs         # Module exports
├── lib.rs             # Library root
└── main.rs            # CLI application
```

**Future layers**:
- Layer 4a: `src/game/` - Game systems
- Layer 4b: `src/companion/` - Companion features
- Layer 5: `src/distribution/` - Sharing mechanisms

---

**Version**: 0.2.0
**Last Updated**: 2025-11-06
**Current Status**: Layers 1-3.5 Complete, Layer 4 Planned
