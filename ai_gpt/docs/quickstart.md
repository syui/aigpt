# クイックスタート

## インストール

```bash
# リポジトリをクローン
git clone https://github.com/yourusername/ai_gpt.git
cd ai_gpt

# インストール
pip install -e .
```

## 初期設定

### 1. OpenAIを使う場合

```bash
# APIキーを設定
ai-gpt config set providers.openai.api_key sk-xxxxx
```

### 2. Ollamaを使う場合（ローカルLLM）

```bash
# Ollamaをインストール（まだの場合）
# https://ollama.ai からダウンロード

# モデルをダウンロード
ollama pull qwen2.5
```

## 基本的な使い方

### 1. AIと会話する

```bash
# シンプルな会話（Ollamaを使用）
ai-gpt chat "did:plc:user123" "こんにちは！"

# OpenAIを使用
ai-gpt chat "did:plc:user123" "今日はどんな気分？" --provider openai --model gpt-4o-mini
```

### 2. 関係性を確認

```bash
# 特定ユーザーとの関係を確認
ai-gpt status "did:plc:user123"

# AIの全体的な状態を確認
ai-gpt status
```

### 3. 自動送信を設定

```bash
# 30分ごとに送信チェック
ai-gpt schedule add transmission_check "30m"

# スケジューラーを起動
ai-gpt schedule run
```

## 次のステップ

- [基本概念](concepts.md) - システムの仕組みを理解
- [コマンドリファレンス](commands.md) - 全コマンドの詳細
- [設定ガイド](configuration.md) - 詳細な設定方法