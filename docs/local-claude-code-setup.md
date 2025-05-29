# ローカルClaude Code環境構築ガイド
RTX 4060 Ti + Qwen2.5-Coder + MCP Server

## 1. 必要なツールのインストール

### Ollamaのセットアップ
```bash
# Ollamaのインストール（Windows）
# https://ollama.com からダウンロード

# Qwen2.5-Coderモデルをダウンロード
ollama pull qwen2.5-coder:14b-instruct-q4_K_M
# または7Bバージョン（軽量）
ollama pull qwen2.5-coder:7b-instruct-q4_K_M
```

### Python環境の準備
```bash
# 仮想環境作成
python -m venv claude-code-env
claude-code-env\Scripts\activate  # Windows
# source claude-code-env/bin/activate  # Linux/Mac

# 必要なパッケージをインストール
pip install requests ollama-python rich click pathspec gitpython
```

## 2. メインスクリプトの作成

### claude_code.py
```python
#!/usr/bin/env python3
import os
import sys
import json
import click
import requests
from pathlib import Path
from rich.console import Console
from rich.markdown import Markdown
from rich.syntax import Syntax

console = Console()

class LocalClaudeCode:
    def __init__(self, model="qwen2.5-coder:14b-instruct-q4_K_M"):
        self.model = model
        self.ollama_url = "http://localhost:11434"
        self.conversation_history = []
        self.project_context = ""
        
    def get_project_context(self):
        """プロジェクトのファイル構造とGitステータスを取得"""
        context = []
        
        # ファイル構造
        try:
            for root, dirs, files in os.walk("."):
                # .git, node_modules, __pycache__ などを除外
                dirs[:] = [d for d in dirs if not d.startswith('.') and d not in ['node_modules', '__pycache__']]
                level = root.replace(".", "").count(os.sep)
                indent = " " * 2 * level
                context.append(f"{indent}{os.path.basename(root)}/")
                subindent = " " * 2 * (level + 1)
                for file in files:
                    if not file.startswith('.'):
                        context.append(f"{subindent}{file}")
        except Exception as e:
            context.append(f"Error reading directory: {e}")
            
        return "\n".join(context[:50])  # 最初の50行まで
    
    def read_file(self, filepath):
        """ファイルを読み込む"""
        try:
            with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                return f.read()
        except Exception as e:
            return f"Error reading file: {e}"
    
    def write_file(self, filepath, content):
        """ファイルに書き込む"""
        try:
            os.makedirs(os.path.dirname(filepath), exist_ok=True)
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return f"✅ File written: {filepath}"
        except Exception as e:
            return f"❌ Error writing file: {e}"
    
    def call_ollama(self, prompt):
        """Ollamaにリクエストを送信"""
        try:
            response = requests.post(
                f"{self.ollama_url}/api/generate",
                json={
                    "model": self.model,
                    "prompt": prompt,
                    "stream": False,
                    "options": {
                        "temperature": 0.1,
                        "top_p": 0.95,
                        "num_predict": 2048
                    }
                }
            )
            if response.status_code == 200:
                return response.json()["response"]
            else:
                return f"Error: {response.status_code} - {response.text}"
        except Exception as e:
            return f"Connection error: {e}"
    
    def process_command(self, user_input):
        """ユーザーの指示を処理"""
        # プロジェクトコンテキストを更新
        self.project_context = self.get_project_context()
        
        # システムプロンプト
        system_prompt = f"""You are an expert coding assistant. You can:
1. Read and analyze code files
2. Write and modify files
3. Explain code and provide suggestions
4. Debug and fix issues

Current project structure:
{self.project_context}

When you need to read a file, respond with: READ_FILE: <filepath>
When you need to write a file, respond with: WRITE_FILE: <filepath>
```
<file content>
```

User request: {user_input}
"""
        
        response = self.call_ollama(system_prompt)
        return self.process_response(response)
    
    def process_response(self, response):
        """レスポンスを処理してファイル操作を実行"""
        lines = response.split('\n')
        processed_response = []
        
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            
            if line.startswith("READ_FILE:"):
                filepath = line.replace("READ_FILE:", "").strip()
                content = self.read_file(filepath)
                processed_response.append(f"📁 Reading {filepath}:")
                processed_response.append(f"```\n{content}\n```")
                
            elif line.startswith("WRITE_FILE:"):
                filepath = line.replace("WRITE_FILE:", "").strip()
                i += 1
                # 次の```まで読み込む
                if i < len(lines) and lines[i].strip() == "```":
                    i += 1
                    file_content = []
                    while i < len(lines) and lines[i].strip() != "```":
                        file_content.append(lines[i])
                        i += 1
                    content = '\n'.join(file_content)
                    result = self.write_file(filepath, content)
                    processed_response.append(result)
                else:
                    processed_response.append("❌ Invalid WRITE_FILE format")
            else:
                processed_response.append(line)
            
            i += 1
        
        return '\n'.join(processed_response)

