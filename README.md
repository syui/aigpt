# ai.gpt - AI駆動記憶システム & 自律対話AI

🧠 **革新的記憶システム** × 🤖 **自律的人格AI** × 🔗 **atproto統合**

ChatGPTの4,000件会話ログから学んだ「効果的な記憶構築」を完全実装した、真の記憶を持つAIシステム。

## 🎯 核心機能

### 📚 AI駆動階層記憶システム
- **CORE記憶**: 人格形成要素の永続的記憶（AIが自動分析・抽出）
- **SUMMARY記憶**: テーマ別スマート要約（AI駆動パターン分析）
- **記憶検索**: コンテキスト認識による関連性スコアリング
- **選択的忘却**: 重要度に基づく自然な記憶の減衰

### 🤝 進化する関係性システム
- **唯一性**: atproto DIDと1:1で紐付き、改変不可能な人格
- **不可逆性**: 関係性が壊れたら修復不可能（現実の人間関係と同じ）
- **時間減衰**: 自然な関係性の変化と送信閾値システム
- **AI運勢**: 1-10のランダム値による日々の人格変動

### 🧬 統合アーキテクチャ
- **fastapi-mcp統一基盤**: Claude Desktop/Cursor完全対応
- **23種類のMCPツール**: 記憶・関係性・AI統合・シェル操作・リモート実行
- **ai.shell統合**: Claude Code風インタラクティブ開発環境
- **ai.bot連携**: systemd-nspawn隔離実行環境統合
- **マルチAI対応**: ollama(qwen3/gemma3) + OpenAI統合

## 🚀 クイックスタート

### 1分で体験する記憶システム

```bash
# 1. セットアップ（自動）
cd /Users/syui/ai/gpt
./setup_venv.sh

# 2. ollama + qwen3で記憶テスト
aigpt chat syui "記憶システムのテストです" --provider ollama --model qwen3:latest

# 3. 記憶の確認
aigpt status syui

# 4. インタラクティブシェル体験
aigpt shell
```

### 記憶システム体験デモ

```bash
# ChatGPTログインポート（既存データを使用）
aigpt import-chatgpt ./json/chatgpt.json --user-id syui

# AI記憶分析
aigpt maintenance  # スマート要約 + コア記憶生成

# 記憶に基づく対話
aigpt chat syui "前回の議論について覚えていますか？" --provider ollama --model qwen3:latest

# 記憶検索
# MCPサーバー経由でのコンテキスト記憶取得
aigpt server --port 8001 &
curl "http://localhost:8001/get_contextual_memories?query=ai&limit=5"
```

## インストール

```bash
# 仮想環境セットアップ（推奨）
cd /Users/syui/ai/gpt
source ~/.config/syui/ai/gpt/venv/bin/activate
pip install -e .

# または自動セットアップ
./setup_venv.sh
```

## 設定

### APIキーの設定
```bash
# OpenAI APIキー
aigpt config set providers.openai.api_key sk-xxxxx

# atproto認証情報（将来の自動投稿用）
aigpt config set atproto.handle your.handle
aigpt config set atproto.password your-password

# 設定一覧を確認
aigpt config list
```

### AIモデルの設定
```bash
# Ollamaのデフォルトモデルを変更
aigpt config set providers.ollama.default_model llama3

# OpenAIのデフォルトモデルを変更  
aigpt config set providers.openai.default_model gpt-4

# Ollamaホストの設定
aigpt config set providers.ollama.host http://localhost:11434

# 設定の確認
aigpt config get providers.ollama.default_model
```

### データ保存場所
- 設定: `~/.config/syui/ai/gpt/config.json`
- データ: `~/.config/syui/ai/gpt/data/`
- 仮想環境: `~/.config/syui/ai/gpt/venv/`

### 設定ファイル構造
```json
{
  "providers": {
    "ollama": {
      "host": "http://localhost:11434",
      "default_model": "qwen3"
    },
    "openai": {
      "api_key": null,
      "default_model": "gpt-4o-mini"
    }
  },
  "default_provider": "ollama"
}
```

