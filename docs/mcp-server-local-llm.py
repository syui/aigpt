#!/usr/bin/env python3
"""
Local LLM MCP Server for Claude Code Integration
Claude Code → MCP Server → Local LLM (Qwen2.5-Coder)
"""

import asyncio
import json
import logging
import requests
import subprocess
import os
from pathlib import Path
from typing import Dict, List, Any, Optional
from mcp.server import Server
from mcp.types import (
    Tool, 
    TextContent, 
    Resource,
    PromptMessage,
    GetPromptResult
)

# ログ設定
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("local-llm-mcp")

class LocalLLMServer:
    def __init__(self, model: str = "qwen2.5-coder:14b-instruct-q4_K_M"):
        self.model = model
        self.ollama_url = "http://localhost:11434"
        self.conversation_history = []
        
    def call_ollama(self, prompt: str, system_prompt: str = "") -> str:
        """Ollamaにリクエストを送信"""
        try:
            full_prompt = f"{system_prompt}\n\nUser: {prompt}\nAssistant:"
            
            response = requests.post(
                f"{self.ollama_url}/api/generate",
                json={
                    "model": self.model,
                    "prompt": full_prompt,
                    "stream": False,
                    "options": {
                        "temperature": 0.1,
                        "top_p": 0.95,
                        "num_predict": 2048,
                        "stop": ["User:", "Human:"]
                    }
                },
                timeout=60
            )
            
            if response.status_code == 200:
                return response.json()["response"].strip()
            else:
                return f"Error: {response.status_code} - {response.text}"
                
        except Exception as e:
            logger.error(f"Ollama call failed: {e}")
            return f"Connection error: {e}"
    
    def get_project_context(self) -> str:
        """現在のプロジェクトの情報を取得"""
        context = []
        
        # 現在のディレクトリ
        cwd = os.getcwd()
        context.append(f"Current directory: {cwd}")
        
        # Git情報
        try:
            git_status = subprocess.run(
                ["git", "status", "--porcelain"], 
                capture_output=True, text=True, cwd=cwd
            )
            if git_status.returncode == 0:
                context.append(f"Git status: {git_status.stdout.strip() or 'Clean'}")
        except:
            context.append("Git: Not a git repository")
        
        # ファイル構造（簡略版）
        try:
            files = []
            for item in Path(cwd).iterdir():
                if not item.name.startswith('.') and item.name not in ['node_modules', '__pycache__']:
                    if item.is_file():
                        files.append(f"📄 {item.name}")
                    elif item.is_dir():
                        files.append(f"📁 {item.name}/")
            
            if files:
                context.append("Project files:")
                context.extend(files[:10])  # 最初の10個まで
                
        except Exception as e:
            context.append(f"File listing error: {e}")
        
        return "\n".join(context)

# MCPサーバーのセットアップ
app = Server("local-llm-mcp")
llm = LocalLLMServer()

@app.tool("code_with_local_llm")
async def code_with_local_llm(
    task: str,
    include_context: bool = True,
    model_override: str = ""
) -> str:
    """
    ローカルLLMでコーディングタスクを実行
    
    Args:
        task: 実行したいコーディングタスク
        include_context: プロジェクトコンテキストを含めるか
        model_override: 使用するモデルを一時的に変更
    """
    logger.info(f"Executing coding task: {task}")
    
    # モデルの一時変更
    original_model = llm.model
    if model_override:
        llm.model = model_override
    
    try:
        # システムプロンプト構築
        system_prompt = """You are an expert coding assistant. You can:
1. Write, analyze, and debug code
2. Explain programming concepts
3. Suggest optimizations and best practices
4. Generate complete, working solutions

Always provide:
- Clear, commented code
- Explanations of your approach
- Any assumptions you've made
- Suggestions for improvements

Format your response clearly with code blocks and explanations."""

        # プロジェクトコンテキストを追加
        if include_context:
            context = llm.get_project_context()
            system_prompt += f"\n\nCurrent project context:\n{context}"
        
        # LLMに送信
        response = llm.call_ollama(task, system_prompt)
        
        return response
        
    except Exception as e:
        logger.error(f"Code generation failed: {e}")
        return f"❌ Error in code generation: {e}"
    finally:
        # モデルを元に戻す
        llm.model = original_model

@app.tool("read_file_with_analysis")
async def read_file_with_analysis(
    filepath: str,
    analysis_type: str = "general"
) -> str:
    """
    ファイルを読み込んでLLMで分析
    
    Args:
        filepath: 分析するファイルのパス
        analysis_type: 分析タイプ (general, bugs, optimization, documentation)
    """
    logger.info(f"Analyzing file: {filepath}")
    
    try:
        # ファイル読み込み
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
        
        # 分析タイプに応じたプロンプト
        analysis_prompts = {
            "general": "Analyze this code and provide a general overview, including its purpose, structure, and key components.",
            "bugs": "Review this code for potential bugs, errors, or issues. Suggest fixes if found.",
            "optimization": "Analyze this code for performance optimizations and suggest improvements.",
            "documentation": "Generate comprehensive documentation for this code, including docstrings and comments."
        }
        
        prompt = f"{analysis_prompts.get(analysis_type, analysis_prompts['general'])}\n\nFile: {filepath}\n\nCode:\n```\n{content}\n```"
        
        system_prompt = "You are a code review expert. Provide detailed, constructive analysis."
        
        response = llm.call_ollama(prompt, system_prompt)
        
        return f"📋 Analysis of {filepath}:\n\n{response}"
        
    except FileNotFoundError:
        return f"❌ File not found: {filepath}"
    except Exception as e:
        logger.error(f"File analysis failed: {e}")
        return f"❌ Error analyzing file: {e}"