@click.command()
@click.option('--model', default="qwen2.5-coder:14b-instruct-q4_K_M", help='Ollama model to use')
@click.option('--interactive', '-i', is_flag=True, help='Interactive mode')
@click.argument('prompt', required=False)
def main(model, interactive, prompt):
    """Local Claude Code - AI Coding Assistant"""
    
    claude = LocalClaudeCode(model)
    
    if interactive or not prompt:
        console.print("[bold green]🤖 Local Claude Code Assistant[/bold green]")
        console.print(f"Model: {model}")
        console.print("Type 'quit' to exit\n")
        
        while True:
            try:
                user_input = input("👤 You: ").strip()
                if user_input.lower() in ['quit', 'exit', 'q']:
                    break
                
                if user_input:
                    console.print("\n🤖 Assistant:")
                    response = claude.process_command(user_input)
                    console.print(Markdown(response))
                    console.print()
                    
            except KeyboardInterrupt:
                console.print("\n👋 Goodbye!")
                break
    else:
        response = claude.process_command(prompt)
        console.print(response)

if __name__ == "__main__":
    main()
```

## 3. MCP Server統合

### mcp_integration.py
```python
import json
import subprocess
from typing import Dict, List, Any

class MCPIntegration:
    def __init__(self):
        self.servers = {}
    
    def add_server(self, name: str, command: List[str], args: Dict[str, Any] = None):
        """MCPサーバーを追加"""
        self.servers[name] = {
            "command": command,
            "args": args or {}
        }
    
    def call_mcp_tool(self, server_name: str, tool_name: str, arguments: Dict[str, Any]):
        """MCPツールを呼び出す"""
        if server_name not in self.servers:
            return {"error": f"Server {server_name} not found"}
        
        try:
            # MCPサーバーとの通信（JSONRPCベース）
            request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": f"tools/{tool_name}",
                "params": {"arguments": arguments}
            }
            
            process = subprocess.Popen(
                self.servers[server_name]["command"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            
            stdout, stderr = process.communicate(json.dumps(request))
            
            if stderr:
                return {"error": stderr}
            
            return json.loads(stdout)
            
        except Exception as e:
            return {"error": str(e)}

# 使用例
mcp = MCPIntegration()
mcp.add_server("filesystem", ["python", "-m", "mcp_server_filesystem"])
mcp.add_server("git", ["python", "-m", "mcp_server_git"])
```

## 4. 設定ファイル

### config.json
```json
{
  "model": "qwen2.5-coder:14b-instruct-q4_K_M",
  "ollama_url": "http://localhost:11434",
  "mcp_servers": {
    "filesystem": {
      "command": ["python", "-m", "mcp_server_filesystem"],
      "args": {"allowed_directories": ["."]}
    },
    "git": {
      "command": ["python", "-m", "mcp_server_git"]
    }
  },
  "excluded_files": [".git", "node_modules", "__pycache__", "*.pyc"],
  "max_file_size": 1048576
}
```

## 5. 使用方法

### 基本的な使い方
```bash
# インタラクティブモード
python claude_code.py -i

# 単発コマンド
python claude_code.py "Pythonでクイックソートを実装して"

# 特定のモデルを使用
python claude_code.py --model qwen2.5-coder:7b-instruct-q4_K_M -i
```

### MCP Serverのセットアップ
```bash
# 必要なMCPサーバーをインストール
pip install mcp-server-git mcp-server-filesystem

# 設定ファイルを編集してMCPサーバーを有効化
```

## 6. 機能一覧

- ✅ ローカルLLMとの対話
- ✅ ファイル読み書き
- ✅ プロジェクト構造の自動認識
- ✅ Gitステータス表示
- ✅ シンタックスハイライト
- ✅ MCP Server統合（オプション）
- ✅ 設定ファイル対応

## 7. トラブルシューティング

### よくある問題
1. **Ollamaが起動しない**: `ollama serve` でサーバーを起動
2. **モデルが見つからない**: `ollama list` でインストール済みモデルを確認
3. **メモリ不足**: より軽量な7Bモデルを使用
4. **ファイル権限エラー**: 実行権限を確認

### パフォーマンス最適化
- GPU使用を確認: `nvidia-smi` でVRAM使用量をチェック
- モデルサイズの調整: Q4_K_M → Q4_K_S で軽量化
- コンテキスト長を調整して応答速度を向上

重い場合は7Bバージョン（qwen2.5-coder:7b-instruct-q4_K_M）に変更。