## 使い方

### 会話する
```bash
# 通常の会話（詳細表示）
aigpt chat "did:plc:xxxxx" "こんにちは、今日はどんな気分？"

# 連続会話モード（シンプルな表示）
aigpt conversation syui --provider ollama --model qwen3:latest
aigpt conv syui --provider ollama --model qwen3:latest  # 短縮形
```

### ステータス確認
```bash
# AI全体の状態
aigpt status

# 特定ユーザーとの関係
aigpt status "did:plc:xxxxx"
```

### 今日の運勢
```bash
aigpt fortune
```

### 自律送信チェック
```bash
# ドライラン（確認のみ）
aigpt transmit

# 実行
aigpt transmit --execute
```

### 日次メンテナンス
```bash
aigpt maintenance
```

### 関係一覧
```bash
aigpt relationships
```

### 会話モード詳細

#### 通常の会話コマンド
```bash
# 詳細表示モード（関係性スコア・送信状態等も表示）
aigpt chat syui "メッセージ" --provider ollama --model qwen3:latest
```

出力例:
```
╭─────────────────────────── AI Response ───────────────────────────╮
│ AIの返答がここに表示されます                                        │
╰─────────────────────────────────────────────────────────────────╯

Relationship Status: stranger
Score: 28.00 / 100.0
Transmission: ✗ Disabled
```

#### 連続会話モード
```bash
# シンプルな会話画面（関係性情報なし）
aigpt conversation syui --provider ollama --model qwen3:latest
aigpt conv syui  # 短縮形、デフォルト設定使用
```

会話画面:
```
Using ollama with model qwen3:latest
Conversation with AI started. Type 'exit' or 'quit' to end.

syui> こんにちは
AI> こんにちは！今日はどんな日でしたか？

syui> 今日は良い天気でした
AI> 良い天気だと気分も晴れやかになりますね！

syui> exit
Conversation ended.
```

#### 会話モードの特徴
- **通常モード**: 詳細な関係性情報とパネル表示
- **連続モード**: シンプルな`ユーザー> ` → `AI> `形式
- **履歴保存**: 両モードとも会話履歴を自動保存
- **コマンド補完**: Tab補完とコマンド履歴機能

### ChatGPTデータインポート
```bash
# ChatGPTの会話履歴をインポート
aigpt import-chatgpt ./json/chatgpt.json --user-id "your_user_id"

# インポート後の確認
aigpt status
aigpt relationships
```

## データ構造

デフォルトでは `~/.config/syui/ai/gpt/` に以下のファイルが保存されます：

- `memories.json` - 会話記憶
- `conversations.json` - 会話ログ
- `relationships.json` - 関係性パラメータ
- `fortunes.json` - AI運勢履歴
- `transmissions.json` - 送信履歴
- `persona_state.json` - 人格状態

## 関係性の仕組み

- スコア0-200の範囲で変動
- 100を超えると送信機能が解禁
- 時間経過で自然減衰
- 大きなネガティブな相互作用で破壊される可能性

## 🖥️ ai.shell統合 - Claude Code風開発環境

### 🚀 **基本起動**
```bash
# デフォルト（qwen2.5使用）
aigpt shell

# qwen2.5-coder使用（コード生成に最適）
aigpt shell --model qwen2.5-coder:latest --provider ollama

# qwen3使用（高度な対話）
aigpt shell --model qwen3:latest --provider ollama

# OpenAI使用
aigpt shell --model gpt-4o-mini --provider openai
```

