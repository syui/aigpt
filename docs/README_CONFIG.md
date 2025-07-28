# Claude Memory MCP 設定ガイド

## モード選択

### 標準モード (Simple Mode)
- 基本的なメモリー機能のみ
- 軽量で高速
- 最小限の依存関係

### 拡張モード (Extended Mode)  
- AI分析機能
- セマンティック検索
- Web統合機能
- 高度なインサイト抽出

## ビルド・実行方法

### 標準モード
```bash
# MCPサーバー起動
cargo run --bin memory-mcp

# CLI実行
cargo run --bin aigpt -- create "メモリー内容"
```

### 拡張モード
```bash
# MCPサーバー起動
cargo run --bin memory-mcp-extended --features extended

# CLI実行 
cargo run --bin aigpt-extended --features extended -- create "メモリー内容" --analyze
```

## 設定ファイルの配置

### 標準モード

#### Claude Desktop
```bash
# macOS
cp claude_desktop_config.json ~/.config/claude-desktop/claude_desktop_config.json

# Windows
cp claude_desktop_config.json %APPDATA%\Claude\claude_desktop_config.json
```

#### Claude Code
```bash
# プロジェクトルートまたはグローバル設定
cp claude_code_config.json .claude/config.json
# または
cp claude_code_config.json ~/.claude/config.json
```

### 拡張モード

#### Claude Desktop
```bash
# macOS
cp claude_desktop_config_extended.json ~/.config/claude-desktop/claude_desktop_config.json

# Windows
cp claude_desktop_config_extended.json %APPDATA%\Claude\claude_desktop_config.json
```

#### Claude Code
```bash
# プロジェクトルートまたはグローバル設定
cp claude_code_config_extended.json .claude/config.json
# または
cp claude_code_config_extended.json ~/.claude/config.json
```

## 環境変数設定

```bash
export MEMORY_AUTO_EXECUTE=true
export MEMORY_AUTO_SAVE=true
export MEMORY_AUTO_SEARCH=true
export TRIGGER_SENSITIVITY=high
export MEMORY_DB_PATH=~/.claude/memory.db
```

## 設定オプション

### auto_execute
- `true`: 自動でMCPツールを実行
- `false`: 手動実行のみ

### trigger_sensitivity  
- `high`: 多くのキーワードで反応
- `medium`: 適度な反応
- `low`: 明確なキーワードのみ

### max_memories
メモリーの最大保存数

### search_limit
検索結果の最大表示数

## カスタマイズ

`trigger_words`セクションでトリガーワードをカスタマイズ可能:

```json
"trigger_words": {
  "custom_category": ["カスタム", "キーワード", "リスト"]
}
```

## トラブルシューティング

1. MCPサーバーが起動しない場合:
   - Rustがインストールされているか確認
   - `cargo build --release`でビルド確認

2. 自動実行されない場合:
   - 環境変数が正しく設定されているか確認
   - トリガーワードが含まれているか確認

3. メモリーが保存されない場合:
   - データベースファイルのパスが正しいか確認
   - 書き込み権限があるか確認