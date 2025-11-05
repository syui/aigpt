# aigpt - AI Memory System with Psychological Priority

AI記憶装置（心理優先記憶システム）。ChatGPTのメモリ機能を参考にしながら、AIによる解釈と心理判定を加えた新しいメモリストレージシステムです。

## コンセプト

従来の「会話 → 保存 → 検索」ではなく、「会話 → AI解釈 → 保存 → 検索」を実現。
AIが記憶を解釈し、重要度を0.0-1.0のスコアで評価。優先度の高い記憶を保持し、低い記憶は自動的に削除されます。

## 機能

- **AI解釈付き記憶**: 元のコンテンツとAI解釈後のコンテンツを保存
- **心理判定スコア**: 0.0-1.0のfloat値で重要度を評価
- **優先順位管理**: スコアに基づく自動ソートとフィルタリング
- **容量制限**: 最大100件（設定可能）、低スコアから自動削除
- **メモリのCRUD操作**: メモリの作成、更新、削除、検索
- **ChatGPT JSONインポート**: ChatGPTの会話履歴からメモリを抽出
- **stdio MCP実装**: Claude Desktop/Codeとの簡潔な連携
- **JSONファイル保存**: シンプルなファイルベースのデータ保存

## インストール

1. Rustをインストール（まだの場合）:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. プロジェクトをビルド:
```bash
cargo build --release
```

3. バイナリをパスの通った場所にコピー（オプション）:
```bash
cp target/release/aigpt $HOME/.cargo/bin/
```

4. Claude Code/Desktopに追加

```sh
# Claude Codeの場合
claude mcp add aigpt $HOME/.cargo/bin/aigpt server

# または
claude mcp add aigpt $HOME/.cargo/bin/aigpt serve
```

## 使用方法

### ヘルプの表示
```bash
aigpt --help
```

### MCPサーバーとして起動
```bash
# MCPサーバー起動 (どちらでも可)
aigpt server
aigpt serve
```

### ChatGPT会話のインポート
```bash
# ChatGPT conversations.jsonをインポート
aigpt import path/to/conversations.json
```

## Claude Desktop/Codeへの設定

1. Claude Desktopの設定ファイルを開く:
   - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
   - Linux: `~/.config/Claude/claude_desktop_config.json`

2. 以下の設定を追加:
```json
{
  "mcpServers": {
    "aigpt": {
      "command": "/Users/syui/.cargo/bin/aigpt",
      "args": ["server"]
    }
  }
}
```

## 提供するMCPツール一覧

1. **create_memory** - 新しいメモリを作成
2. **update_memory** - 既存のメモリを更新
3. **delete_memory** - メモリを削除
4. **search_memories** - メモリを検索
5. **list_conversations** - インポートされた会話を一覧表示

## ツールの使用例

Claude Desktop/Codeで以下のように使用します：

### メモリの作成
```
MCPツールを使って「今日は良い天気です」というメモリーを作成してください
```

### メモリの検索
```
MCPツールを使って「天気」に関するメモリーを検索してください
```

### 会話一覧の表示
```
MCPツールを使ってインポートした会話の一覧を表示してください
```

## データ保存

- デフォルトパス: `~/.config/syui/ai/gpt/memory.json`
- JSONファイルでデータを保存
- 自動的にディレクトリとファイルを作成

### データ構造

```json
{
  "memories": {
    "uuid": {
      "id": "uuid",
      "content": "元のメモリー内容",
      "interpreted_content": "AI解釈後のメモリー内容",
      "priority_score": 0.75,
      "user_context": "ユーザー固有のコンテキスト（オプション）",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  },
  "conversations": {
    "conversation_id": {
      "id": "conversation_id",
      "title": "会話のタイトル",
      "created_at": "2024-01-01T00:00:00Z",
      "message_count": 10
    }
  }
}
```

### 心理判定スコアについて

0.0-1.0のfloat値で重要度を表現：
- **0.0-0.25**: 低優先度（忘れられやすい）
- **0.25-0.5**: 中優先度
- **0.5-0.75**: 高優先度
- **0.75-1.0**: 最高優先度（重要な記憶）

評価基準：
- 感情的インパクト (0.0-0.25)
- ユーザーとの関連性 (0.0-0.25)
- 新規性・独自性 (0.0-0.25)
- 実用性 (0.0-0.25)

## 開発

```bash
# 開発モードで実行
cargo run -- server

# ChatGPTインポートのテスト
cargo run -- import json/conversations.json

# テストの実行
cargo test

# フォーマット
cargo fmt

# Lintチェック
cargo clippy
```

## トラブルシューティング

### MCPサーバーが起動しない
```bash
# バイナリが存在するか確認
ls -la ~/.cargo/bin/aigpt

# 手動でテスト
echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' | aigpt server
```

### Claude Desktopでツールが見つからない
1. Claude Desktopを完全に再起動
2. 設定ファイルのパスが正しいか確認
3. ログファイルを確認: `~/Library/Logs/Claude/mcp-server-aigpt.log`

### インポートが失敗する
```bash
# JSONファイルの形式を確認
head -100 conversations.json | jq '.[0] | keys'
```

## ライセンス

MIT
