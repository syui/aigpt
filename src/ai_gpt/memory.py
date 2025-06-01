"""Memory management system for ai.gpt"""

import json
import hashlib
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Optional, Dict, Any
import logging

from .models import Memory, MemoryLevel, Conversation


class MemoryManager:
    """Manages AI's memory with hierarchical storage and forgetting"""
    
    def __init__(self, data_dir: Path):
        self.data_dir = data_dir
        self.memories_file = data_dir / "memories.json"
        self.conversations_file = data_dir / "conversations.json"
        self.memories: Dict[str, Memory] = {}
        self.conversations: List[Conversation] = []
        self.logger = logging.getLogger(__name__)
        self._load_memories()
    
    def _load_memories(self):
        """Load memories from persistent storage"""
        if self.memories_file.exists():
            with open(self.memories_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                for mem_data in data:
                    memory = Memory(**mem_data)
                    self.memories[memory.id] = memory
        
        if self.conversations_file.exists():
            with open(self.conversations_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                self.conversations = [Conversation(**conv) for conv in data]
    
    def _save_memories(self):
        """Save memories to persistent storage"""
        memories_data = [mem.model_dump(mode='json') for mem in self.memories.values()]
        with open(self.memories_file, 'w', encoding='utf-8') as f:
            json.dump(memories_data, f, indent=2, default=str)
        
        conv_data = [conv.model_dump(mode='json') for conv in self.conversations]
        with open(self.conversations_file, 'w', encoding='utf-8') as f:
            json.dump(conv_data, f, indent=2, default=str)
    
    def add_conversation(self, conversation: Conversation) -> Memory:
        """Add a conversation and create memory from it"""
        self.conversations.append(conversation)
        
        # Create memory from conversation
        memory_id = hashlib.sha256(
            f"{conversation.id}{conversation.timestamp}".encode()
        ).hexdigest()[:16]
        
        memory = Memory(
            id=memory_id,
            timestamp=conversation.timestamp,
            content=f"User: {conversation.user_message}\nAI: {conversation.ai_response}",
            level=MemoryLevel.FULL_LOG,
            importance_score=abs(conversation.relationship_delta) * 0.1
        )
        
        self.memories[memory.id] = memory
        self._save_memories()
        return memory
    
    def summarize_memories(self, user_id: str) -> Optional[Memory]:
        """Create summary from recent memories"""
        recent_memories = [
            mem for mem in self.memories.values()
            if mem.level == MemoryLevel.FULL_LOG
            and (datetime.now() - mem.timestamp).days < 7
        ]
        
        if len(recent_memories) < 5:
            return None
        
        # Simple summary creation (in real implementation, use AI)
        summary_content = f"Summary of {len(recent_memories)} recent interactions"
        summary_id = hashlib.sha256(
            f"summary_{datetime.now().isoformat()}".encode()
        ).hexdigest()[:16]
        
        summary = Memory(
            id=summary_id,
            timestamp=datetime.now(),
            content=summary_content,
            summary=summary_content,
            level=MemoryLevel.SUMMARY,
            importance_score=0.5
        )
        
        self.memories[summary.id] = summary
        
        # Mark summarized memories for potential forgetting
        for mem in recent_memories:
            mem.importance_score *= 0.9
        
        self._save_memories()
        return summary
    
    def identify_core_memories(self) -> List[Memory]:
        """Identify memories that should become core (never forgotten)"""
        core_candidates = [
            mem for mem in self.memories.values()
            if mem.importance_score > 0.8 
            and not mem.is_core
            and mem.level != MemoryLevel.FORGOTTEN
        ]
        
        for memory in core_candidates:
            memory.is_core = True
            memory.level = MemoryLevel.CORE
            self.logger.info(f"Memory {memory.id} promoted to core")
        
        self._save_memories()
        return core_candidates
    
    def apply_forgetting(self):
        """Apply selective forgetting based on importance and time"""
        now = datetime.now()
        
        for memory in self.memories.values():
            if memory.is_core or memory.level == MemoryLevel.FORGOTTEN:
                continue
            
            # Time-based decay
            age_days = (now - memory.timestamp).days
            decay_factor = memory.decay_rate * age_days
            memory.importance_score -= decay_factor
            
            # Forget unimportant old memories
            if memory.importance_score <= 0.1 and age_days > 30:
                memory.level = MemoryLevel.FORGOTTEN
                self.logger.info(f"Memory {memory.id} forgotten")
        
        self._save_memories()
    
    def get_active_memories(self, limit: int = 10) -> List[Memory]:
        """Get currently active memories for persona"""
        active = [
            mem for mem in self.memories.values()
            if mem.level != MemoryLevel.FORGOTTEN
        ]
        
        # Sort by importance and recency
        active.sort(
            key=lambda m: (m.is_core, m.importance_score, m.timestamp),
            reverse=True
        )
        
        return active[:limit]