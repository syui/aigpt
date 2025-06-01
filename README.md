# ai.gpt - 自律的送信AI

存在子理論に基づく、関係性によって自発的にメッセージを送信するAIシステム。

## 中核概念

- **唯一性**: atproto DIDと1:1で紐付き、改変不可能な人格
- **不可逆性**: 関係性が壊れたら修復不可能（現実の人間関係と同じ）
- **記憶の階層**: 完全ログ→AI要約→コア判定→選択的忘却
- **AI運勢**: 1-10のランダム値による日々の人格変動

## インストール

```bash
# Python仮想環境を推奨
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
pip install -e .
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

### データ保存場所
- 設定: `~/.config/syui/ai/gpt/config.json`
- データ: `~/.config/syui/ai/gpt/data/`

## 使い方

### 会話する
```bash
aigpt chat "did:plc:xxxxx" "こんにちは、今日はどんな気分？"
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

## ai.shell統合

インタラクティブシェルモード（Claude Code風の体験）：

```bash
aigpt shell

# シェル内で使えるコマンド:
# help              - コマンド一覧
# !<command>        - シェルコマンド実行（例: !ls, !pwd）
# analyze <file>    - ファイルをAIで分析
# generate <desc>   - コード生成
# explain <topic>   - 概念の説明
# load              - aishell.mdプロジェクトファイルを読み込み
# status            - AI状態確認
# fortune           - AI運勢確認
# clear             - 画面クリア
# exit/quit         - 終了

# 通常のメッセージも送れます
ai.shell> こんにちは、今日は何をしましょうか？
```

## MCP Server

### サーバー起動
```bash
# Ollamaを使用（デフォルト）
aigpt server --model qwen2.5 --provider ollama

# OpenAIを使用
aigpt server --model gpt-4o-mini --provider openai

# カスタムポート
aigpt server --port 8080

# ai.card統合を有効化
aigpt server --enable-card
```

### AIプロバイダーを使った会話
```bash
# Ollamaで会話
aigpt chat "did:plc:xxxxx" "こんにちは" --provider ollama --model qwen2.5

# OpenAIで会話
aigpt chat "did:plc:xxxxx" "今日の調子はどう？" --provider openai --model gpt-4o-mini
```

### MCP Tools

サーバーが起動すると、以下のツールがAIから利用可能になります：

**ai.gpt ツール:**
- `get_memories` - アクティブな記憶を取得
- `get_relationship` - 特定ユーザーとの関係を取得
- `get_all_relationships` - すべての関係を取得
- `get_persona_state` - 現在の人格状態を取得
- `process_interaction` - ユーザーとの対話を処理
- `check_transmission_eligibility` - 送信可能かチェック
- `get_fortune` - 今日の運勢を取得
- `summarize_memories` - 記憶を要約
- `run_maintenance` - メンテナンス実行

**ai.shell ツール:**
- `execute_command` - シェルコマンド実行
- `analyze_file` - ファイルのAI分析
- `write_file` - ファイル書き込み
- `read_project_file` - プロジェクトファイル読み込み
- `list_files` - ファイル一覧

**ai.card ツール（--enable-card時）:**
- `get_user_cards` - ユーザーのカード取得
- `draw_card` - カードを引く（ガチャ）
- `get_card_details` - カード詳細情報
- `sync_cards_atproto` - atproto同期
- `analyze_card_collection` - コレクション分析

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

## 次のステップ

- atprotoへの実送信機能実装
- systemdサービス化
- Docker対応
- Webダッシュボード