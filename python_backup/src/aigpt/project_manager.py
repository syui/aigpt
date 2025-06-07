"""Project management and continuous development logic for ai.shell"""

import json
import os
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime
import subprocess
import hashlib

from .models import Memory
from .ai_provider import AIProvider


class ProjectState:
    """プロジェクトの現在状態を追跡"""
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.files_state: Dict[str, str] = {}  # ファイルパス: ハッシュ
        self.last_analysis: Optional[datetime] = None
        self.project_context: Optional[str] = None
        self.development_goals: List[str] = []
        self.known_patterns: Dict[str, Any] = {}
        
    def scan_project_files(self) -> Dict[str, str]:
        """プロジェクトファイルをスキャンしてハッシュ計算"""
        current_state = {}
        
        # 対象ファイル拡張子
        target_extensions = {'.py', '.js', '.ts', '.rs', '.go', '.java', '.cpp', '.c', '.h'}
        
        for file_path in self.project_root.rglob('*'):
            if (file_path.is_file() and 
                file_path.suffix in target_extensions and
                not any(part.startswith('.') for part in file_path.parts)):
                
                try:
                    with open(file_path, 'r', encoding='utf-8') as f:
                        content = f.read()
                    
                    file_hash = hashlib.md5(content.encode()).hexdigest()
                    relative_path = str(file_path.relative_to(self.project_root))
                    current_state[relative_path] = file_hash
                except Exception:
                    continue
        
        return current_state
    
    def detect_changes(self) -> Dict[str, str]:
        """ファイル変更を検出"""
        current_state = self.scan_project_files()
        changes = {}
        
        # 新規・変更ファイル
        for path, current_hash in current_state.items():
            if path not in self.files_state or self.files_state[path] != current_hash:
                changes[path] = "modified" if path in self.files_state else "added"
        
        # 削除ファイル
        for path in self.files_state:
            if path not in current_state:
                changes[path] = "deleted"
        
        self.files_state = current_state
        return changes


