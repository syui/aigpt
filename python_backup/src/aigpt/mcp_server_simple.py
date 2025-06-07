"""Simple MCP Server implementation for ai.gpt"""

from mcp import Server
from mcp.types import Tool, TextContent
from pathlib import Path
from typing import Any, Dict, List, Optional
import json

from .persona import Persona
from .ai_provider import create_ai_provider
import subprocess
import os


def create_mcp_server(data_dir: Path, enable_card: bool = False) -> Server:
    """Create MCP server with ai.gpt tools"""
    server = Server("aigpt")
    persona = Persona(data_dir)
    
    @server.tool()
    async def get_memories(limit: int = 10) -> List[Dict[str, Any]]:
        """Get active memories from the AI's memory system"""
        memories = persona.memory.get_active_memories(limit=limit)
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
    
    @server.tool()
    async def get_relationship(user_id: str) -> Dict[str, Any]:
        """Get relationship status with a specific user"""
        rel = persona.relationships.get_or_create_relationship(user_id)
        return {
            "user_id": rel.user_id,
            "status": rel.status.value,
            "score": rel.score,
            "transmission_enabled": rel.transmission_enabled,
            "is_broken": rel.is_broken,
            "total_interactions": rel.total_interactions,
            "last_interaction": rel.last_interaction.isoformat() if rel.last_interaction else None
        }
    
    @server.tool()
    async def process_interaction(user_id: str, message: str, provider: str = "ollama", model: str = "qwen2.5") -> Dict[str, Any]:
        """Process an interaction with a user"""
        ai_provider = create_ai_provider(provider, model)
        response, relationship_delta = persona.process_interaction(user_id, message, ai_provider)
        rel = persona.relationships.get_or_create_relationship(user_id)
        
        return {
            "response": response,
            "relationship_delta": relationship_delta,
            "new_relationship_score": rel.score,
            "transmission_enabled": rel.transmission_enabled,
            "relationship_status": rel.status.value
        }
    
    @server.tool()
    async def get_fortune() -> Dict[str, Any]:
        """Get today's AI fortune"""
        fortune = persona.fortune_system.get_today_fortune()
        modifiers = persona.fortune_system.get_personality_modifier(fortune)
        
        return {
            "value": fortune.fortune_value,
            "date": fortune.date.isoformat(),
            "consecutive_good": fortune.consecutive_good,
            "consecutive_bad": fortune.consecutive_bad,
            "breakthrough": fortune.breakthrough_triggered,
            "personality_modifiers": modifiers
        }
    
    @server.tool()
    async def execute_command(command: str, working_dir: str = ".") -> Dict[str, Any]:
        """Execute a shell command"""
        try:
            import shlex
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
    
    @server.tool()
    async def analyze_file(file_path: str) -> Dict[str, Any]:
        """Analyze a file using AI"""
        try:
            if not os.path.exists(file_path):
                return {"error": f"File not found: {file_path}"}
            
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            ai_provider = create_ai_provider("ollama", "qwen2.5")
            
            prompt = f"Analyze this file and provide insights:\\n\\nFile: {file_path}\\n\\nContent:\\n{content[:2000]}"
            analysis = ai_provider.generate_response(prompt, "You are a code analyst.")
            
            return {
                "analysis": analysis,
                "file_path": file_path,
                "file_size": len(content),
                "line_count": len(content.split('\\n'))
            }
        except Exception as e:
            return {"error": str(e)}
    
    return server


async def main():
    """Run MCP server"""
    import sys
    from mcp import stdio_server
    
    data_dir = Path.home() / ".config" / "syui" / "ai" / "gpt" / "data"
    data_dir.mkdir(parents=True, exist_ok=True)
    
    server = create_mcp_server(data_dir)
    await stdio_server(server)


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())