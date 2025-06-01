"""MCP Server for ai.gpt system"""

from typing import Optional, List, Dict, Any
from fastapi_mcp import FastApiMCP
from fastapi import FastAPI
from pathlib import Path
import logging
import subprocess
import os
import shlex
from .ai_provider import create_ai_provider

from .persona import Persona
from .models import Memory, Relationship, PersonaState
from .card_integration import CardIntegration, register_card_tools

logger = logging.getLogger(__name__)


class AIGptMcpServer:
    """MCP Server that exposes ai.gpt functionality to AI assistants"""
    
    def __init__(self, data_dir: Path, enable_card_integration: bool = False):
        self.data_dir = data_dir
        self.persona = Persona(data_dir)
        
        # Create FastAPI app
        self.app = FastAPI(
            title="AI.GPT Memory and Relationship System",
            description="MCP server for ai.gpt system"
        )
        
        # Create MCP server with FastAPI app
        self.server = FastApiMCP(self.app)
        self.card_integration = None
        
        if enable_card_integration:
            self.card_integration = CardIntegration()
        
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
        
        # Register ai.card tools if integration is enabled
        if self.card_integration:
            register_card_tools(self.app, self.card_integration)
        
        # Mount MCP server
        self.server.mount()
    
    def get_server(self) -> FastApiMCP:
        """Get the FastAPI MCP server instance"""
        return self.server
    
    async def close(self):
        """Cleanup resources"""
        if self.card_integration:
            await self.card_integration.close()