### 📋 **利用可能コマンド**
```bash
# === プロジェクト管理 ===
load                    # aishell.md読み込み（AIがプロジェクト理解）
status                  # AI状態・関係性確認
fortune                 # AI運勢確認（人格に影響）
relationships           # 全関係性一覧

# === AI開発支援 ===
analyze <file>          # ファイル分析・コードレビュー
generate <description>  # コード生成（qwen2.5-coder推奨）
explain <topic>         # 概念・技術説明

# === シェル操作 ===
!<command>             # シェルコマンド実行
!git status            # git操作
!ls -la               # ファイル確認
!mkdir project        # ディレクトリ作成
!pytest tests/        # テスト実行

# === リモート実行（ai.bot統合）===
remote <command>       # systemd-nspawn隔離コンテナでコマンド実行
isolated <code>        # Python隔離実行環境
aibot-status          # ai.botサーバー接続確認

# === インタラクティブ対話 ===
help                   # コマンド一覧
clear                  # 画面クリア
exit/quit             # 終了
<任意のメッセージ>      # 自由なAI対話
```

### 🎯 **コマンド使用例**
```bash
ai.shell> load
# → aishell.mdを読み込み、AIがプロジェクト目標を記憶

ai.shell> generate Python FastAPI CRUD for User model
# → 完全なCRUD API コードを生成

ai.shell> analyze src/main.py
# → コード品質・改善点を分析

ai.shell> !git log --oneline -5
# → 最近のコミット履歴を表示

ai.shell> remote ls -la /tmp
# → ai.bot隔離コンテナでディレクトリ確認

ai.shell> isolated print("Hello from isolated environment!")
# → Python隔離実行でHello World

ai.shell> aibot-status
# → ai.botサーバー接続状態とコンテナ情報確認

ai.shell> このAPIのセキュリティを改善してください
# → 記憶に基づく具体的なセキュリティ改善提案

ai.shell> explain async/await in Python
# → 非同期プログラミングの詳細説明
```

## MCP Server統合アーキテクチャ

### ai.gpt統合サーバー（簡素化設計）
```bash
# シンプルなサーバー起動（config.jsonから自動設定読み込み）
aigpt server

# カスタム設定での起動
aigpt server --host localhost --port 8001
```

**重要**: MCP function callingは**OpenAIプロバイダーでのみ対応**
- OpenAI GPT-4o-mini/GPT-4でfunction calling機能が利用可能
- Ollamaはシンプルなchat APIのみ（MCPツール非対応）

### MCP統合の動作条件
```bash
# MCP function calling対応（推奨）
aigpt conv test_user --provider openai --model gpt-4o-mini

# 通常の会話のみ（MCPツール非対応）
aigpt conv test_user --provider ollama --model qwen3
```

### ai.card独立サーバー
```bash
# ai.card独立サーバー起動（port 8000）
cd card/api
source ~/.config/syui/ai/card/venv/bin/activate
uvicorn app.main:app --port 8000
```

### 統合アーキテクチャ構成
```
OpenAI GPT-4o-mini (Function Calling対応)
    ↓
MCP Client (aigpt conv --provider openai)
    ↓ HTTP API
ai.gpt統合サーバー (port 8001) ← 27ツール
    ├── 🧠 Memory System: 5 tools
    ├── 🤝 Relationships: 4 tools  
    ├── ⚙️ System State: 3 tools
    ├── 💻 Shell Integration: 5 tools
    ├── 🔒 Remote Execution: 4 tools
    └── 📋 Project Management: 6 tools
                         
Ollama qwen3/gemma3 (Chat APIのみ)
    ↓
Direct Chat (aigpt conv --provider ollama)
    ↓ Direct Access
Memory/Relationship Systems
```

### プロバイダー別機能対応表
| 機能 | OpenAI | Ollama |
|------|--------|--------|
| 基本会話 | ✅ | ✅ |
| MCP Function Calling | ✅ | ❌ |
| 記憶システム連携 | ✅ (自動) | ✅ (直接) |
| `/memories`, `/search`コマンド | ✅ | ✅ |
| 自動記憶検索 | ✅ | ❌ |

