"""
Shell Tools

ai.shellの既存機能をMCPツールとして統合
- コード生成
- ファイル分析
- プロジェクト管理
- LLM統合
"""

from typing import Dict, Any, List, Optional
import os
import subprocess
import tempfile
from pathlib import Path
import requests
from .base_tools import BaseMCPTool, config_manager


class ShellTools(BaseMCPTool):
    """シェルツール（元ai.shell機能）"""
    
    def __init__(self, config_dir: Optional[str] = None):
        super().__init__(config_dir)
        self.ollama_url = "http://localhost:11434"
    
    async def code_with_local_llm(self, prompt: str, language: str = "python") -> Dict[str, Any]:
        """ローカルLLMでコード生成"""
        config = config_manager.load_config()
        model = config.get("providers", {}).get("ollama", {}).get("default_model", "qwen2.5-coder:7b")
        
        system_prompt = f"You are an expert {language} programmer. Generate clean, well-commented code."
        
        try:
            response = requests.post(
                f"{self.ollama_url}/api/generate",
                json={
                    "model": model,
                    "prompt": f"{system_prompt}\\n\\nUser: {prompt}\\n\\nPlease provide the code:",
                    "stream": False,
                    "options": {
                        "temperature": 0.1,
                        "top_p": 0.95,
                    }
                },
                timeout=300
            )
            
            if response.status_code == 200:
                result = response.json()
                code = result.get("response", "")
                return {"code": code, "language": language}
            else:
                return {"error": f"Ollama returned status {response.status_code}"}
                
        except Exception as e:
            return {"error": str(e)}
    
    async def analyze_file(self, file_path: str, analysis_prompt: str = "Analyze this file") -> Dict[str, Any]:
        """ファイルを分析"""
        try:
            if not os.path.exists(file_path):
                return {"error": f"File not found: {file_path}"}
            
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # ファイル拡張子から言語を判定
            ext = Path(file_path).suffix
            language_map = {
                '.py': 'python',
                '.rs': 'rust',
                '.js': 'javascript',
                '.ts': 'typescript',
                '.go': 'go',
                '.java': 'java',
                '.cpp': 'cpp',
                '.c': 'c',
                '.sh': 'shell',
                '.toml': 'toml',
                '.json': 'json',
                '.md': 'markdown'
            }
            language = language_map.get(ext, 'text')
            
            config = config_manager.load_config()
            model = config.get("providers", {}).get("ollama", {}).get("default_model", "qwen2.5-coder:7b")
            
            prompt = f"{analysis_prompt}\\n\\nFile: {file_path}\\nLanguage: {language}\\n\\nContent:\\n{content}"
            
            response = requests.post(
                f"{self.ollama_url}/api/generate",
                json={
                    "model": model,
                    "prompt": prompt,
                    "stream": False,
                },
                timeout=300
            )
            
            if response.status_code == 200:
                result = response.json()
                analysis = result.get("response", "")
                return {
                    "analysis": analysis, 
                    "file_path": file_path,
                    "language": language,
                    "file_size": len(content),
                    "line_count": len(content.split('\\n'))
                }
            else:
                return {"error": f"Analysis failed: {response.status_code}"}
                
        except Exception as e:
            return {"error": str(e)}
    
    async def explain_code(self, code: str, language: str = "python") -> Dict[str, Any]:
        """コードを説明"""
        config = config_manager.load_config()
        model = config.get("providers", {}).get("ollama", {}).get("default_model", "qwen2.5-coder:7b")
        
        prompt = f"Explain this {language} code in detail:\\n\\n{code}"
        
        try:
            response = requests.post(
                f"{self.ollama_url}/api/generate",
                json={
                    "model": model,
                    "prompt": prompt,
                    "stream": False,
                },
                timeout=300
            )
            
            if response.status_code == 200:
                result = response.json()
                explanation = result.get("response", "")
                return {"explanation": explanation}
            else:
                return {"error": f"Explanation failed: {response.status_code}"}
                
        except Exception as e:
            return {"error": str(e)}
    
    async def create_project(self, project_type: str, project_name: str, location: str = ".") -> Dict[str, Any]:
        """プロジェクトを作成"""
        try:
            project_path = Path(location) / project_name
            
            if project_path.exists():
                return {"error": f"Project directory already exists: {project_path}"}
            
            project_path.mkdir(parents=True, exist_ok=True)
            
            # プロジェクトタイプに応じたテンプレートを作成
            if project_type == "rust":
                await self._create_rust_project(project_path)
            elif project_type == "python":
                await self._create_python_project(project_path)
            elif project_type == "node":
                await self._create_node_project(project_path)
            else:
                # 基本的なプロジェクト構造
                (project_path / "src").mkdir()
                (project_path / "README.md").write_text(f"# {project_name}\\n\\nA new {project_type} project.")
            
            return {
                "status": "success",
                "project_path": str(project_path),
                "project_type": project_type,
                "files_created": list(self._get_project_files(project_path))
            }
            
        except Exception as e:
            return {"error": str(e)}
    
    async def _create_rust_project(self, project_path: Path):
        """Rustプロジェクトを作成"""
        # Cargo.toml
        cargo_toml = f"""[package]
name = "{project_path.name}"
version = "0.1.0"
edition = "2021"

[dependencies]
"""
        (project_path / "Cargo.toml").write_text(cargo_toml)
        
        # src/main.rs
        src_dir = project_path / "src"
        src_dir.mkdir()
        (src_dir / "main.rs").write_text('fn main() {\\n    println!("Hello, world!");\\n}\\n')
        
        # README.md
        (project_path / "README.md").write_text(f"# {project_path.name}\\n\\nA Rust project.")
    
    async def _create_python_project(self, project_path: Path):
        """Pythonプロジェクトを作成"""
        # pyproject.toml
        pyproject_toml = f"""[project]
name = "{project_path.name}"
version = "0.1.0"
description = "A Python project"
requires-python = ">=3.8"
dependencies = []

[build-system]
requires = ["setuptools>=61.0", "wheel"]
build-backend = "setuptools.build_meta"
"""
        (project_path / "pyproject.toml").write_text(pyproject_toml)
        
        # src/
        src_dir = project_path / "src" / project_path.name
        src_dir.mkdir(parents=True)
        (src_dir / "__init__.py").write_text("")
        (src_dir / "main.py").write_text('def main():\\n    print("Hello, world!")\\n\\nif __name__ == "__main__":\\n    main()\\n')
        
        # README.md
        (project_path / "README.md").write_text(f"# {project_path.name}\\n\\nA Python project.")
    
    async def _create_node_project(self, project_path: Path):
        """Node.jsプロジェクトを作成"""
        # package.json
        package_json = f"""{{
  "name": "{project_path.name}",
  "version": "1.0.0",
  "description": "A Node.js project",
  "main": "index.js",
  "scripts": {{
    "start": "node index.js",
    "test": "echo \\"Error: no test specified\\" && exit 1"
  }},
  "dependencies": {{}}
}}
"""
        (project_path / "package.json").write_text(package_json)
        
        # index.js
        (project_path / "index.js").write_text('console.log("Hello, world!");\\n')
        
        # README.md
        (project_path / "README.md").write_text(f"# {project_path.name}\\n\\nA Node.js project.")
    
    def _get_project_files(self, project_path: Path) -> List[str]:
        """プロジェクト内のファイル一覧を取得"""
        files = []
        for file_path in project_path.rglob("*"):
            if file_path.is_file():
                files.append(str(file_path.relative_to(project_path)))
        return files
    
    async def execute_command(self, command: str, working_dir: str = ".") -> Dict[str, Any]:
        """シェルコマンドを実行"""
        try:
            result = subprocess.run(
                command,
                shell=True,
                cwd=working_dir,
                capture_output=True,
                text=True,
                timeout=60
            )
            
            return {
                "status": "success" if result.returncode == 0 else "error",
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "command": command,
                "working_dir": working_dir
            }
            
        except subprocess.TimeoutExpired:
            return {"error": "Command timed out"}
        except Exception as e:
            return {"error": str(e)}
    
    async def write_file(self, file_path: str, content: str, backup: bool = True) -> Dict[str, Any]:
        """ファイルを書き込み（バックアップオプション付き）"""
        try:
            file_path_obj = Path(file_path)
            
            # バックアップ作成
            backup_path = None
            if backup and file_path_obj.exists():
                backup_path = f"{file_path}.backup"
                with open(file_path, 'r', encoding='utf-8') as src:
                    with open(backup_path, 'w', encoding='utf-8') as dst:
                        dst.write(src.read())
            
            # ファイル書き込み
            file_path_obj.parent.mkdir(parents=True, exist_ok=True)
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            
            return {
                "status": "success",
                "file_path": file_path,
                "backup_path": backup_path,
                "bytes_written": len(content.encode('utf-8'))
            }
            
        except Exception as e:
            return {"error": str(e)}
    
    def get_tools(self) -> List[Dict[str, Any]]:
        """利用可能なツール一覧"""
        return [
            {
                "name": "generate_code",
                "description": "ローカルLLMでコード生成",
                "parameters": {
                    "prompt": "string",
                    "language": "string (optional, default: python)"
                }
            },
            {
                "name": "analyze_file",
                "description": "ファイルを分析",
                "parameters": {
                    "file_path": "string",
                    "analysis_prompt": "string (optional)"
                }
            },
            {
                "name": "explain_code",
                "description": "コードを説明",
                "parameters": {
                    "code": "string",
                    "language": "string (optional, default: python)"
                }
            },
            {
                "name": "create_project",
                "description": "新しいプロジェクトを作成",
                "parameters": {
                    "project_type": "string (rust/python/node)",
                    "project_name": "string",
                    "location": "string (optional, default: .)"
                }
            },
            {
                "name": "execute_command",
                "description": "シェルコマンドを実行",
                "parameters": {
                    "command": "string",
                    "working_dir": "string (optional, default: .)"
                }
            },
            {
                "name": "write_file",
                "description": "ファイルを書き込み",
                "parameters": {
                    "file_path": "string",
                    "content": "string",
                    "backup": "boolean (optional, default: true)"
                }
            }
        ]
    
    async def execute_tool(self, tool_name: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """ツールを実行"""
        try:
            if tool_name == "generate_code":
                result = await self.code_with_local_llm(
                    prompt=params["prompt"],
                    language=params.get("language", "python")
                )
                return result
            
            elif tool_name == "analyze_file":
                result = await self.analyze_file(
                    file_path=params["file_path"],
                    analysis_prompt=params.get("analysis_prompt", "Analyze this file")
                )
                return result
            
            elif tool_name == "explain_code":
                result = await self.explain_code(
                    code=params["code"],
                    language=params.get("language", "python")
                )
                return result
            
            elif tool_name == "create_project":
                result = await self.create_project(
                    project_type=params["project_type"],
                    project_name=params["project_name"],
                    location=params.get("location", ".")
                )
                return result
            
            elif tool_name == "execute_command":
                result = await self.execute_command(
                    command=params["command"],
                    working_dir=params.get("working_dir", ".")
                )
                return result
            
            elif tool_name == "write_file":
                result = await self.write_file(
                    file_path=params["file_path"],
                    content=params["content"],
                    backup=params.get("backup", True)
                )
                return result
            
            else:
                return {"error": f"Unknown tool: {tool_name}"}
        
        except Exception as e:
            return {"error": str(e)}