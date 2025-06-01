# 設定ガイド

## 設定ファイルの場所

ai.gptの設定は `~/.config/aigpt/config.json` に保存されます。

## 設定構造

```json
{
  "providers": {
    "openai": {
      "api_key": "sk-xxxxx",
      "default_model": "gpt-4o-mini"
    },
    "ollama": {
      "host": "http://localhost:11434",
      "default_model": "qwen2.5"
    }
  },
  "atproto": {
    "handle": "your.handle",
    "password": "your-password",
    "host": "https://bsky.social"
  },
  "default_provider": "ollama"
}
```

## プロバイダー設定

### OpenAI

```bash
# APIキーを設定
ai-gpt config set providers.openai.api_key sk-xxxxx

# デフォルトモデルを変更
ai-gpt config set providers.openai.default_model gpt-4-turbo
```

### Ollama

```bash
# ホストを変更（リモートOllamaサーバーを使用する場合）
ai-gpt config set providers.ollama.host http://192.168.1.100:11434

# デフォルトモデルを変更
ai-gpt config set providers.ollama.default_model llama2
```

## atproto設定（将来の自動投稿用）

```bash
# Blueskyアカウント
ai-gpt config set atproto.handle yourhandle.bsky.social
ai-gpt config set atproto.password your-app-password

# セルフホストサーバーを使用
ai-gpt config set atproto.host https://your-pds.example.com
```

## デフォルトプロバイダー

```bash
# デフォルトをOpenAIに変更
ai-gpt config set default_provider openai
```

## セキュリティ

### APIキーの保護

設定ファイルは平文で保存されるため、適切なファイル権限を設定してください：

```bash
chmod 600 ~/.config/aigpt/config.json
```

### 環境変数との優先順位

1. コマンドラインオプション（最優先）
2. 設定ファイル
3. 環境変数（最低優先）

例：OpenAI APIキーの場合
- `--api-key` オプション
- `config.json` の `providers.openai.api_key`
- 環境変数 `OPENAI_API_KEY`

## 設定のバックアップ

```bash
# バックアップ
cp ~/.config/aigpt/config.json ~/.config/aigpt/config.json.backup

# リストア
cp ~/.config/aigpt/config.json.backup ~/.config/aigpt/config.json
```

## トラブルシューティング

### 設定が反映されない

```bash
# 現在の設定を確認
ai-gpt config list

# 特定のキーを確認
ai-gpt config get providers.openai.api_key
```

### 設定をリセット

```bash
# 設定ファイルを削除（次回実行時に再作成）
rm ~/.config/aigpt/config.json
```