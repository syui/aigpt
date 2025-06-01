"""MCP Server for ai.gpt system"""

from typing import Optional, List, Dict, Any
from fastapi_mcp import FastapiMcpServer
from pathlib import Path
import logging

from .persona import Persona
from .models import Memory, Relationship, PersonaState

logger = logging.getLogger(__name__)


class AIGptMcpServer:
    """MCP Server that exposes ai.gpt functionality to AI assistants"""
    
    def __init__(self, data_dir: Path):
        self.data_dir = data_dir
        self.persona = Persona(data_dir)
        self.server = FastapiMcpServer("ai-gpt", "AI.GPT Memory and Relationship System")
        self._register_tools()
    
    def _register_tools(self):
        """Register all MCP tools"""
        
        @self.server.tool("get_memories")
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
        
        @self.server.tool("get_relationship")
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
        
        @self.server.tool("get_all_relationships")
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
        
        @self.server.tool("get_persona_state")
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
        
        @self.server.tool("process_interaction")
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
        
        @self.server.tool("check_transmission_eligibility")
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
        
        @self.server.tool("get_fortune")
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
        
        @self.server.tool("summarize_memories")
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
        
        @self.server.tool("run_maintenance")
        async def run_maintenance() -> Dict[str, str]:
            """Run daily maintenance tasks"""
            self.persona.daily_maintenance()
            return {"status": "Maintenance completed successfully"}
    
    def get_server(self) -> FastapiMcpServer:
        """Get the FastAPI MCP server instance"""
        return self.server