@app.tool("write_code_to_file")
async def write_code_to_file(
    filepath: str,
    task_description: str,
    overwrite: bool = False
) -> str:
    """
    LLMでコードを生成してファイルに書き込み
    
    Args:
        filepath: 書き込み先のファイルパス
        task_description: コード生成のタスク説明
        overwrite: 既存ファイルを上書きするか
    """
    logger.info(f"Generating code for file: {filepath}")
    
    try:
        # 既存ファイルのチェック
        if os.path.exists(filepath) and not overwrite:
            return f"❌ File already exists: {filepath}. Use overwrite=true to replace."
        
        # ファイル拡張子から言語を推定
        ext = Path(filepath).suffix.lower()
        language_map = {
            '.py': 'Python',
            '.js': 'JavaScript',
            '.ts': 'TypeScript',
            '.java': 'Java',
            '.cpp': 'C++',
            '.c': 'C',
            '.rs': 'Rust',
            '.go': 'Go'
        }
        language = language_map.get(ext, 'appropriate language')
        
        # コード生成プロンプト
        prompt = f"""Generate {language} code for the following task and save it to {filepath}:

Task: {task_description}

Requirements:
- Write complete, working code
- Include appropriate comments
- Follow best practices for {language}
- Make the code production-ready

Return ONLY the code that should be saved to the file, without any additional explanation or markdown formatting."""
        
        system_prompt = f"You are an expert {language} developer. Generate clean, efficient, well-documented code."
        
        # コード生成
        generated_code = llm.call_ollama(prompt, system_prompt)
        
        # ファイルに書き込み
        os.makedirs(os.path.dirname(filepath), exist_ok=True)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(generated_code)
        
        return f"✅ Code generated and saved to {filepath}\n\nGenerated code:\n```{language.lower()}\n{generated_code}\n```"
        
    except Exception as e:
        logger.error(f"Code generation and file writing failed: {e}")
        return f"❌ Error: {e}"

@app.tool("debug_with_llm")
async def debug_with_llm(
    error_message: str,
    code_context: str = "",
    filepath: str = ""
) -> str:
    """
    エラーメッセージとコードコンテキストでデバッグ支援
    
    Args:
        error_message: エラーメッセージ
        code_context: エラーが発生したコードの部分
        filepath: エラーが発生したファイル（オプション）
    """
    logger.info("Debugging with LLM")
    
    try:
        # ファイルが指定されていれば読み込み
        if filepath and os.path.exists(filepath):
            with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                file_content = f.read()
            code_context = f"Full file content:\n{file_content}"
        
        prompt = f"""Help debug this error:

Error message: {error_message}

Code context:
{code_context}

Please:
1. Explain what's causing the error
2. Provide a specific solution
3. Show the corrected code if applicable
4. Suggest ways to prevent similar errors"""
        
        system_prompt = "You are an expert debugger. Provide clear, actionable solutions to programming errors."
        
        response = llm.call_ollama(prompt, system_prompt)
        
        return f"🔧 Debug Analysis:\n\n{response}"
        
    except Exception as e:
        logger.error(f"Debugging failed: {e}")
        return f"❌ Debug error: {e}"

@app.tool("explain_code")
async def explain_code(
    code: str,
    detail_level: str = "medium"
) -> str:
    """
    コードの説明を生成
    
    Args:
        code: 説明するコード
        detail_level: 説明の詳細レベル (basic, medium, detailed)
    """
    logger.info("Explaining code with LLM")
    
    try:
        detail_prompts = {
            "basic": "Provide a brief, high-level explanation of what this code does.",
            "medium": "Explain this code in detail, including its purpose, how it works, and key components.",
            "detailed": "Provide a comprehensive explanation including line-by-line analysis, design patterns used, and potential improvements."
        }
        
        prompt = f"{detail_prompts.get(detail_level, detail_prompts['medium'])}\n\nCode:\n```\n{code}\n```"
        
        system_prompt = "You are a programming instructor. Explain code clearly and educationally."
        
        response = llm.call_ollama(prompt, system_prompt)
        
        return f"📚 Code Explanation:\n\n{response}"
        
    except Exception as e:
        logger.error(f"Code explanation failed: {e}")
        return f"❌ Explanation error: {e}"

@app.tool("switch_model")
async def switch_model(model_name: str) -> str:
    """
    使用するローカルLLMモデルを切り替え
    
    Args:
        model_name: 切り替え先のモデル名
    """
    logger.info(f"Switching model to: {model_name}")
    
    try:
        # モデルの存在確認
        response = requests.get(f"{llm.ollama_url}/api/tags")
        if response.status_code == 200:
            models = response.json().get("models", [])
            available_models = [model["name"] for model in models]
            
            if model_name in available_models:
                llm.model = model_name
                return f"✅ Model switched to: {model_name}"
            else:
                return f"❌ Model not found. Available models: {', '.join(available_models)}"
        else:
            return "❌ Cannot check available models"
            
    except Exception as e:
        logger.error(f"Model switching failed: {e}")
        return f"❌ Error switching model: {e}"

async def main():
    """MCPサーバーを起動"""
    logger.info("Starting Local LLM MCP Server...")
    logger.info(f"Using model: {llm.model}")
    
    # Ollamaの接続確認
    try:
        response = requests.get(f"{llm.ollama_url}/api/tags", timeout=5)
        if response.status_code == 200:
            logger.info("✅ Ollama connection successful")
        else:
            logger.warning("⚠️ Ollama connection issue")
    except Exception as e:
        logger.error(f"❌ Cannot connect to Ollama: {e}")
    
    # サーバー起動
    await app.run()

if __name__ == "__main__":
    asyncio.run(main())