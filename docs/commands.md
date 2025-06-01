# コマンドリファレンス

## chat - AIと会話

ユーザーとAIの対話を処理し、関係性を更新します。

```bash
ai-gpt chat USER_ID MESSAGE [OPTIONS]
```

### 引数
- `USER_ID`: ユーザーID（atproto DID形式）
- `MESSAGE`: 送信するメッセージ

### オプション
- `--provider`: AIプロバイダー（ollama/openai）
- `--model`, `-m`: 使用するモデル
- `--data-dir`, `-d`: データディレクトリ

### 例
```bash
# 基本的な会話
ai-gpt chat "did:plc:user123" "こんにちは"

# OpenAIを使用
ai-gpt chat "did:plc:user123" "調子はどう？" --provider openai --model gpt-4o-mini

# Ollamaでカスタムモデル
ai-gpt chat "did:plc:user123" "今日の天気は？" --provider ollama --model llama2
```

## status - 状態確認

AIの状態や特定ユーザーとの関係を表示します。

```bash
ai-gpt status [USER_ID] [OPTIONS]
```

### 引数
- `USER_ID`: （オプション）特定ユーザーとの関係を確認

### 例
```bash
# AI全体の状態
ai-gpt status

# 特定ユーザーとの関係
ai-gpt status "did:plc:user123"
```

## fortune - 今日の運勢

AIの今日の運勢を確認します。

```bash
ai-gpt fortune [OPTIONS]
```

### 表示内容
- 運勢値（1-10）
- 連続した幸運/不運の日数
- ブレークスルー状態

## relationships - 関係一覧

すべてのユーザーとの関係を一覧表示します。

```bash
ai-gpt relationships [OPTIONS]
```

### 表示内容
- ユーザーID
- 関係性ステータス
- スコア
- 送信可否
- 最終対話日

## transmit - 送信実行

送信可能なユーザーへのメッセージを確認・実行します。

```bash
ai-gpt transmit [OPTIONS]
```

### オプション
- `--dry-run/--execute`: ドライラン（デフォルト）または実行
- `--data-dir`, `-d`: データディレクトリ

### 例
```bash
# 送信内容を確認（ドライラン）
ai-gpt transmit

# 実際に送信を実行
ai-gpt transmit --execute
```

## maintenance - メンテナンス

日次メンテナンスタスクを実行します。

```bash
ai-gpt maintenance [OPTIONS]
```

### 実行内容
- 関係性の時間減衰
- 記憶の忘却処理
- コア記憶の判定
- 記憶の要約作成

## config - 設定管理

設定の確認・変更を行います。

```bash
ai-gpt config ACTION [KEY] [VALUE]
```

### アクション
- `get`: 設定値を取得
- `set`: 設定値を変更
- `delete`: 設定を削除
- `list`: 設定一覧を表示

### 例
```bash
# APIキーを設定
ai-gpt config set providers.openai.api_key sk-xxxxx

# 設定を確認
ai-gpt config get providers.openai.api_key

# 設定一覧
ai-gpt config list

# プロバイダー設定のみ表示
ai-gpt config list providers
```

## schedule - スケジュール管理

定期実行タスクを管理します。

```bash
ai-gpt schedule ACTION [TASK_TYPE] [SCHEDULE] [OPTIONS]
```

### アクション
- `add`: タスクを追加
- `list`: タスク一覧
- `enable`: タスクを有効化
- `disable`: タスクを無効化
- `remove`: タスクを削除
- `run`: スケジューラーを起動

### タスクタイプ
- `transmission_check`: 送信チェック
- `maintenance`: 日次メンテナンス
- `fortune_update`: 運勢更新
- `relationship_decay`: 関係性減衰
- `memory_summary`: 記憶要約

### スケジュール形式
- **Cron形式**: `"0 */6 * * *"` (6時間ごと)
- **インターバル**: `"30m"`, `"2h"`, `"1d"`

### 例
```bash
# 30分ごとに送信チェック
ai-gpt schedule add transmission_check "30m"

# 毎日午前3時にメンテナンス
ai-gpt schedule add maintenance "0 3 * * *"

# タスク一覧
ai-gpt schedule list

# スケジューラーを起動
ai-gpt schedule run
```

## server - MCP Server

AIの記憶と機能をMCPツールとして公開します。

```bash
ai-gpt server [OPTIONS]
```

### オプション
- `--host`, `-h`: サーバーホスト（デフォルト: localhost）
- `--port`, `-p`: サーバーポート（デフォルト: 8000）
- `--model`, `-m`: AIモデル
- `--provider`: AIプロバイダー

### 例
```bash
# 基本的な起動
ai-gpt server

# カスタム設定
ai-gpt server --port 8080 --model gpt-4o-mini --provider openai
```