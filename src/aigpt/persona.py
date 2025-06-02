"""Persona management system integrating memory, relationships, and fortune"""

import json
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional
import logging

from .models import PersonaState, Conversation
from .memory import MemoryManager
from .relationship import RelationshipTracker
from .fortune import FortuneSystem


class Persona:
    """AI persona with unique characteristics based on interactions"""
    
    def __init__(self, data_dir: Path, name: str = "ai"):
        self.data_dir = data_dir
        self.name = name
        self.memory = MemoryManager(data_dir)
        self.relationships = RelationshipTracker(data_dir)
        self.fortune_system = FortuneSystem(data_dir)
        self.logger = logging.getLogger(__name__)
        
        # Base personality traits
        self.base_personality = {
            "curiosity": 0.7,
            "empathy": 0.8,
            "creativity": 0.6,
            "patience": 0.7,
            "optimism": 0.6
        }
        
        self.state_file = data_dir / "persona_state.json"
        self._load_state()
    
    def _load_state(self):
        """Load persona state from storage"""
        if self.state_file.exists():
            with open(self.state_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                self.base_personality = data.get("base_personality", self.base_personality)
    
    def _save_state(self):
        """Save persona state to storage"""
        state_data = {
            "base_personality": self.base_personality,
            "last_updated": datetime.now().isoformat()
        }
        with open(self.state_file, 'w', encoding='utf-8') as f:
            json.dump(state_data, f, indent=2)
    
    def get_current_state(self) -> PersonaState:
        """Get current persona state including all modifiers"""
        # Get today's fortune
        fortune = self.fortune_system.get_today_fortune()
        fortune_modifiers = self.fortune_system.get_personality_modifier(fortune)
        
        # Apply fortune modifiers to base personality
        current_personality = {}
        for trait, base_value in self.base_personality.items():
            modifier = fortune_modifiers.get(trait, 1.0)
            current_personality[trait] = min(1.0, base_value * modifier)
        
        # Get active memories for context
        active_memories = self.memory.get_active_memories(limit=5)
        
        # Determine mood based on fortune and recent interactions
        mood = self._determine_mood(fortune.fortune_value)
        
        state = PersonaState(
            base_personality=current_personality,
            current_mood=mood,
            fortune=fortune,
            active_memories=[mem.id for mem in active_memories],
            relationship_modifiers={}
        )
        
        return state
    
    def _determine_mood(self, fortune_value: int) -> str:
        """Determine current mood based on fortune and other factors"""
        if fortune_value >= 8:
            return "joyful"
        elif fortune_value >= 6:
            return "cheerful"
        elif fortune_value >= 4:
            return "neutral"
        elif fortune_value >= 2:
            return "melancholic"
        else:
            return "contemplative"
    
    def build_context_prompt(self, user_id: str, current_message: str) -> str:
        """Build context-aware prompt with relevant memories"""
        # Get contextual memories based on current message
        memory_groups = self.memory.get_contextual_memories(query=current_message, limit=8)
        
        # Build context sections
        context_parts = []
        
        # Core personality elements (always included)
        if memory_groups["core"]:
            core_context = "\n".join([mem.content for mem in memory_groups["core"]])
            context_parts.append(f"CORE PERSONALITY:\n{core_context}")
        
        # Recent summaries for context
        if memory_groups["summary"]:
            summary_context = "\n".join([mem.summary or mem.content for mem in memory_groups["summary"]])
            context_parts.append(f"RECENT PATTERNS:\n{summary_context}")
        
        # Recent specific interactions
        if memory_groups["recent"]:
            recent_context = "\n".join([
                f"[{mem.timestamp.strftime('%m-%d')}] {mem.content[:100]}..."
                for mem in memory_groups["recent"][:3]
            ])
            context_parts.append(f"RECENT INTERACTIONS:\n{recent_context}")
        
        # Get current persona state
        state = self.get_current_state()
        
        # Build final prompt
        context_prompt = f"""You are an AI with persistent memory and evolving relationships. Your current state:

PERSONALITY: {', '.join([f'{k}={v:.1f}' for k, v in state.base_personality.items()])}
MOOD: {state.current_mood}
FORTUNE: {state.fortune.fortune_value}/10

"""
        
        if context_parts:
            context_prompt += "RELEVANT CONTEXT:\n" + "\n\n".join(context_parts) + "\n\n"
        
        context_prompt += f"""Respond to this message while staying true to your personality and the established relationship context:

User: {current_message}

AI:"""
        
        return context_prompt
    
    def process_interaction(self, user_id: str, message: str, ai_provider=None) -> tuple[str, float]:
        """Process user interaction and generate response with enhanced context"""
        # Get current state
        state = self.get_current_state()
        
        # Get relationship with user
        relationship = self.relationships.get_or_create_relationship(user_id)
        
        # Enhanced response generation with context awareness
        if relationship.is_broken:
            response = "..."
            relationship_delta = 0.0
        else:
            if ai_provider:
                # Build context-aware prompt
                context_prompt = self.build_context_prompt(user_id, message)
                
                # Generate response using AI with full context
                try:
                    # Check if AI provider supports MCP
                    if hasattr(ai_provider, 'chat_with_mcp'):
                        import asyncio
                        response = asyncio.run(ai_provider.chat_with_mcp(context_prompt, max_tokens=2000, user_id=user_id))
                    else:
                        response = ai_provider.chat(context_prompt, max_tokens=2000)
                    
                    # Clean up response if it includes the prompt echo
                    if "AI:" in response:
                        response = response.split("AI:")[-1].strip()
                        
                except Exception as e:
                    self.logger.error(f"AI response generation failed: {e}")
                    response = f"I appreciate your message about {message[:50]}..."
                
                # Calculate relationship delta based on interaction quality and context
                if state.current_mood in ["joyful", "cheerful"]:
                    relationship_delta = 2.0
                elif relationship.status.value == "close_friend":
                    relationship_delta = 1.5
                else:
                    relationship_delta = 1.0
            else:
                # Context-aware fallback responses
                memory_groups = self.memory.get_contextual_memories(query=message, limit=3)
                
                if memory_groups["core"]:
                    # Reference core memories for continuity
                    response = f"Based on our relationship, I think {message.lower()} connects to what we've discussed before."
                    relationship_delta = 1.5
                elif state.current_mood == "joyful":
                    response = f"What a wonderful day! {message} sounds interesting!"
                    relationship_delta = 2.0
                elif relationship.status.value == "close_friend":
                    response = f"I've been thinking about our conversations. {message}"
                    relationship_delta = 1.5
                else:
                    response = f"I understand. {message}"
                    relationship_delta = 1.0
        
        # Create conversation record
        conv_id = f"{user_id}_{datetime.now().timestamp()}"
        conversation = Conversation(
            id=conv_id,
            user_id=user_id,
            timestamp=datetime.now(),
            user_message=message,
            ai_response=response,
            relationship_delta=relationship_delta,
            memory_created=True
        )
        
        # Update memory
        self.memory.add_conversation(conversation)
        
        # Update relationship
        self.relationships.update_interaction(user_id, relationship_delta)
        
        return response, relationship_delta
    
    def can_transmit_to(self, user_id: str) -> bool:
        """Check if AI can transmit messages to this user"""
        relationship = self.relationships.get_or_create_relationship(user_id)
        return relationship.transmission_enabled and not relationship.is_broken
    
    def daily_maintenance(self):
        """Perform daily maintenance tasks"""
        self.logger.info("Performing daily maintenance...")
        
        # Apply time decay to relationships
        self.relationships.apply_time_decay()
        
        # Apply forgetting to memories
        self.memory.apply_forgetting()
        
        # Identify core memories
        core_memories = self.memory.identify_core_memories()
        if core_memories:
            self.logger.info(f"Identified {len(core_memories)} new core memories")
        
        # Create memory summaries  
        for user_id in self.relationships.relationships:
            try:
                from .ai_provider import create_ai_provider
                ai_provider = create_ai_provider()
                summary = self.memory.create_smart_summary(user_id, ai_provider=ai_provider)
                if summary:
                    self.logger.info(f"Created smart summary for interactions with {user_id}")
            except Exception as e:
                self.logger.warning(f"Could not create AI summary for {user_id}: {e}")
        
        self._save_state()
        self.logger.info("Daily maintenance completed")