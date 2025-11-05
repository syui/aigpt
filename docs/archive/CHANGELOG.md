# Changelog

## [Unreleased] - 2025-11-05

### 🎉 Major Changes: Complete Local Operation

#### Changed
- **Removed external AI API dependency**: No longer calls Claude/OpenAI APIs
- **Claude Code does the interpretation**: AIが解釈するのではなく、Claude Code 自身が解釈
- **Zero cost**: API料金が一切かからない
- **Complete privacy**: データが外部に送信されない

#### Technical Details
- Removed `openai` crate dependency
- Removed `ai-analysis` feature (no longer needed)
- Simplified `ai_interpreter.rs` to be a lightweight wrapper
- Updated `create_memory_with_ai` MCP tool to accept `interpreted_content` and `priority_score` from Claude Code
- Added `create_memory_with_interpretation()` method to MemoryManager
- Updated tool descriptions to guide Claude Code on how to interpret and score

#### Benefits
- ✅ **完全ローカル**: 外部 API 不要
- ✅ **ゼロコスト**: API 料金なし
- ✅ **プライバシー**: データ漏洩の心配なし
- ✅ **シンプル**: 依存関係が少ない
- ✅ **高速**: ネットワーク遅延なし

#### How It Works Now

1. User: 「今日、新しいアイデアを思いついた」とメモリを作成
2. Claude Code: 内容を解釈し、スコア (0.0-1.0) を計算
3. Claude Code: `create_memory_with_ai` ツールを呼び出し、解釈とスコアを渡す
4. aigpt: メモリを保存し、ゲーム風の結果を返す
5. Claude Code: ユーザーに結果を表示

#### Migration Notes

For users who were expecting external AI API usage:
- No API keys needed anymore (ANTHROPIC_API_KEY, OPENAI_API_KEY)
- Claude Code (local) now does all the interpretation
- This is actually better: faster, cheaper, more private!

---

## [0.1.0] - Initial Release

### Added
- Basic memory CRUD operations
- ChatGPT conversation import
- stdio MCP server implementation
- Psychological priority scoring (0.0-1.0)
- Gamification features (rarity, diagnosis types, XP)
- Romance companion system
- 11 MCP tools for Claude Code integration

### Features
- Memory capacity management (max 100 by default)
- Automatic pruning of low-priority memories
- Game-style result displays
- Companion affection and level system
- Daily challenges
- Ranking displays

### Documentation
- README.md with full examples
- DESIGN.md with system architecture
- TECHNICAL_REVIEW.md with evaluation
- ROADMAP.md with 7-phase plan
- QUICKSTART.md for immediate usage
- USAGE.md for detailed instructions
