"""AI Provider integration for response generation"""

import os
from typing import Optional, Dict, List, Any, Protocol
from abc import abstractmethod
import logging
import httpx
from openai import OpenAI
import ollama

from .models import PersonaState, Memory
from .config import Config


class AIProvider(Protocol):
    """Protocol for AI providers"""
    
    @abstractmethod
    async def generate_response(
        self, 
        prompt: str, 
        persona_state: PersonaState,
        memories: List[Memory],
        system_prompt: Optional[str] = None
    ) -> str:
        """Generate a response based on prompt and context"""
        pass


class OllamaProvider:
    """Ollama AI provider"""
    
    def __init__(self, model: str = "qwen2.5", host: str = "http://localhost:11434"):
        self.model = model
        self.host = host
        self.client = ollama.Client(host=host)
        self.logger = logging.getLogger(__name__)
    
    async def generate_response(
        self,
        prompt: str,
        persona_state: PersonaState,
        memories: List[Memory],
        system_prompt: Optional[str] = None
    ) -> str:
        """Generate response using Ollama"""
        
        # Build context from memories
        memory_context = "\n".join([
            f"[{mem.level.value}] {mem.content[:200]}..."
            for mem in memories[:5]
        ])
        
        # Build personality context
        personality_desc = ", ".join([
            f"{trait}: {value:.1f}"
            for trait, value in persona_state.base_personality.items()
        ])
        
        # System prompt with persona context
        full_system_prompt = f"""You are an AI with the following characteristics:
Current mood: {persona_state.current_mood}
Fortune today: {persona_state.fortune.fortune_value}/10
Personality traits: {personality_desc}

Recent memories:
{memory_context}

{system_prompt or 'Respond naturally based on your current state and memories.'}"""
        
        try:
            response = self.client.chat(
                model=self.model,
                messages=[
                    {"role": "system", "content": full_system_prompt},
                    {"role": "user", "content": prompt}
                ]
            )
            return response['message']['content']
        except Exception as e:
            self.logger.error(f"Ollama generation failed: {e}")
            return self._fallback_response(persona_state)
    
    def _fallback_response(self, persona_state: PersonaState) -> str:
        """Fallback response based on mood"""
        mood_responses = {
            "joyful": "That's wonderful! I'm feeling great today!",
            "cheerful": "That sounds nice!",
            "neutral": "I understand.",
            "melancholic": "I see... That's something to think about.",
            "contemplative": "Hmm, let me consider that..."
        }
        return mood_responses.get(persona_state.current_mood, "I see.")


class OpenAIProvider:
    """OpenAI API provider"""
    
    def __init__(self, model: str = "gpt-4o-mini", api_key: Optional[str] = None):
        self.model = model
        # Try to get API key from config first
        config = Config()
        self.api_key = api_key or config.get_api_key("openai") or os.getenv("OPENAI_API_KEY")
        if not self.api_key:
            raise ValueError("OpenAI API key not provided. Set it with: ai-gpt config set providers.openai.api_key YOUR_KEY")
        self.client = OpenAI(api_key=self.api_key)
        self.logger = logging.getLogger(__name__)
    
    async def generate_response(
        self,
        prompt: str,
        persona_state: PersonaState,
        memories: List[Memory],
        system_prompt: Optional[str] = None
    ) -> str:
        """Generate response using OpenAI"""
        
        # Build context similar to Ollama
        memory_context = "\n".join([
            f"[{mem.level.value}] {mem.content[:200]}..."
            for mem in memories[:5]
        ])
        
        personality_desc = ", ".join([
            f"{trait}: {value:.1f}"
            for trait, value in persona_state.base_personality.items()
        ])
        
        full_system_prompt = f"""You are an AI with unique personality traits and memories.
Current mood: {persona_state.current_mood}
Fortune today: {persona_state.fortune.fortune_value}/10
Personality traits: {personality_desc}

Recent memories:
{memory_context}

{system_prompt or 'Respond naturally based on your current state and memories. Be authentic to your mood and personality.'}"""
        
        try:
            response = self.client.chat.completions.create(
                model=self.model,
                messages=[
                    {"role": "system", "content": full_system_prompt},
                    {"role": "user", "content": prompt}
                ],
                temperature=0.7 + (persona_state.fortune.fortune_value - 5) * 0.05  # Vary by fortune
            )
            return response.choices[0].message.content
        except Exception as e:
            self.logger.error(f"OpenAI generation failed: {e}")
            return self._fallback_response(persona_state)
    
    def _fallback_response(self, persona_state: PersonaState) -> str:
        """Fallback response based on mood"""
        mood_responses = {
            "joyful": "What a delightful conversation!",
            "cheerful": "That's interesting!",
            "neutral": "I understand what you mean.",
            "melancholic": "I've been thinking about that too...",
            "contemplative": "That gives me something to ponder..."
        }
        return mood_responses.get(persona_state.current_mood, "I see.")


def create_ai_provider(provider: str, model: str, **kwargs) -> AIProvider:
    """Factory function to create AI providers"""
    if provider == "ollama":
        return OllamaProvider(model=model, **kwargs)
    elif provider == "openai":
        return OpenAIProvider(model=model, **kwargs)
    else:
        raise ValueError(f"Unknown provider: {provider}")