### 使い分けガイド
```bash
# 高機能記憶連携（推奨）- OpenAI
aigpt conv syui --provider openai
# 「覚えていることある？」→ 自動的にget_memoriesツール実行

# シンプル会話 - Ollama  
aigpt conv syui --provider ollama
# 通常の会話、手動で /memories コマンド使用
```

### MCP Tools

サーバーが起動すると、以下のツールがAIから利用可能になります：

**ai.gpt ツール (9個):**
- `get_memories` - アクティブな記憶を取得
- `get_relationship` - 特定ユーザーとの関係を取得
- `get_all_relationships` - すべての関係を取得
- `get_persona_state` - 現在の人格状態を取得
- `process_interaction` - ユーザーとの対話を処理
- `check_transmission_eligibility` - 送信可能かチェック
- `get_fortune` - 今日の運勢を取得
- `summarize_memories` - 記憶を要約
- `run_maintenance` - メンテナンス実行

**ai.memory ツール (5個):**
- `get_contextual_memories` - 文脈的記憶検索
- `search_memories` - キーワード記憶検索
- `create_summary` - AI駆動記憶要約生成
- `create_core_memory` - コア記憶分析・抽出
- `get_context_prompt` - 記憶ベース文脈プロンプト

**ai.shell ツール (5個):**
- `execute_command` - シェルコマンド実行
- `analyze_file` - ファイルのAI分析
- `write_file` - ファイル書き込み
- `read_project_file` - プロジェクトファイル読み込み
- `list_files` - ファイル一覧

**ai.bot連携ツール (4個):**
- `remote_shell` - 隔離コンテナでコマンド実行
- `ai_bot_status` - ai.botサーバー状態確認
- `isolated_python` - Python隔離実行
- `isolated_analysis` - ファイル解析（隔離環境）

### ai.card独立サーバーとの連携

ai.cardは独立したMCPサーバーとして動作：
- **ポート**: 8000
- **9つのMCPツール**: カード管理・ガチャ・atproto同期等
- **データベース**: PostgreSQL/SQLite
- **起動**: `uvicorn app.main:app --port 8000`

ai.gptサーバーからHTTP経由で連携可能

## 環境変数

`.env`ファイルを作成して設定：

```bash
cp .env.example .env
# OpenAI APIキーを設定
```

## スケジューラー機能

### タスクの追加

```bash
# 6時間ごとに送信チェック
aigpt schedule add transmission_check "0 */6 * * *" --provider ollama --model qwen2.5

# 30分ごとに送信チェック（インターバル形式）
aigpt schedule add transmission_check "30m"

# 毎日午前3時にメンテナンス
aigpt schedule add maintenance "0 3 * * *"

# 1時間ごとに関係性減衰
aigpt schedule add relationship_decay "1h"

# 毎週月曜日に記憶要約
aigpt schedule add memory_summary "0 0 * * MON"
```

### タスク管理

```bash
# タスク一覧
aigpt schedule list

# タスクを無効化
aigpt schedule disable --task-id transmission_check_1234567890

# タスクを有効化
aigpt schedule enable --task-id transmission_check_1234567890

# タスクを削除
aigpt schedule remove --task-id transmission_check_1234567890
```

### スケジューラーデーモンの起動

```bash
# バックグラウンドでスケジューラーを実行
aigpt schedule run
```

### スケジュール形式

**Cron形式**:
- `"0 */6 * * *"` - 6時間ごと
- `"0 0 * * *"` - 毎日午前0時
- `"*/5 * * * *"` - 5分ごと

**インターバル形式**:
- `"30s"` - 30秒ごと
- `"5m"` - 5分ごと
- `"2h"` - 2時間ごと
- `"1d"` - 1日ごと

### タスクタイプ

- `transmission_check` - 送信可能なユーザーをチェックして自動送信
- `maintenance` - 日次メンテナンス（忘却、コア記憶判定など）
- `fortune_update` - AI運勢の更新
- `relationship_decay` - 関係性の時間減衰
- `memory_summary` - 記憶の要約作成

