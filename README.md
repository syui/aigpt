# ai.gpt

## プロジェクト概要
- **名前**: ai.gpt
- **パッケージ**: aigpt  
- **言語**: Rust (完全移行済み)
- **タイプ**: 自律的送信AI + 統合MCP基盤
- **役割**: 記憶・関係性・開発支援の統合AIシステム

## 実装完了状況

### 🧠 記憶システム（MemoryManager）
- **階層的記憶**: 完全ログ→AI要約→コア記憶→選択的忘却
- **文脈検索**: キーワード・意味的検索
- **記憶要約**: AI駆動自動要約機能

### 🤝 関係性システム（RelationshipTracker）
- **不可逆性**: 現実の人間関係と同じ重み
- **時間減衰**: 自然な関係性変化
- **送信判定**: 関係性閾値による自発的コミュニケーション

### 🎭 人格システム（Persona）
- **AI運勢**: 1-10ランダム値による日々の人格変動
- **統合管理**: 記憶・関係性・運勢の統合判断
- **継続性**: 長期記憶による人格継承

### 💻 ai.shell統合（Claude Code機能）
- **インタラクティブ環境**: `aigpt shell`
- **開発支援**: ファイル分析・コード生成・プロジェクト管理
- **継続開発**: プロジェクト文脈保持

## MCP Server統合（17ツール）

### 🧠 Memory System（5ツール）
- get_memories, get_contextual_memories, search_memories
- create_summary, create_core_memory

### 🤝 Relationships（4ツール）  
- get_relationships, get_status
- chat_with_ai, check_transmissions

### 💻 Shell Integration（5ツール）
- execute_command, analyze_file, write_file
- list_files, run_scheduler

### ⚙️ System State（3ツール）
- get_scheduler_status, run_maintenance, get_transmission_history

### 🎴 ai.card連携（3ツール）
- get_user_cards, draw_card, get_draw_status
- **統合ServiceClient**: 統一されたHTTP通信基盤

### 📝 ai.log連携（新機能）
- **統合ServiceClient**: ai.logサービスとの統一インターフェース
- create_blog_post, build_blog, translate_document

## 開発環境・設定

### 環境構築
```bash
cd /Users/syui/ai/ai/gpt
cargo build --release
```

### 設定管理
- **メイン設定**: `/Users/syui/ai/ai/gpt/config.json.example`
- **データディレクトリ**: `~/.config/syui/ai/gpt/`

### 使用方法
```bash
# ai.shell起動
aigpt shell --model qwen2.5-coder:latest --provider ollama

# MCPサーバー起動
aigpt server --port 8001

# 記憶システム体験
aigpt chat syui "質問内容" --provider ollama --model qwen3:latest

# ドキュメント生成（ai.wiki統合）
aigpt docs --wiki

# トークン使用量・料金分析（Claude Code連携）
aigpt tokens report --days 7      # 美しい日別レポート（要DuckDB）
aigpt tokens cost --month today   # セッション別料金分析
aigpt tokens summary --period week # 基本的な使用量サマリー
```

## 技術アーキテクチャ

### Rust実装の統合構成
```
ai.gpt (Rust製MCPサーバー:8001)
├── 🧠 Memory & Persona System (Rust)
├── 🤝 Relationship Management (Rust) 
├── 📊 Scheduler & Transmission (Rust)
├── 💻 Shell Integration (Rust)
├── 🔗 ServiceClient (統一HTTP基盤)
│   ├── 🎴 ai.card (port 8000)
│   ├── 📝 ai.log (port 8002)
│   └── 🤖 ai.bot (port 8003)
└── 📚 ai.wiki Generator (Rust)
```

### 最新機能 (2024.06.09)
- **MCP API共通化**: ServiceClient統一基盤
- **ai.wiki統合**: 自動ドキュメント生成
- **サービス設定統一**: 動的サービス登録
- **完全Rust移行**: Python依存完全排除

### 今後の展開
- **自律送信**: atproto実装による真の自発的コミュニケーション
- **ai.ai連携**: 心理分析AIとの統合
- **分散SNS統合**: atproto完全対応

## 革新的な特徴

### AI駆動記憶システム
- ChatGPT 4,000件ログから学習した効果的記憶構築
- 人間的な忘却・重要度判定

### 不可逆関係性
- 現実の人間関係と同じ重みを持つAI関係性
- 修復不可能な関係性破綻システム

### 統合ServiceClient
- 複数AIサービスの統一インターフェース
- DRY原則に基づく共通化実現
- 設定ベースの柔軟なサービス管理

## アーカイブ情報
詳細な実装履歴・設計資料は `~/ai/ai/ai.wiki/gpt/` に移動済み
