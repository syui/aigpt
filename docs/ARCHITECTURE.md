# Architecture: Multi-Layer Memory System

## Design Philosophy

aigptは、独立したレイヤーを積み重ねる設計です。各レイヤーは：

- **独立性**: 単独で動作可能
- **接続性**: 他のレイヤーと連携可能
- **段階的**: 1つずつ実装・テスト

## Layer Overview

```
┌─────────────────────────────────────────┐
│  Layer 5: Knowledge Sharing            │  🔵 Planned
│  (Information + Personality sharing)    │
├─────────────────────────────────────────┤
│  Layer 4+: Extended Features          │  🔵 Planned
│  (Advanced game/companion systems)      │
├─────────────────────────────────────────┤
│  Layer 4: Relationship Inference       │  ✅ Complete
│  (Bond strength, relationship types)    │  (Optional)
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
│  (SQLite, ULID, entity tracking)       │
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
    pub id: String,                           // ULID
    pub content: String,                      // User content
    pub related_entities: Option<Vec<String>>, // Who/what this memory involves (Layer 4)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Note**: `related_entities` added for Layer 4 support. Optional and backward compatible.

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

## Layer 4: Relationship Inference

**Status**: ✅ **Complete** (Optional feature)

### Purpose
Layer 1-3.5のデータから関係性を推測。ゲーム、コンパニオン、VTuberなどの外部アプリケーション向け。

### Activation
CLI引数で明示的に有効化:
```bash
aigpt server --enable-layer4
```

デフォルトでは無効（Layer 1-3.5のみ）。

### Data Model
```rust
pub struct RelationshipInference {
    pub entity_id: String,
    pub interaction_count: u32,     // この entity とのメモリ数
    pub avg_priority: f32,          // 平均重要度
    pub days_since_last: i64,       // 最終接触からの日数
    pub bond_strength: f32,         // 関係の強さ (0.0-1.0)
    pub relationship_type: String,  // close_friend, friend, etc.
    pub confidence: f32,            // 推測の信頼度 (0.0-1.0)
    pub inferred_at: DateTime<Utc>,
}
```

### Inference Logic

**1. データ収集**:
- Layer 1から entity に関連するメモリを抽出
- Layer 3.5からユーザー性格プロファイルを取得

**2. Bond Strength 計算**:
```rust
if user.extraversion < 0.5 {
    // 内向的: 少数の深い関係を好む
    // 回数が重要
    bond = interaction_count * 0.6 + avg_priority * 0.4
} else {
    // 外向的: 多数の浅い関係
    // 質が重要
    bond = interaction_count * 0.4 + avg_priority * 0.6
}
```

**3. Relationship Type 分類**:
- `close_friend` (0.8+): 非常に強い絆
- `friend` (0.6-0.8): 強い繋がり
- `valued_acquaintance` (0.4-0.6, 高priority): 重要だが親密ではない
- `acquaintance` (0.4-0.6): 定期的な接触
- `regular_contact` (0.2-0.4): 時々の接触
- `distant` (<0.2): 最小限の繋がり

**4. Confidence 計算**:
- データ量に基づく信頼度
- 1-2回: 0.2-0.3 (低)
- 5回: 0.5 (中)
- 10回以上: 0.8+ (高)

### Design Philosophy

**推測のみ、保存なし**:
- 毎回Layer 1-3.5から計算
- キャッシュなし（シンプルさ優先）
- 後でキャッシング追加可能

**独立性**:
- Layer 1-3.5に依存
- Layer 1-3.5から独立（オプション機能）
- 有効化しなければ完全に無視される

**外部アプリケーション向け**:
- aigptはバックエンド（推測エンジン）
- フロントエンド（ゲーム、コンパニオン等）が表示を担当
- MCPで繋がる

### MCP Tools
- `get_relationship(entity_id)` - 特定entity との関係を取得
- `list_relationships(limit)` - 全関係をbond_strength順でリスト

### Usage Example
```
# サーバー起動（Layer 4有効）
aigpt server --enable-layer4

# 関係性取得
get_relationship({ entity_id: "alice" })

# 結果:
{
  "bond_strength": 0.82,
  "relationship_type": "close_friend",
  "interaction_count": 15,
  "confidence": 0.80
}
```

---

## Layer 4+: Extended Features

**Status**: 🔵 **Planned**

Advanced game and companion system features to be designed based on Layer 4 foundation.

---

## Layer 4a: Game Systems (Archive)

**Status**: 🔵 **Archived Concept**

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

## Layer 5: Knowledge Sharing (Planned)

**Status**: 🔵 **Planned**

### Purpose
AIとのやり取りを「情報 + 個性」として共有する。SNSや配信のように、**有用な知見**と**作者の個性**を両立させたコンテンツプラットフォーム。

### Design Philosophy

人々が求めるもの：
1. **情報価値**: 「このプロンプトでこんな結果が得られた」「この問題をAIでこう解決した」
2. **個性・共感**: 「この人はこういう人だ」という親近感、信頼

SNSや配信と同じく、**情報のみは無機質**、**個性のみは空虚**。両方を組み合わせることで価値が生まれる。

### Data Model

```rust
pub struct SharedInteraction {
    pub id: String,