## 🚀 最新機能 (2025/06/02 大幅更新完了)

### ✅ **革新的記憶システム完成**
#### 🧠 AI駆動記憶機能
- **スマート要約生成**: AIによるテーマ別記憶要約（`create_smart_summary`）
- **コア記憶分析**: 人格形成要素の自動抽出（`create_core_memory`）
- **階層的記憶検索**: CORE→SUMMARY→RECENT優先度システム
- **コンテキスト認識**: クエリベース関連性スコアリング
- **文脈プロンプト**: 記憶に基づく一貫性のある対話生成

#### 🔗 完全統合アーキテクチャ
- **ChatGPTインポート**: 4,000件ログからの記憶構築実証
- **マルチAI対応**: ollama(qwen3:latest/gemma3:4b) + OpenAI完全統合
- **環境変数対応**: `OLLAMA_HOST`自動読み込み
- **MCP統合**: 23種類のツール（記憶5種+関係性4種+AI3種+シェル5種+ai.bot4種+項目管理2種）

#### 🧬 動作確認済み
- **記憶参照**: ChatGPTログからの文脈的記憶活用
- **人格統合**: ムード・運勢・記憶に基づく応答生成
- **関係性進化**: 記憶に基づく段階的信頼構築
- **AI協働**: qwen3との記憶システム完全連携

### 🎯 **新MCPツール**
```bash
# 新記憶システムツール
curl "http://localhost:8001/get_contextual_memories?query=programming&limit=5"
curl "http://localhost:8001/search_memories" -d '{"keywords":["memory","AI"]}'
curl "http://localhost:8001/create_summary" -d '{"user_id":"syui"}'
curl "http://localhost:8001/create_core_memory" -d '{}'
curl "http://localhost:8001/get_context_prompt" -d '{"user_id":"syui","message":"test"}'
```

### 🧪 **AIとの記憶テスト**
```bash
# qwen3での記憶システムテスト
aigpt chat syui "前回の会話を覚えていますか？" --provider ollama --model qwen3:latest

# 記憶に基づくスマート要約生成
aigpt maintenance  # AI要約を自動実行

# コンテキスト検索テスト
aigpt chat syui "記憶システムについて" --provider ollama --model qwen3:latest
```

## 🎉 **TODAY: MCP統合とサーバー表示改善完了** (2025/01/06)

### ✅ **本日の主要な改善**

#### 🚀 **サーバー起動表示の大幅改善**
従来のシンプルな表示から、プロフェッショナルな情報表示に刷新：

```bash
aigpt server
```
**改善前:**
```
Starting ai.gpt MCP Server
Host: localhost:8001
Endpoints: 27 MCP tools
```

**改善後:**
```
🚀 ai.gpt MCP Server

Server Configuration:
🌐 Address: http://localhost:8001
📋 API Docs: http://localhost:8001/docs
💾 Data Directory: /Users/syui/.config/syui/ai/gpt/data

AI Provider Configuration:
🤖 Provider: ollama ✅ http://192.168.11.95:11434
🧩 Model: qwen3

MCP Tools Available (27 total):
🧠 Memory System: 5 tools
🤝 Relationships: 4 tools  
⚙️  System State: 3 tools
💻 Shell Integration: 5 tools
🔒 Remote Execution: 4 tools

Integration Status:
✅ MCP Client Ready
🔗 Config: /Users/syui/.config/syui/ai/gpt/config.json
```

#### 🔧 **OpenAI Function Calling + MCP統合の実証**
OpenAI GPT-4o-miniでMCP function callingが完全動作：

```bash
aigpt conv test_user --provider openai --model gpt-4o-mini
```
**動作フロー:**
1. **自然言語入力**: 「覚えていることはある？」
2. **自動ツール選択**: OpenAIが`get_memories`を自動呼び出し
3. **MCP通信**: `http://localhost:8001/get_memories`にHTTPリクエスト
4. **記憶取得**: 実際の過去の会話データを取得
5. **文脈回答**: 記憶に基づく具体的な内容で回答

