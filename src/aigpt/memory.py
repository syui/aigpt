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
    
    def add_memory(self, memory: Memory):
        """Add a memory directly to the system"""
        self.memories[memory.id] = memory
        self._save_memories()
    
    def create_smart_summary(self, user_id: str, ai_provider=None) -> Optional[Memory]:
        """Create AI-powered thematic summary from recent memories"""
        recent_memories = [
            mem for mem in self.memories.values()
            if mem.level == MemoryLevel.FULL_LOG
            and (datetime.now() - mem.timestamp).days < 7
        ]
        
        if len(recent_memories) < 5:
            return None
        
        # Sort by timestamp for chronological analysis
        recent_memories.sort(key=lambda m: m.timestamp)
        
        # Prepare conversation context for AI analysis
        conversations_text = "\n\n".join([
            f"[{mem.timestamp.strftime('%Y-%m-%d %H:%M')}] {mem.content}"
            for mem in recent_memories
        ])
        
        summary_prompt = f"""
Analyze these recent conversations and create a thematic summary focusing on:
1. Communication patterns and user preferences
2. Technical topics and problem-solving approaches  
3. Relationship progression and trust level
4. Key recurring themes and interests

Conversations:
{conversations_text}

Create a concise summary (2-3 sentences) that captures the essence of this interaction period:
"""
        
        try:
            if ai_provider:
                summary_content = ai_provider.chat(summary_prompt, max_tokens=200)
            else:
                # Fallback to pattern-based analysis
                themes = self._extract_themes(recent_memories)
                summary_content = f"Themes: {', '.join(themes[:3])}. {len(recent_memories)} interactions with focus on technical discussions."
        except Exception as e:
            self.logger.warning(f"AI summary failed, using fallback: {e}")
            themes = self._extract_themes(recent_memories)
            summary_content = f"Themes: {', '.join(themes[:3])}. {len(recent_memories)} interactions."
        
        summary_id = hashlib.sha256(
            f"summary_{datetime.now().isoformat()}".encode()
        ).hexdigest()[:16]
        
        summary = Memory(
            id=summary_id,
            timestamp=datetime.now(),
            content=f"SUMMARY ({len(recent_memories)} conversations): {summary_content}",
            summary=summary_content,
            level=MemoryLevel.SUMMARY,
            importance_score=0.6,
            metadata={
                "memory_count": len(recent_memories),
                "time_span": f"{recent_memories[0].timestamp.date()} to {recent_memories[-1].timestamp.date()}",
                "themes": self._extract_themes(recent_memories)[:5]
            }
        )
        
        self.memories[summary.id] = summary
        
        # Reduce importance of summarized memories
        for mem in recent_memories:
            mem.importance_score *= 0.8
        
        self._save_memories()
        return summary
    
    def _extract_themes(self, memories: List[Memory]) -> List[str]:
        """Extract common themes from memory content"""
        common_words = {}
        for memory in memories:
            # Simple keyword extraction
            words = memory.content.lower().split()
            for word in words:
                if len(word) > 4 and word.isalpha():
                    common_words[word] = common_words.get(word, 0) + 1
        
        # Return most frequent meaningful words
        return sorted(common_words.keys(), key=common_words.get, reverse=True)[:10]
    
    def create_core_memory(self, ai_provider=None) -> Optional[Memory]:
        """Analyze all memories to extract core personality-forming elements"""
        # Collect all non-forgotten memories for analysis
        all_memories = [
            mem for mem in self.memories.values()
            if mem.level != MemoryLevel.FORGOTTEN
        ]
        
        if len(all_memories) < 10:
            return None
        
        # Sort by importance and timestamp for comprehensive analysis
        all_memories.sort(key=lambda m: (m.importance_score, m.timestamp), reverse=True)
        
        # Prepare memory context for AI analysis
        memory_context = "\n".join([
            f"[{mem.level.value}] {mem.timestamp.strftime('%Y-%m-%d')}: {mem.content[:200]}..."
            for mem in all_memories[:20]  # Top 20 memories
        ])
        
        core_prompt = f"""
Analyze these conversations and memories to identify core personality elements that define this user relationship:

1. Communication style and preferences
2. Core values and principles  
3. Problem-solving patterns
4. Trust level and relationship depth
5. Unique characteristics that make this relationship special

Memories:
{memory_context}

Extract the essential personality-forming elements (2-3 sentences) that should NEVER be forgotten:
"""
        
        try:
            if ai_provider:
                core_content = ai_provider.chat(core_prompt, max_tokens=150)
            else:
                # Fallback to pattern analysis
                user_patterns = self._analyze_user_patterns(all_memories)
                core_content = f"User shows {user_patterns['communication_style']} communication, focuses on {user_patterns['main_interests']}, and demonstrates {user_patterns['problem_solving']} approach."
        except Exception as e:
            self.logger.warning(f"AI core analysis failed, using fallback: {e}")
            user_patterns = self._analyze_user_patterns(all_memories)
            core_content = f"Core pattern: {user_patterns['communication_style']} style, {user_patterns['main_interests']} interests."
        
        # Create core memory
        core_id = hashlib.sha256(
            f"core_{datetime.now().isoformat()}".encode()
        ).hexdigest()[:16]
        
        core_memory = Memory(
            id=core_id,
            timestamp=datetime.now(),
            content=f"CORE PERSONALITY: {core_content}",
            summary=core_content,
            level=MemoryLevel.CORE,
            importance_score=1.0,
            is_core=True,
            metadata={
                "source_memories": len(all_memories),
                "analysis_date": datetime.now().isoformat(),
                "patterns": self._analyze_user_patterns(all_memories)
            }
        )
        
        self.memories[core_memory.id] = core_memory
        self._save_memories()
        
        self.logger.info(f"Core memory created: {core_id}")
        return core_memory
    
    def _analyze_user_patterns(self, memories: List[Memory]) -> Dict[str, str]:
        """Analyze patterns in user behavior from memories"""
        # Extract patterns from conversation content
        all_content = " ".join([mem.content.lower() for mem in memories])
        
        # Simple pattern detection
        communication_indicators = {
            "technical": ["code", "implementation", "system", "api", "database"],
            "casual": ["thanks", "please", "sorry", "help"],
            "formal": ["could", "would", "should", "proper"]
        }
        
        problem_solving_indicators = {
            "systematic": ["first", "then", "next", "step", "plan"],
            "experimental": ["try", "test", "experiment", "see"],
            "theoretical": ["concept", "design", "architecture", "pattern"]
        }
        
        # Score each pattern
        communication_style = max(
            communication_indicators.keys(),
            key=lambda style: sum(all_content.count(word) for word in communication_indicators[style])
        )
        
        problem_solving = max(
            problem_solving_indicators.keys(),
            key=lambda style: sum(all_content.count(word) for word in problem_solving_indicators[style])
        )
        
        # Extract main interests from themes
        themes = self._extract_themes(memories)
        main_interests = ", ".join(themes[:3]) if themes else "general technology"
        
        return {
            "communication_style": communication_style,
            "problem_solving": problem_solving,
            "main_interests": main_interests,
            "interaction_count": len(memories)
        }
    
    def identify_core_memories(self) -> List[Memory]:
        """Identify existing memories that should become core (legacy method)"""
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
        """Get currently active memories for persona (legacy method)"""
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
    
    def get_contextual_memories(self, query: str = "", limit: int = 10) -> Dict[str, List[Memory]]:
        """Get memories organized by priority with contextual relevance"""
        all_memories = [
            mem for mem in self.memories.values()
            if mem.level != MemoryLevel.FORGOTTEN
        ]
        
        # Categorize memories by type and importance
        core_memories = [mem for mem in all_memories if mem.level == MemoryLevel.CORE]
        summary_memories = [mem for mem in all_memories if mem.level == MemoryLevel.SUMMARY]
        recent_memories = [
            mem for mem in all_memories 
            if mem.level == MemoryLevel.FULL_LOG
            and (datetime.now() - mem.timestamp).days < 3
        ]
        
        # Apply keyword relevance if query provided
        if query:
            query_lower = query.lower()
            
            def relevance_score(memory: Memory) -> float:
                content_score = 1 if query_lower in memory.content.lower() else 0
                summary_score = 1 if memory.summary and query_lower in memory.summary.lower() else 0
                metadata_score = 1 if any(
                    query_lower in str(v).lower() 
                    for v in (memory.metadata or {}).values()
                ) else 0
                return content_score + summary_score + metadata_score
            
            # Re-rank by relevance while maintaining type priority
            core_memories.sort(key=lambda m: (relevance_score(m), m.importance_score), reverse=True)
            summary_memories.sort(key=lambda m: (relevance_score(m), m.importance_score), reverse=True)
            recent_memories.sort(key=lambda m: (relevance_score(m), m.importance_score), reverse=True)
        else:
            # Sort by importance and recency
            core_memories.sort(key=lambda m: (m.importance_score, m.timestamp), reverse=True)
            summary_memories.sort(key=lambda m: (m.importance_score, m.timestamp), reverse=True)
            recent_memories.sort(key=lambda m: (m.importance_score, m.timestamp), reverse=True)
        
        # Return organized memory structure
        return {
            "core": core_memories[:3],  # Always include top core memories
            "summary": summary_memories[:3],  # Recent summaries
            "recent": recent_memories[:limit-6],  # Fill remaining with recent
            "all_active": all_memories[:limit]  # Fallback for simple access
        }
    
    def search_memories(self, keywords: List[str], memory_types: List[MemoryLevel] = None) -> List[Memory]:
        """Search memories by keywords and optionally filter by memory types"""
        if memory_types is None:
            memory_types = [MemoryLevel.CORE, MemoryLevel.SUMMARY, MemoryLevel.FULL_LOG]
        
        matching_memories = []
        
        for memory in self.memories.values():
            if memory.level not in memory_types or memory.level == MemoryLevel.FORGOTTEN:
                continue
            
            # Check if any keyword matches in content, summary, or metadata
            content_text = f"{memory.content} {memory.summary or ''}"
            if memory.metadata:
                content_text += " " + " ".join(str(v) for v in memory.metadata.values())
            
            content_lower = content_text.lower()
            
            # Score by keyword matches
            match_score = sum(
                keyword.lower() in content_lower 
                for keyword in keywords
            )
            
            if match_score > 0:
                # Add match score to memory for sorting
                memory_copy = memory.model_copy()
                memory_copy.importance_score += match_score * 0.1
                matching_memories.append(memory_copy)
        
        # Sort by relevance (match score + importance + core status)
        matching_memories.sort(
            key=lambda m: (m.is_core, m.importance_score, m.timestamp),
            reverse=True
        )
        
        return matching_memories