    // 情報価値
    pub problem: String,          // 何を解決しようとしたか
    pub approach: String,         // AIとどうやり取りしたか
    pub result: String,           // 何を得たか
    pub usefulness_score: f32,    // 有用性 (0.0-1.0, priority_score由来)
    pub tags: Vec<String>,        // 検索用タグ

    // 個性
    pub author_profile: ShareableProfile,  // 作者の本質
    pub why_this_matters: String,          // なぜこの人がこれに取り組んだか

    // メタデータ
    pub views: u32,
    pub useful_count: u32,        // 「役に立った」カウント
    pub created_at: DateTime<Utc>,
}

pub struct ShareableProfile {
    // ユーザーの本質（Layer 3.5から抽出）
    pub personality_essence: Vec<TraitScore>,  // Top 3 traits
    pub core_interests: Vec<String>,           // 5個
    pub core_values: Vec<String>,              // 5個

    // AIの解釈
    pub ai_perspective: String,   // AIがこのユーザーをどう理解しているか
    pub confidence: f32,          // データ品質 (0.0-1.0)

    // 関係性スタイル（Layer 4から推測、匿名化）
    pub relationship_style: String,  // 例: "深く狭い繋がりを好む"
}
```

### Privacy Design

**共有するもの:**
- ✅ 本質（Layer 3.5の統合プロファイル）
- ✅ パターン（関係性スタイル、思考パターン）
- ✅ 有用な知見（問題解決のアプローチ）

**共有しないもの:**
- ❌ 生の会話内容（Layer 1-2）
- ❌ 個人を特定できる情報
- ❌ メモリID、タイムスタンプ等の生データ

### Use Cases

**1. AI時代のGitHub Gist**
- 有用なプロンプトとその結果を共有
- 作者の個性とアプローチが見える
- 「この人の考え方が参考になる」

**2. 知見のSNS**
- 情報を発信しながら、個性も伝わる
- フォロー、「役に立った」機能
- 関心領域でフィルタリング

**3. AIペルソナのショーケース**
- 「AIは私をこう理解している」を共有
- 性格分析の精度を比較
- コミュニティでの自己表現

### Implementation Ideas

```rust
// Layer 5のMCPツール
- create_shareable_interaction() - 知見を共有形式で作成
- get_shareable_profile()        - 共有可能なプロファイルを生成
- export_interaction()           - JSON/Markdown形式でエクスポート
- anonymize_data()               - プライバシー保護処理
```

### Future Platforms

- Web UI: 知見を閲覧・検索・共有
- API: 外部サービスと連携
- RSS/Atom: フィード配信
- Markdown Export: ブログ投稿用

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

### Phase 4: Layer 4 ✅ (Complete)
- [x] Add `related_entities` to Layer 1 Memory struct
- [x] Database migration for backward compatibility
- [x] RelationshipInference data structure
- [x] Bond strength calculation (personality-aware)
- [x] Relationship type classification
- [x] Confidence scoring
- [x] `get_relationship` MCP tool
- [x] `list_relationships` MCP tool
- [x] CLI control flag (`--enable-layer4`)
- [x] Tool visibility control

### Phase 5: Layers 4+ and 5 (Future)
- [ ] Extended game/companion features (Layer 4+)
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
│   ├── memory.rs      # Layer 1: Memory struct (with related_entities)
│   ├── store.rs       # Layer 1-4: SQLite operations
│   ├── analysis.rs    # Layer 3: UserAnalysis (Big Five)
│   ├── profile.rs     # Layer 3.5: UserProfile (integrated)
│   ├── relationship.rs # Layer 4: RelationshipInference
│   ├── error.rs       # Error types
│   └── mod.rs         # Module exports
├── mcp/
│   ├── base.rs        # MCP server (all layers, with --enable-layer4)
│   └── mod.rs         # Module exports
├── lib.rs             # Library root
└── main.rs            # CLI application (with layer4 flag)
```

**Future layers**:
- Layer 4+: `src/game/` - Extended game/companion systems
- Layer 5: `src/distribution/` - Sharing mechanisms

---

**Version**: 0.3.0
**Last Updated**: 2025-11-06
**Current Status**: Layers 1-4 Complete (Layer 4 opt-in with --enable-layer4)