**技術的実証:**
```sh
🔧 [OpenAI] 1 tools called:
  - get_memories({"limit":5})
🌐 [MCP] Executing get_memories...
✅ [MCP] Result: [{'id': '5ce8f7d0-c078-43f1...
```

#### 📊 **統合アーキテクチャの完成**
```
OpenAI GPT-4o-mini
    ↓ (Function Calling)
MCP Client (aigpt conv)
    ↓ (HTTP API)
MCP Server (aigpt server:8001)
    ↓ (Direct Access)  
Memory/Relationship Systems
    ↓
JSON/SQLite Data
```

### 🎯 **技術的成果**
- ✅ **分散型AIシステム**: プロセス間MCP通信で複数AIアプリが記憶共有
- ✅ **OpenAI統合**: GPT-4o-miniのfunction callingが記憶システムと完全連携
- ✅ **プロフェッショナルUI**: enterprise-grade開発ツール風の情報表示
- ✅ **設定統合**: config.jsonからの自動設定読み込み
- ✅ **エラーハンドリング**: graceful shutdown、設定チェック、接続状態表示

### 📈 **ユーザー体験の向上**
- **開発者体験**: サーバー状況が一目で把握可能
- **デバッグ効率**: 詳細なログと状態表示
- **設定管理**: 設定ファイルパス、プロバイダー状態の明確化
- **AI連携**: OpenAI + MCP + 記憶システムのシームレス統合

**ai.gptの基盤アーキテクチャが完成し、実用的なAI記憶システムとして動作開始！** 🚀

## 🔥 **NEW: Claude Code的継続開発機能** (2025/06/03 完成)

### 🚀 **プロジェクト管理システム完全実装**
ai.shellに真のClaude Code風継続開発機能を実装しました：

#### 📊 **プロジェクト分析機能**
```bash
ai.shell> project-status
# ✓ プロジェクト構造自動分析
# Language: Python, Framework: FastAPI  
# 1268クラス, 5656関数, 22 API endpoints, 129 async functions
# 57個のファイル変更を検出

ai.shell> suggest-next
# ✓ AI駆動開発提案
# 1. 継続的な単体テストと統合テスト実装
# 2. API エンドポイントのセキュリティ強化
# 3. データベース最適化とキャッシュ戦略
```

#### 🧠 **コンテキスト認識開発**
```bash
ai.shell> continuous
# ✓ 継続開発モード開始
# プロジェクト文脈読込: 21,986文字
# claude.md + aishell.md + pyproject.toml + 依存関係を解析
# AIがプロジェクト全体を理解した状態で開発支援

ai.shell> analyze src/aigpt/project_manager.py
# ✓ プロジェクト文脈を考慮したファイル分析
# - コード品質評価
# - プロジェクトとの整合性チェック
# - 改善提案と潜在的問題の指摘

ai.shell> generate Create a test function for ContinuousDeveloper
# ✓ プロジェクト文脈を考慮したコード生成
# FastAPI, Python, 既存パターンに合わせた実装を自動生成
```

#### 🛠️ **実装詳細**
- **ProjectState**: ファイル変更検出・プロジェクト状態追跡
- **ContinuousDeveloper**: AI駆動プロジェクト分析・提案・コード生成
- **プロジェクト文脈**: claude.md/aishell.md/pyproject.toml等を自動読込
- **言語検出**: Python/JavaScript/Rust等の自動判定
- **フレームワーク分析**: FastAPI/Django/React等の依存関係検出
- **コードパターン**: 既存の設計パターン学習・適用

#### ✅ **動作確認済み機能**
- ✓ プロジェクト構造分析 (Language: Python, Framework: FastAPI)
- ✓ ファイル変更検出 (57個の変更検出)
- ✓ プロジェクト文脈読込 (21,986文字)
- ✓ AI駆動提案機能 (具体的な次ステップ提案)
- ✓ 文脈認識ファイル分析 (コード品質・整合性評価)
- ✓ プロジェクト文脈考慮コード生成 (FastAPI準拠コード生成)

