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
    
    def process_interaction(self, user_id: str, message: str, ai_provider=None) -> tuple[str, float]:
        """Process user interaction and generate response"""
        # Get current state
        state = self.get_current_state()
        
        # Get relationship with user
        relationship = self.relationships.get_or_create_relationship(user_id)
        
        # Simple response generation (use AI provider if available)
        if relationship.is_broken:
            response = "..."
            relationship_delta = 0.0
        else:
            if ai_provider:
                # Use AI provider for response generation
                memories = self.memory.get_active_memories(limit=5)
                import asyncio
                response = asyncio.run(
                    ai_provider.generate_response(message, state, memories)
                )
                # Calculate relationship delta based on interaction quality
                if state.current_mood in ["joyful", "cheerful"]:
                    relationship_delta = 2.0
                elif relationship.status.value == "close_friend":
                    relationship_delta = 1.5
                else:
                    relationship_delta = 1.0
            else:
                # Fallback to simple responses
                if state.current_mood == "joyful":
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
            summary = self.memory.summarize_memories(user_id)
            if summary:
                self.logger.info(f"Created summary for interactions with {user_id}")
        
        self._save_state()
        self.logger.info("Daily maintenance completed")