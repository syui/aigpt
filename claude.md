# ai.gpt プロジェクト固有情報

## プロジェクト概要
- **名前**: ai.gpt
- **パッケージ**: aigpt
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

## MCP Server統合（23ツール）

### 🧠 Memory System（5ツール）
- get_memories, get_contextual_memories, search_memories
- create_summary, create_core_memory

### 🤝 Relationships（4ツール）  
- get_relationship, get_all_relationships
- process_interaction, check_transmission_eligibility

### 💻 Shell Integration（5ツール）
- execute_command, analyze_file, write_file
- read_project_file, list_files

### 🔒 Remote Execution（4ツール）
- remote_shell, ai_bot_status
- isolated_python, isolated_analysis

### ⚙️ System State（3ツール）
- get_persona_state, get_fortune, run_maintenance

### 🎴 ai.card連携（6ツール + 独立MCPサーバー）
- card_draw_card, card_get_user_cards, card_analyze_collection
- **独立サーバー**: FastAPI + MCP (port 8000)

### 📝 ai.log連携（8ツール + Rustサーバー）
- log_create_post, log_ai_content, log_translate_document
- **独立サーバー**: Rust製 (port 8002)

## 開発環境・設定

### 環境構築
```bash
cd /Users/syui/ai/gpt
./setup_venv.sh
source ~/.config/syui/ai/gpt/venv/bin/activate
```

### 設定管理
- **メイン設定**: `/Users/syui/ai/gpt/config.json`
- **データディレクトリ**: `~/.config/syui/ai/gpt/`
- **仮想環境**: `~/.config/syui/ai/gpt/venv/`

### 使用方法
```bash
# ai.shell起動
aigpt shell --model qwen2.5-coder:latest --provider ollama

# MCPサーバー起動
aigpt server --port 8001

# 記憶システム体験
aigpt chat syui "質問内容" --provider ollama --model qwen3:latest
```

## 技術アーキテクチャ

### 統合構成
```
ai.gpt (統合MCPサーバー:8001)
├── 🧠 ai.gpt core (記憶・関係性・人格)
├── 💻 ai.shell (Claude Code風開発環境)
├── 🎴 ai.card (独立MCPサーバー:8000)
└── 📝 ai.log (Rust製ブログシステム:8002)
```

### 今後の展開
- **自律送信**: atproto実装による真の自発的コミュニケーション
- **ai.ai連携**: 心理分析AIとの統合
- **ai.verse統合**: UEメタバースとの連携
- **分散SNS統合**: atproto完全対応

## 革新的な特徴

### AI駆動記憶システム
- ChatGPT 4,000件ログから学習した効果的記憶構築
- 人間的な忘却・重要度判定

### 不可逆関係性
- 現実の人間関係と同じ重みを持つAI関係性
- 修復不可能な関係性破綻システム

### 統合アーキテクチャ
- fastapi_mcp基盤での複数AIシステム統合
- OpenAI Function Calling + MCP完全連携実証済み