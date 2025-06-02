# 設定ガイド

## 設定ファイルの場所

ai.gptの設定は `~/.config/syui/ai/gpt/config.json` に保存されます。

## 仮想環境の場所

ai.gptの仮想環境は `~/.config/syui/ai/gpt/venv/` に配置されます。これにより、設定とデータが一か所にまとまります。

```bash
# 仮想環境の有効化
source ~/.config/syui/ai/gpt/venv/bin/activate

# aigptコマンドが利用可能に
aigpt --help
```

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
aigpt config set providers.openai.api_key sk-xxxxx

# デフォルトモデルを変更
aigpt config set providers.openai.default_model gpt-4-turbo
```

### Ollama

```bash
# ホストを変更（リモートOllamaサーバーを使用する場合）
aigpt config set providers.ollama.host http://192.168.1.100:11434

# デフォルトモデルを変更
aigpt config set providers.ollama.default_model llama2
```

## atproto設定（将来の自動投稿用）

```bash
# Blueskyアカウント
aigpt config set atproto.handle yourhandle.bsky.social
aigpt config set atproto.password your-app-password

# セルフホストサーバーを使用
aigpt config set atproto.host https://your-pds.example.com
```

## デフォルトプロバイダー

```bash
# デフォルトをOpenAIに変更
aigpt config set default_provider openai
```

## セキュリティ

### APIキーの保護

設定ファイルは平文で保存されるため、適切なファイル権限を設定してください：

```bash
chmod 600 ~/.config/syui/ai/gpt/config.json
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
cp ~/.config/syui/ai/gpt/config.json ~/.config/syui/ai/gpt/config.json.backup

# リストア
cp ~/.config/syui/ai/gpt/config.json.backup ~/.config/syui/ai/gpt/config.json
```

## データディレクトリ

記憶データは `~/.config/syui/ai/gpt/data/` に保存されます：

```bash
ls ~/.config/syui/ai/gpt/data/
# conversations.json   memories.json   relationships.json   personas.json
```

これらのファイルも設定と同様にバックアップを推奨します。

## トラブルシューティング

### 設定が反映されない

```bash
# 現在の設定を確認
aigpt config list

# 特定のキーを確認
aigpt config get providers.openai.api_key
```

### 設定をリセット

```bash
# 設定ファイルを削除（次回実行時に再作成）
rm ~/.config/syui/ai/gpt/config.json
```