### 🎯 **Claude Code風ワークフロー**
```bash
# 1. プロジェクト理解
aigpt shell --model qwen2.5-coder:latest --provider ollama
ai.shell> load               # プロジェクト仕様読み込み
ai.shell> project-status     # 現在の構造分析

# 2. AI駆動開発
ai.shell> suggest-next       # 次のタスク提案
ai.shell> continuous         # 継続開発モード開始

# 3. 文脈認識開発
ai.shell> analyze <file>     # プロジェクト文脈でファイル分析
ai.shell> generate <desc>    # 文脈考慮コード生成
ai.shell> 具体的な開発相談    # 記憶+文脈で最適な提案

# 4. 継続的改善
# AIがプロジェクト全体を理解して一貫した開発支援
# 前回の議論・決定事項を記憶して適切な提案継続
```

### 💡 **従来のai.shellとの違い**
| 機能 | 従来 | 新実装 |
|------|------|--------|
| プロジェクト理解 | 単発 | 構造分析+文脈保持 |
| コード生成 | 汎用 | プロジェクト文脈考慮 |
| 開発提案 | なし | AI駆動次ステップ提案 |
| ファイル分析 | 単体 | 整合性+改善提案 |
| 変更追跡 | なし | 自動検出+影響分析 |

**真のClaude Code化完成！** 記憶システム + プロジェクト文脈認識で、一貫した長期開発支援が可能になりました。

## 🛠️ ai.shell継続的開発 - 実践Example

### 🚀 **プロジェクト開発ワークフロー実例**

#### 📝 **Example 1: RESTful API開発**
```bash
# 1. ai.shellでプロジェクト開始（qwen2.5-coder使用）
aigpt shell --model qwen2.5-coder:latest --provider ollama

# 2. プロジェクト仕様を読み込んでAIに理解させる
ai.shell> load
# → aishell.mdを自動検索・読み込み、AIがプロジェクト目標を記憶

# 3. プロジェクト構造確認
ai.shell> !ls -la
ai.shell> !git status

# 4. ユーザー管理APIの設計を相談
ai.shell> RESTful APIでユーザー管理機能を作りたいです。設計について相談できますか？

# 5. AIの提案を基にコード生成
ai.shell> generate Python FastAPI user management with CRUD operations

# 6. 生成されたコードをファイルに保存
ai.shell> !mkdir -p src/api
ai.shell> !touch src/api/users.py

# 7. 実装されたコードを分析・改善
ai.shell> analyze src/api/users.py
ai.shell> セキュリティ面での改善点を教えてください

# 8. テストコード生成
ai.shell> generate pytest test cases for the user management API

# 9. 隔離環境でテスト実行
ai.shell> remote python -m pytest tests/ -v
ai.shell> isolated import requests; print(requests.get("http://localhost:8000/health").status_code)

# 10. 段階的コミット
ai.shell> !git add .
ai.shell> !git commit -m "Add user management API with security improvements"

# 11. 継続的な改善相談
ai.shell> 次はデータベース設計について相談したいです
```

#### 🔄 **Example 2: 機能拡張と リファクタリング**
```bash
# ai.shell継続セッション（記憶システムが前回の議論を覚えている）
aigpt shell --model qwen2.5-coder:latest --provider ollama

# AIが前回のAPI開発を記憶して続きから開始
ai.shell> status
# Relationship Status: acquaintance (関係性が進展)
# Score: 25.00 / 100.0

# 前回の続きから自然に議論
ai.shell> 前回作ったユーザー管理APIに認証機能を追加したいです

# AIが前回のコードを考慮した提案
ai.shell> generate JWT authentication middleware for our FastAPI

# 既存コードとの整合性チェック
ai.shell> analyze src/api/users.py
ai.shell> この認証システムと既存のAPIの統合方法は？

# 段階的実装
ai.shell> explain JWT token flow in our architecture
ai.shell> generate authentication decorator for protected endpoints

# リファクタリング提案
ai.shell> 現在のコード構造で改善できる点はありますか？
ai.shell> generate improved project structure for scalability

# データベース設計相談
ai.shell> explain SQLAlchemy models for user authentication
ai.shell> generate database migration scripts

# 隔離環境での安全なテスト
ai.shell> remote alembic upgrade head
ai.shell> isolated import sqlalchemy; print("DB connection test")
```

