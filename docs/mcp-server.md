# MCP Server

## 概要

MCP (Model Context Protocol) Serverは、ai.gptの記憶と機能をAIツールとして公開します。これにより、Claude DesktopなどのMCP対応AIアシスタントがai.gptの機能にアクセスできます。

## 起動方法

```bash
# 基本的な起動
ai-gpt server

# カスタム設定
ai-gpt server --host 0.0.0.0 --port 8080 --model gpt-4o-mini --provider openai
```

## 利用可能なツール

### get_memories
アクティブな記憶を取得します。

**パラメータ**:
- `user_id` (optional): 特定ユーザーに関する記憶
- `limit`: 取得する記憶の最大数（デフォルト: 10）

**返り値**: 記憶のリスト（ID、内容、レベル、重要度、コア判定、タイムスタンプ）

### get_relationship
特定ユーザーとの関係性を取得します。

**パラメータ**:
- `user_id`: ユーザーID（必須）

**返り値**: 関係性情報（ステータス、スコア、送信可否、総対話数など）

### get_all_relationships
すべての関係性を取得します。

**返り値**: すべてのユーザーとの関係性リスト

### get_persona_state
現在のAI人格状態を取得します。

**返り値**: 
- 現在の気分
- 今日の運勢
- 人格特性値
- アクティブな記憶数

### process_interaction
ユーザーとの対話を処理します。

**パラメータ**:
- `user_id`: ユーザーID
- `message`: メッセージ内容

**返り値**: 
- AIの応答
- 関係性の変化量
- 新しい関係性スコア
- 送信機能の状態

### check_transmission_eligibility
特定ユーザーへの送信可否をチェックします。

**パラメータ**:
- `user_id`: ユーザーID

**返り値**: 送信可否と関係性情報

### get_fortune
今日のAI運勢を取得します。

**返り値**: 運勢値、連続日数、ブレークスルー状態、人格への影響

### summarize_memories
記憶の要約を作成します。

**パラメータ**:
- `user_id`: ユーザーID

**返り値**: 作成された要約（ある場合）

### run_maintenance
日次メンテナンスを実行します。

**返り値**: 実行ステータス

## Claude Desktopでの設定

`~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ai-gpt": {
      "command": "ai-gpt",
      "args": ["server", "--port", "8001"],
      "env": {}
    }
  }
}
```

## 使用例

### AIアシスタントからの利用

```
User: ai.gptで私との関係性を確認して