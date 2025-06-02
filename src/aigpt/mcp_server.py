"""MCP Server for ai.gpt system"""

from typing import Optional, List, Dict, Any
from fastapi_mcp import FastApiMCP
from fastapi import FastAPI
from pathlib import Path
import logging
import subprocess
import os
import shlex
import httpx
import json
from .ai_provider import create_ai_provider

from .persona import Persona
from .models import Memory, Relationship, PersonaState

logger = logging.getLogger(__name__)


class AIGptMcpServer:
    """MCP Server that exposes ai.gpt functionality to AI assistants"""
    
    def __init__(self, data_dir: Path):
        self.data_dir = data_dir
        self.persona = Persona(data_dir)
        
        # Create FastAPI app
        self.app = FastAPI(
            title="AI.GPT Memory and Relationship System",
            description="MCP server for ai.gpt system"
        )
        
        # Create MCP server with FastAPI app
        self.server = FastApiMCP(self.app)
        
        self._register_tools()
    
    def _register_tools(self):
        """Register all MCP tools"""
        
        @self.app.get("/get_memories", operation_id="get_memories")
        async def get_memories(user_id: Optional[str] = None, limit: int = 10) -> List[Dict[str, Any]]:
            """Get active memories from the AI's memory system"""
            memories = self.persona.memory.get_active_memories(limit=limit)
            return [
                {
                    "id": mem.id,
                    "content": mem.content,
                    "level": mem.level.value,
                    "importance": mem.importance_score,
                    "is_core": mem.is_core,
                    "timestamp": mem.timestamp.isoformat()
                }
                for mem in memories
            ]
        
        @self.app.get("/get_contextual_memories", operation_id="get_contextual_memories")
        async def get_contextual_memories(query: str = "", limit: int = 10) -> Dict[str, List[Dict[str, Any]]]:
            """Get memories organized by priority with contextual relevance"""
            memory_groups = self.persona.memory.get_contextual_memories(query=query, limit=limit)
            
            result = {}
            for group_name, memories in memory_groups.items():
                result[group_name] = [
                    {
                        "id": mem.id,
                        "content": mem.content,
                        "level": mem.level.value,
                        "importance": mem.importance_score,
                        "is_core": mem.is_core,
                        "timestamp": mem.timestamp.isoformat(),
                        "summary": mem.summary,
                        "metadata": mem.metadata
                    }
                    for mem in memories
                ]
            return result
        
        @self.app.post("/search_memories", operation_id="search_memories")
        async def search_memories(keywords: List[str], memory_types: Optional[List[str]] = None) -> List[Dict[str, Any]]:
            """Search memories by keywords and optionally filter by memory types"""
            from .models import MemoryLevel
            
            # Convert string memory types to enum if provided
            level_filter = None
            if memory_types:
                level_filter = []
                for mt in memory_types:
                    try:
                        level_filter.append(MemoryLevel(mt))
                    except ValueError:
                        pass  # Skip invalid memory types
            
            memories = self.persona.memory.search_memories(keywords, memory_types=level_filter)
            return [
                {
                    "id": mem.id,
                    "content": mem.content,
                    "level": mem.level.value,
                    "importance": mem.importance_score,
                    "is_core": mem.is_core,
                    "timestamp": mem.timestamp.isoformat(),
                    "summary": mem.summary,
                    "metadata": mem.metadata
                }
                for mem in memories
            ]
        
        @self.app.post("/create_summary", operation_id="create_summary")
        async def create_summary(user_id: str) -> Dict[str, Any]:
            """Create an AI-powered summary of recent memories"""
            try:
                ai_provider = create_ai_provider()
                summary = self.persona.memory.create_smart_summary(user_id, ai_provider=ai_provider)
                
                if summary:
                    return {
                        "success": True,
                        "summary": {
                            "id": summary.id,
                            "content": summary.content,
                            "level": summary.level.value,
                            "importance": summary.importance_score,
                            "timestamp": summary.timestamp.isoformat(),
                            "metadata": summary.metadata
                        }
                    }
                else:
                    return {"success": False, "reason": "Not enough memories to summarize"}
            except Exception as e:
                logger.error(f"Failed to create summary: {e}")
                return {"success": False, "reason": str(e)}
        
        @self.app.post("/create_core_memory", operation_id="create_core_memory")
        async def create_core_memory() -> Dict[str, Any]:
            """Create a core memory by analyzing all existing memories"""
            try:
                ai_provider = create_ai_provider()
                core_memory = self.persona.memory.create_core_memory(ai_provider=ai_provider)
                
                if core_memory:
                    return {
                        "success": True,
                        "core_memory": {
                            "id": core_memory.id,
                            "content": core_memory.content,
                            "level": core_memory.level.value,
                            "importance": core_memory.importance_score,
                            "timestamp": core_memory.timestamp.isoformat(),
                            "metadata": core_memory.metadata
                        }
                    }
                else:
                    return {"success": False, "reason": "Not enough memories to create core memory"}
            except Exception as e:
                logger.error(f"Failed to create core memory: {e}")
                return {"success": False, "reason": str(e)}
        
        @self.app.get("/get_relationship", operation_id="get_relationship")
        async def get_relationship(user_id: str) -> Dict[str, Any]:
            """Get relationship status with a specific user"""
            rel = self.persona.relationships.get_or_create_relationship(user_id)
            return {
                "user_id": rel.user_id,
                "status": rel.status.value,
                "score": rel.score,
                "transmission_enabled": rel.transmission_enabled,
                "is_broken": rel.is_broken,
                "total_interactions": rel.total_interactions,
                "last_interaction": rel.last_interaction.isoformat() if rel.last_interaction else None
            }
        
        @self.app.get("/get_all_relationships", operation_id="get_all_relationships")
        async def get_all_relationships() -> List[Dict[str, Any]]:
            """Get all relationships"""
            relationships = []
            for user_id, rel in self.persona.relationships.relationships.items():
                relationships.append({
                    "user_id": user_id,
                    "status": rel.status.value,
                    "score": rel.score,
                    "transmission_enabled": rel.transmission_enabled,
                    "is_broken": rel.is_broken
                })
            return relationships
        
        @self.app.get("/get_persona_state", operation_id="get_persona_state")
        async def get_persona_state() -> Dict[str, Any]:
            """Get current persona state including fortune and mood"""
            state = self.persona.get_current_state()
            return {
                "mood": state.current_mood,
                "fortune": {
                    "value": state.fortune.fortune_value,
                    "date": state.fortune.date.isoformat(),
                    "breakthrough": state.fortune.breakthrough_triggered
                },
                "personality": state.base_personality,
                "active_memory_count": len(state.active_memories)
            }
        
        @self.app.post("/get_context_prompt", operation_id="get_context_prompt")
        async def get_context_prompt(user_id: str, message: str) -> Dict[str, Any]:
            """Get context-aware prompt for AI response generation"""
            try:
                context_prompt = self.persona.build_context_prompt(user_id, message)
                return {
                    "success": True,
                    "context_prompt": context_prompt,
                    "user_id": user_id,
                    "message": message
                }
            except Exception as e:
                logger.error(f"Failed to build context prompt: {e}")
                return {"success": False, "reason": str(e)}
        
        @self.app.post("/process_interaction", operation_id="process_interaction")
        async def process_interaction(user_id: str, message: str) -> Dict[str, Any]:
            """Process an interaction with a user"""
            response, relationship_delta = self.persona.process_interaction(user_id, message)
            rel = self.persona.relationships.get_or_create_relationship(user_id)
            
            return {
                "response": response,
                "relationship_delta": relationship_delta,
                "new_relationship_score": rel.score,
                "transmission_enabled": rel.transmission_enabled,
                "relationship_status": rel.status.value
            }
        
        @self.app.get("/check_transmission_eligibility", operation_id="check_transmission_eligibility")
        async def check_transmission_eligibility(user_id: str) -> Dict[str, Any]:
            """Check if AI can transmit to a specific user"""
            can_transmit = self.persona.can_transmit_to(user_id)
            rel = self.persona.relationships.get_or_create_relationship(user_id)
            
            return {
                "can_transmit": can_transmit,
                "relationship_score": rel.score,
                "threshold": rel.threshold,
                "is_broken": rel.is_broken,
                "transmission_enabled": rel.transmission_enabled
            }
        
        @self.app.get("/get_fortune", operation_id="get_fortune")
        async def get_fortune() -> Dict[str, Any]:
            """Get today's AI fortune"""
            fortune = self.persona.fortune_system.get_today_fortune()
            modifiers = self.persona.fortune_system.get_personality_modifier(fortune)
            
            return {
                "value": fortune.fortune_value,
                "date": fortune.date.isoformat(),
                "consecutive_good": fortune.consecutive_good,
                "consecutive_bad": fortune.consecutive_bad,
                "breakthrough": fortune.breakthrough_triggered,
                "personality_modifiers": modifiers
            }
        
        @self.app.post("/summarize_memories", operation_id="summarize_memories")
        async def summarize_memories(user_id: str) -> Optional[Dict[str, Any]]:
            """Create a summary of recent memories for a user"""
            summary = self.persona.memory.summarize_memories(user_id)
            if summary:
                return {
                    "id": summary.id,
                    "content": summary.content,
                    "level": summary.level.value,
                    "timestamp": summary.timestamp.isoformat()
                }
            return None
        
        @self.app.post("/run_maintenance", operation_id="run_maintenance")
        async def run_maintenance() -> Dict[str, str]:
            """Run daily maintenance tasks"""
            self.persona.daily_maintenance()
            return {"status": "Maintenance completed successfully"}
        
        # Shell integration tools (ai.shell)
        @self.app.post("/execute_command", operation_id="execute_command")
        async def execute_command(command: str, working_dir: str = ".") -> Dict[str, Any]:
            """Execute a shell command"""
            try:
                result = subprocess.run(
                    shlex.split(command),
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
                    "command": command
                }
            except subprocess.TimeoutExpired:
                return {"error": "Command timed out"}
            except Exception as e:
                return {"error": str(e)}
        
        @self.app.post("/analyze_file", operation_id="analyze_file")
        async def analyze_file(file_path: str, analysis_prompt: str = "Analyze this file") -> Dict[str, Any]:
            """Analyze a file using AI"""
            try:
                if not os.path.exists(file_path):
                    return {"error": f"File not found: {file_path}"}
                
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Get AI provider from app state
                ai_provider = getattr(self.app.state, 'ai_provider', 'ollama')
                ai_model = getattr(self.app.state, 'ai_model', 'qwen2.5')
                
                provider = create_ai_provider(ai_provider, ai_model)
                
                # Analyze with AI
                prompt = f"{analysis_prompt}\n\nFile: {file_path}\n\nContent:\n{content}"
                analysis = provider.generate_response(prompt, "You are a code analyst.")
                
                return {
                    "analysis": analysis,
                    "file_path": file_path,
                    "file_size": len(content),
                    "line_count": len(content.split('\n'))
                }
            except Exception as e:
                return {"error": str(e)}
        
        @self.app.post("/write_file", operation_id="write_file")
        async def write_file(file_path: str, content: str, backup: bool = True) -> Dict[str, Any]:
            """Write content to a file"""
            try:
                file_path_obj = Path(file_path)
                
                # Create backup if requested
                backup_path = None
                if backup and file_path_obj.exists():
                    backup_path = f"{file_path}.backup"
                    with open(file_path, 'r', encoding='utf-8') as src:
                        with open(backup_path, 'w', encoding='utf-8') as dst:
                            dst.write(src.read())
                
                # Write file
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
        
        @self.app.get("/read_project_file", operation_id="read_project_file")
        async def read_project_file(file_name: str = "aishell.md") -> Dict[str, Any]:
            """Read project files like aishell.md (similar to claude.md)"""
            try:
                # Check common locations
                search_paths = [
                    Path.cwd() / file_name,
                    Path.cwd() / "docs" / file_name,
                    self.data_dir.parent / file_name,
                ]
                
                for path in search_paths:
                    if path.exists():
                        with open(path, 'r', encoding='utf-8') as f:
                            content = f.read()
                        return {
                            "content": content,
                            "path": str(path),
                            "exists": True
                        }
                
                return {
                    "exists": False,
                    "searched_paths": [str(p) for p in search_paths],
                    "error": f"{file_name} not found"
                }
            except Exception as e:
                return {"error": str(e)}
        
        @self.app.get("/list_files", operation_id="list_files")
        async def list_files(directory: str = ".", pattern: str = "*") -> Dict[str, Any]:
            """List files in a directory"""
            try:
                dir_path = Path(directory)
                if not dir_path.exists():
                    return {"error": f"Directory not found: {directory}"}
                
                files = []
                for item in dir_path.glob(pattern):
                    files.append({
                        "name": item.name,
                        "path": str(item),
                        "is_file": item.is_file(),
                        "is_dir": item.is_dir(),
                        "size": item.stat().st_size if item.is_file() else None
                    })
                
                return {
                    "directory": directory,
                    "pattern": pattern,
                    "files": files,
                    "count": len(files)
                }
            except Exception as e:
                return {"error": str(e)}
        
        # ai.bot integration tools
        @self.app.post("/remote_shell", operation_id="remote_shell")
        async def remote_shell(command: str, ai_bot_url: str = "http://localhost:8080") -> Dict[str, Any]:
            """Execute command via ai.bot /sh functionality (systemd-nspawn isolated execution)"""
            try:
                async with httpx.AsyncClient(timeout=30.0) as client:
                    # ai.bot の /sh エンドポイントに送信
                    response = await client.post(
                        f"{ai_bot_url}/sh",
                        json={"command": command},
                        headers={"Content-Type": "application/json"}
                    )
                    
                    if response.status_code == 200:
                        result = response.json()
                        return {
                            "status": "success",
                            "command": command,
                            "output": result.get("output", ""),
                            "error": result.get("error", ""),
                            "exit_code": result.get("exit_code", 0),
                            "execution_time": result.get("execution_time", ""),
                            "container_id": result.get("container_id", ""),
                            "isolated": True  # systemd-nspawn isolation
                        }
                    else:
                        return {
                            "status": "error",
                            "error": f"ai.bot responded with status {response.status_code}",
                            "response_text": response.text
                        }
            except httpx.TimeoutException:
                return {"status": "error", "error": "Request to ai.bot timed out"}
            except Exception as e:
                return {"status": "error", "error": f"Failed to connect to ai.bot: {str(e)}"}
        
        @self.app.get("/ai_bot_status", operation_id="ai_bot_status")
        async def ai_bot_status(ai_bot_url: str = "http://localhost:8080") -> Dict[str, Any]:
            """Check ai.bot server status and available commands"""
            try:
                async with httpx.AsyncClient(timeout=10.0) as client:
                    response = await client.get(f"{ai_bot_url}/status")
                    
                    if response.status_code == 200:
                        result = response.json()
                        return {
                            "status": "online",
                            "ai_bot_url": ai_bot_url,
                            "server_info": result,
                            "shell_available": True
                        }
                    else:
                        return {
                            "status": "error",
                            "error": f"ai.bot status check failed: {response.status_code}"
                        }
            except Exception as e:
                return {
                    "status": "offline",
                    "error": f"Cannot connect to ai.bot: {str(e)}",
                    "ai_bot_url": ai_bot_url
                }
        
        @self.app.post("/isolated_python", operation_id="isolated_python")
        async def isolated_python(code: str, ai_bot_url: str = "http://localhost:8080") -> Dict[str, Any]:
            """Execute Python code in isolated ai.bot environment"""
            # Python コードを /sh 経由で実行
            python_command = f'python3 -c "{code.replace('"', '\\"')}"'
            return await remote_shell(python_command, ai_bot_url)
        
        @self.app.post("/isolated_analysis", operation_id="isolated_analysis")
        async def isolated_analysis(file_path: str, analysis_type: str = "structure", ai_bot_url: str = "http://localhost:8080") -> Dict[str, Any]:
            """Perform code analysis in isolated environment"""
            if analysis_type == "structure":
                command = f"find {file_path} -type f -name '*.py' | head -20"
            elif analysis_type == "lines":
                command = f"wc -l {file_path}"
            elif analysis_type == "syntax":
                command = f"python3 -m py_compile {file_path}"
            else:
                command = f"file {file_path}"
            
            return await remote_shell(command, ai_bot_url)
        
        # Mount MCP server
        self.server.mount()
    
    def get_server(self) -> FastApiMCP:
        """Get the FastAPI MCP server instance"""
        return self.server
    
    async def close(self):
        """Cleanup resources"""
        pass