#### 🎯 **Example 3: バグ修正と最適化**
```bash
# 開発継続（AIが開発履歴を完全記憶）
aigpt shell --model qwen2.5-coder:latest --provider ollama

# 関係性が更に進展（close_friend level）
ai.shell> status
# Relationship Status: close_friend
# Score: 45.00 / 100.0

# バグレポートと分析
ai.shell> API のレスポンス時間が遅いです。パフォーマンス分析をお願いします
ai.shell> analyze src/api/users.py

# AIによる最適化提案
ai.shell> generate database query optimization for user lookup
ai.shell> explain async/await patterns for better performance

# テスト駆動改善
ai.shell> generate performance test cases
ai.shell> !pytest tests/ -v --benchmark

# キャッシュ戦略相談
ai.shell> Redis caching strategy for our user API?
ai.shell> generate caching layer implementation

# 本番デプロイ準備
ai.shell> explain Docker containerization for our API
ai.shell> generate Dockerfile and docker-compose.yml
ai.shell> generate production environment configurations

# 隔離環境でのデプロイテスト
ai.shell> remote docker build -t myapi .
ai.shell> isolated os.system("docker run --rm myapi python -c 'print(\"Container works!\")'")
ai.shell> aibot-status  # デプロイ環境確認
```

### 🧠 **記憶システム活用のメリット**

#### 💡 **継続性のある開発体験**
- **文脈保持**: 前回の議論やコードを記憶して一貫した提案
- **関係性進化**: 協働を通じて信頼関係が構築され、より深い提案
- **段階的成長**: プロジェクトの発展を理解した適切なレベルの支援

#### 🔧 **実践的な使い方**
```bash
# 日々の開発ルーチン
aigpt shell --model qwen2.5-coder:latest --provider ollama
ai.shell> load                    # プロジェクト状況をAIに再確認
ai.shell> !git log --oneline -5   # 最近の変更を確認
ai.shell> 今日は何から始めましょうか？ # AIが文脈を考慮した提案

# 長期プロジェクトでの活用
ai.shell> 先週議論したアーキテクチャの件、覚えていますか？
ai.shell> あのときの懸念点は解決されましたか？
ai.shell> 次のマイルストーンに向けて何が必要でしょうか？

# チーム開発での知識共有
ai.shell> 新しいメンバーに説明するための設計書を生成してください
ai.shell> このプロジェクトの技術的負債について分析してください
```

### 🚧 次のステップ
- **自律送信**: atproto実装（記憶ベース判定）
- **記憶可視化**: Webダッシュボード（関係性グラフ）
- **分散記憶**: atproto上でのユーザーデータ主権
- **AI協働**: 複数AIでの記憶共有プロトコル

## トラブルシューティング

### 環境セットアップ
```bash
# 仮想環境の確認
source ~/.config/syui/ai/gpt/venv/bin/activate
aigpt --help

# 設定の確認
aigpt config list

# データの確認
ls ~/.config/syui/ai/gpt/data/
```

### MCPサーバー動作確認
```bash
# ai.gpt統合サーバー (14ツール)
aigpt server --port 8001
curl http://localhost:8001/docs

# ai.card独立サーバー (9ツール)
cd card/api && uvicorn app.main:app --port 8000
curl http://localhost:8000/health
```