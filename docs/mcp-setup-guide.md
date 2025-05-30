# MCP Server セットアップガイド
Claude Code + ローカルLLM統合環境

## 🚀 セットアップ手順

### 1. 依存関係のインストール

```bash
# 仮想環境作成
python -m venv mcp-env
mcp-env\Scripts\activate  # Windows
# source mcp-env/bin/activate  # Linux/Mac

# 必要なパッケージをインストール
pip install mcp requests pathlib asyncio
```

### 2. Ollamaのセットアップ

```bash
# Ollamaのインストール（https://ollama.com）
# Windows: インストーラーをダウンロード
# Linux: curl -fsSL https://ollama.com/install.sh | sh

# Qwen2.5-Coderモデルをダウンロード
ollama pull qwen2.5-coder:14b-instruct-q4_K_M

# Ollamaサーバー起動確認
ollama serve
```

### 3. Claude Desktop設定

#### claude_desktop_config.json の作成
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Linux**: `~/.config/claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "local-llm": {
      "command": "python",
      "args": ["/path/to/your/local_llm_mcp_server.py"],
      "env": {
        "OLLAMA_URL": "http://localhost:11434",
        "DEFAULT_MODEL": "qwen2.5-coder:14b-instruct-q4_K_M"
      }
    }
  }
}
```

### 4. Claude Code設定

```bash
# Claude Codeをインストール（既にインストール済みの場合はスキップ）
# 公式サイトからダウンロード

# MCP サーバーを追加
claude mcp add local-llm

# または手動で設定ファイルを編集
# ~/.config/claude-code/config.json
```

## 🎯 使用方法

### Claude Codeから使用

```bash
# Claude Codeを起動
claude code

# プロンプト例:
# "Use local LLM to implement a Python quicksort function"
# "Analyze main.py with local model for potential bugs"
# "Generate a REST API using the local coding model"
```

### 利用可能なツール

1. **code_with_local_llm**
   - タスク: `"Implement a binary search tree in Python"`
   - プロジェクトコンテキスト含む: `true`

2. **read_file_with_analysis**
   - ファイルパス: `"src/main.py"`
   - 分析タイプ: `"bugs"` | `"optimization"` | `"documentation"`

3. **write_code_to_file**
   - ファイルパス: `"utils/helpers.py"`
   - タスク説明: `"Create utility functions for data processing"`

4. **debug_with_llm**
   - エラーメッセージ: `"IndexError: list index out of range"`
   - コードコンテキスト: 該当するコード部分

5. **explain_code**
   - コード: 解説したいコード
   - 詳細レベル: `"basic"` | `"medium"` | `"detailed"`

6. **switch_model**
   - モデル名: `"qwen2.5-coder:7b-instruct"`

## 🔧 カスタマイズ

### モデル設定の変更

```python
# デフォルトモデルの変更
llm = LocalLLMServer("deepseek-coder:6.7b-instruct-q4_K_M")

# 複数モデル対応
models = {
    "coding": "qwen2.5-coder:14b-instruct-q4_K_M",
    "general": "qwen2.5:14b-instruct-q4_K_M",
    "light": "mistral-nemo:12b-instruct-q5_K_M"
}
```

### プロンプトのカスタマイズ

```python
# システムプロンプトの調整
system_prompt = """You are an expert coding assistant specialized in:
- Clean, efficient code generation
- Best practices and design patterns
- Security-conscious development
- Performance optimization

Always provide:
- Working, tested code
- Comprehensive comments
- Error handling
- Performance considerations"""
```

## 🛠️ トラブルシューティング

### よくある問題

1. **MCPサーバーが起動しない**
```bash
# ログ確認
tail -f ~/.config/claude-desktop/logs/mcp.log

# Pythonパスの確認
which python
```

2. **Ollamaに接続できない**
```bash
# Ollamaの状態確認
ollama ps
curl http://localhost:11434/api/tags

# サービス再起動
ollama serve
```

3. **モデルが見つからない**
```bash
# インストール済みモデル確認
ollama list

# モデルの再ダウンロード
ollama pull qwen2.5-coder:14b-instruct-q4_K_M
```

### パフォーマンス最適化

```python
# Ollamaの設定調整
{
    "temperature": 0.1,      # 一貫性重視
    "top_p": 0.95,          # 品質バランス
    "num_predict": 2048,    # 応答長制限
    "num_ctx": 4096         # コンテキスト長
}
```

### セキュリティ設定

```python
# ファイルアクセス制限
ALLOWED_DIRECTORIES = [
    os.getcwd(),
    os.path.expanduser("~/projects")
]

# 実行可能コマンドの制限
ALLOWED_COMMANDS = ["git", "python", "node", "npm"]
```

## 🎉 使用例

### 1. 新機能の実装
```
Claude Code Prompt:
"Use local LLM to create a user authentication system with JWT tokens in Python Flask"

→ MCPサーバーがローカルLLMでコード生成
→ ファイルに自動保存
→ Claude Codeが結果を表示
```

### 2. バグ修正
```
Claude Code Prompt:
"Analyze app.py for bugs and fix them using the local model"

→ ファイル読み込み + LLM分析
→ 修正版コードを生成
→ バックアップ作成後に上書き
```

### 3. コードレビュー
```
Claude Code Prompt:
"Review the entire codebase with local LLM and provide optimization suggestions"

→ プロジェクト全体をスキャン
→ 各ファイルをLLMで分析
→ 改善提案をレポート形式で生成
```

## 📊 パフォーマンス比較

| 機能 | Claude Code (公式) | ローカルLLM + MCP |
|------|-------------------|-------------------|
| 応答速度 | ⚡ 高速 | 🟡 中程度 |
| プライバシー | 🟡 クラウド | 🟢 完全ローカル |
| カスタマイズ | 🟡 限定的 | 🟢 完全自由 |
| コスト | 💰 従量課金 | 🟢 無料 |
| 専門性 | 🟢 汎用的 | 🟢 カスタマイズ可能 |

## 🔄 今後の拡張

- [ ] 複数LLMモデルの同時利用
- [ ] コード実行環境の統合
- [ ] Gitワークフローの自動化
- [ ] プロジェクトテンプレートの生成
- [ ] 自動テスト生成機能