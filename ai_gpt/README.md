# ai.gpt - 自律的送信AI

存在子理論に基づく、関係性によって自発的にメッセージを送信するAIシステム。

## 中核概念

- **唯一性**: atproto DIDと1:1で紐付き、改変不可能な人格
- **不可逆性**: 関係性が壊れたら修復不可能（現実の人間関係と同じ）
- **記憶の階層**: 完全ログ→AI要約→コア判定→選択的忘却
- **AI運勢**: 1-10のランダム値による日々の人格変動

## インストール

```bash
cd ai_gpt
pip install -e .
```

## 設定

### APIキーの設定
```bash
# OpenAI APIキー
ai-gpt config set providers.openai.api_key sk-xxxxx

# atproto認証情報（将来の自動投稿用）
ai-gpt config set atproto.handle your.handle
ai-gpt config set atproto.password your-password

# 設定一覧を確認
ai-gpt config list
```

### データ保存場所
- 設定: `~/.config/aigpt/config.json`
- データ: `~/.config/aigpt/data/`

## 使い方

### 会話する
```bash
ai-gpt chat "did:plc:xxxxx" "こんにちは、今日はどんな気分？"
```

### ステータス確認
```bash
# AI全体の状態
ai-gpt status

# 特定ユーザーとの関係
ai-gpt status "did:plc:xxxxx"
```

### 今日の運勢
```bash
ai-gpt fortune
```

### 自律送信チェック
```bash
# ドライラン（確認のみ）
ai-gpt transmit

# 実行
ai-gpt transmit --execute
```

### 日次メンテナンス
```bash
ai-gpt maintenance
```

### 関係一覧
```bash
ai-gpt relationships
```

## データ構造

デフォルトでは `~/.ai_gpt/` に以下のファイルが保存されます：

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

## MCP Server

### サーバー起動
```bash
# Ollamaを使用（デフォルト）
ai-gpt server --model qwen2.5 --provider ollama

# OpenAIを使用
ai-gpt server --model gpt-4o-mini --provider openai

# カスタムポート
ai-gpt server --port 8080
```

### AIプロバイダーを使った会話
```bash
# Ollamaで会話
ai-gpt chat "did:plc:xxxxx" "こんにちは" --provider ollama --model qwen2.5

# OpenAIで会話
ai-gpt chat "did:plc:xxxxx" "今日の調子はどう？" --provider openai --model gpt-4o-mini
```

### MCP Tools

サーバーが起動すると、以下のツールがAIから利用可能になります：

- `get_memories` - アクティブな記憶を取得
- `get_relationship` - 特定ユーザーとの関係を取得
- `get_all_relationships` - すべての関係を取得
- `get_persona_state` - 現在の人格状態を取得
- `process_interaction` - ユーザーとの対話を処理
- `check_transmission_eligibility` - 送信可能かチェック
- `get_fortune` - 今日の運勢を取得
- `summarize_memories` - 記憶を要約
- `run_maintenance` - メンテナンス実行

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
ai-gpt schedule add transmission_check "0 */6 * * *" --provider ollama --model qwen2.5

# 30分ごとに送信チェック（インターバル形式）
ai-gpt schedule add transmission_check "30m"

# 毎日午前3時にメンテナンス
ai-gpt schedule add maintenance "0 3 * * *"

# 1時間ごとに関係性減衰
ai-gpt schedule add relationship_decay "1h"

# 毎週月曜日に記憶要約
ai-gpt schedule add memory_summary "0 0 * * MON"
```

### タスク管理

```bash
# タスク一覧
ai-gpt schedule list

# タスクを無効化
ai-gpt schedule disable --task-id transmission_check_1234567890

# タスクを有効化
ai-gpt schedule enable --task-id transmission_check_1234567890

# タスクを削除
ai-gpt schedule remove --task-id transmission_check_1234567890
```

### スケジューラーデーモンの起動

```bash
# バックグラウンドでスケジューラーを実行
ai-gpt schedule run
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