class ContinuousDeveloper:
    """Claude Code的な継続開発機能"""
    
    def __init__(self, project_root: Path, ai_provider: Optional[AIProvider] = None):
        self.project_root = project_root
        self.ai_provider = ai_provider
        self.project_state = ProjectState(project_root)
        self.session_memory: List[str] = []
        
    def load_project_context(self) -> str:
        """プロジェクト文脈を読み込み"""
        context_files = [
            "claude.md", "aishell.md", "README.md", 
            "pyproject.toml", "package.json", "Cargo.toml"
        ]
        
        context_parts = []
        for filename in context_files:
            file_path = self.project_root / filename
            if file_path.exists():
                try:
                    with open(file_path, 'r', encoding='utf-8') as f:
                        content = f.read()
                    context_parts.append(f"## {filename}\n{content}")
                except Exception:
                    continue
        
        return "\n\n".join(context_parts)
    
    def analyze_project_structure(self) -> Dict[str, Any]:
        """プロジェクト構造を分析"""
        analysis = {
            "language": self._detect_primary_language(),
            "framework": self._detect_framework(),
            "structure": self._analyze_file_structure(),
            "dependencies": self._analyze_dependencies(),
            "patterns": self._detect_code_patterns()
        }
        return analysis
    
    def _detect_primary_language(self) -> str:
        """主要言語を検出"""
        file_counts = {}
        for file_path in self.project_root.rglob('*'):
            if file_path.is_file() and file_path.suffix:
                ext = file_path.suffix.lower()
                file_counts[ext] = file_counts.get(ext, 0) + 1
        
        language_map = {
            '.py': 'Python',
            '.js': 'JavaScript', 
            '.ts': 'TypeScript',
            '.rs': 'Rust',
            '.go': 'Go',
            '.java': 'Java'
        }
        
        if file_counts:
            primary_ext = max(file_counts.items(), key=lambda x: x[1])[0]
            return language_map.get(primary_ext, 'Unknown')
        return 'Unknown'
    
    def _detect_framework(self) -> str:
        """フレームワークを検出"""
        frameworks = {
            'fastapi': ['fastapi', 'uvicorn'],
            'django': ['django'],
            'flask': ['flask'],
            'react': ['react'],
            'next.js': ['next'],
            'rust-actix': ['actix-web'],
        }
        
        # pyproject.toml, package.json, Cargo.tomlから依存関係を確認
        for config_file in ['pyproject.toml', 'package.json', 'Cargo.toml']:
            config_path = self.project_root / config_file
            if config_path.exists():
                try:
                    with open(config_path, 'r') as f:
                        content = f.read().lower()
                    
                    for framework, keywords in frameworks.items():
                        if any(keyword in content for keyword in keywords):
                            return framework
                except Exception:
                    continue
        
        return 'Unknown'
    
    def _analyze_file_structure(self) -> Dict[str, List[str]]:
        """ファイル構造を分析"""
        structure = {"directories": [], "key_files": []}
        
        for item in self.project_root.iterdir():
            if item.is_dir() and not item.name.startswith('.'):
                structure["directories"].append(item.name)
            elif item.is_file() and item.name in [
                'main.py', 'app.py', 'index.js', 'main.rs', 'main.go'
            ]:
                structure["key_files"].append(item.name)
        
        return structure
    
    def _analyze_dependencies(self) -> List[str]:
        """依存関係を分析"""
        deps = []
        
        # Python dependencies
        pyproject = self.project_root / "pyproject.toml"
        if pyproject.exists():
            try:
                with open(pyproject, 'r') as f:
                    content = f.read()
                # Simple regex would be better but for now just check for common packages
                common_packages = ['fastapi', 'pydantic', 'uvicorn', 'ollama', 'openai']
                for package in common_packages:
                    if package in content:
                        deps.append(package)
            except Exception:
                pass
        
        return deps
    
    def _detect_code_patterns(self) -> Dict[str, int]:
        """コードパターンを検出"""
        patterns = {
            "classes": 0,
            "functions": 0, 
            "api_endpoints": 0,
            "async_functions": 0
        }
        
        for py_file in self.project_root.rglob('*.py'):
            try:
                with open(py_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                patterns["classes"] += content.count('class ')
                patterns["functions"] += content.count('def ')
                patterns["api_endpoints"] += content.count('@app.')
                patterns["async_functions"] += content.count('async def')
            except Exception:
                continue
        
        return patterns
    
    def suggest_next_steps(self, current_task: Optional[str] = None) -> List[str]:
        """次のステップを提案"""
        if not self.ai_provider:
            return ["AI provider not available for suggestions"]
        
        context = self.load_project_context()
        analysis = self.analyze_project_structure()
        changes = self.project_state.detect_changes()
        
        prompt = f"""
プロジェクト分析に基づいて、次の開発ステップを3-5個提案してください。

## プロジェクト文脈
{context[:1000]}

## 構造分析
言語: {analysis['language']}
フレームワーク: {analysis['framework']}
パターン: {analysis['patterns']}

## 最近の変更
{changes}

## 現在のタスク
{current_task or "特になし"}

具体的で実行可能なステップを提案してください：
"""
        
        try:
            response = self.ai_provider.chat(prompt, max_tokens=300)
            # Simple parsing - in real implementation would be more sophisticated
            steps = [line.strip() for line in response.split('\n') 
                    if line.strip() and (line.strip().startswith('-') or line.strip().startswith('1.'))]
            return steps[:5]
        except Exception as e:
            return [f"Error generating suggestions: {str(e)}"]
    
    def generate_code(self, description: str, file_path: Optional[str] = None) -> str:
        """コード生成"""
        if not self.ai_provider:
            return "AI provider not available for code generation"
        
        context = self.load_project_context()
        analysis = self.analyze_project_structure()
        
        prompt = f"""
以下の仕様に基づいてコードを生成してください。

## プロジェクト文脈
{context[:800]}

## 言語・フレームワーク
言語: {analysis['language']}
フレームワーク: {analysis['framework']}
既存パターン: {analysis['patterns']}

## 生成要求
{description}

{"ファイルパス: " + file_path if file_path else ""}

プロジェクトの既存コードスタイルと一貫性を保ったコードを生成してください：
"""
        
        try:
            return self.ai_provider.chat(prompt, max_tokens=500)
        except Exception as e:
            return f"Error generating code: {str(e)}"
    
    def analyze_file(self, file_path: str) -> str:
        """ファイル分析"""
        full_path = self.project_root / file_path
        if not full_path.exists():
            return f"File not found: {file_path}"
        
        try:
            with open(full_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            return f"Error reading file: {str(e)}"
        
        if not self.ai_provider:
            return f"File contents ({len(content)} chars):\n{content[:200]}..."
        
        context = self.load_project_context()
        
        prompt = f"""
以下のファイルを分析して、改善点や問題点を指摘してください。

## プロジェクト文脈
{context[:500]}

## ファイル: {file_path}
{content[:1500]}

分析内容:
1. コード品質
2. プロジェクトとの整合性  
3. 改善提案
4. 潜在的な問題
"""
        
        try:
            return self.ai_provider.chat(prompt, max_tokens=400)
        except Exception as e:
            return f"Error analyzing file: {str(e)}"