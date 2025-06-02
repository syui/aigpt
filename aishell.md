# ai.shell プロジェクト仕様書

## 概要
ai.shellは、AIを活用したインタラクティブなシェル環境です。Claude Codeのような体験を提供し、プロジェクトの目標と仕様をAIが理解して、開発を支援します。

## 主要機能

### 1. インタラクティブシェル
- AIとの対話型インターフェース
- シェルコマンドの実行（!command形式）
- 高度な補完機能
- コマンド履歴

### 2. AI支援機能
- **analyze <file>**: ファイルの分析
- **generate <description>**: コード生成
- **explain <topic>**: 概念の説明
- **load**: プロジェクト仕様（このファイル）の読み込み

### 3. ai.gpt統合
- 関係性ベースのAI人格
- 記憶システム
- 運勢システムによる応答の変化

## 使用方法

```bash
# ai.shellを起動
aigpt shell

# プロジェクト仕様を読み込み
ai.shell> load

# ファイルを分析
ai.shell> analyze src/main.py

# コードを生成
ai.shell> generate Python function to calculate fibonacci

# シェルコマンドを実行
ai.shell> !ls -la

# AIと対話
ai.shell> How can I improve this code?
```

## 技術スタック
- Python 3.10+
- prompt-toolkit（補完機能）
- fastapi-mcp（MCP統合）
- ai.gpt（人格・記憶システム）

## 開発目標
1. Claude Codeのような自然な開発体験
2. AIがプロジェクトコンテキストを理解
3. シェルコマンドとAIの seamless な統合
4. 開発者の生産性向上

## 今後の展開
- ai.cardとの統合（カードゲームMCPサーバー）
- より高度なプロジェクト理解機能
- 自動コード修正・リファクタリング
- テスト生成・実行