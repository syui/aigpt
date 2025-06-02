"""Shared modules for AI ecosystem"""

from .ai_provider import (
    AIProvider,
    OllamaProvider,
    OpenAIProvider,
    create_ai_provider
)

__all__ = [
    'AIProvider',
    'OllamaProvider', 
    'OpenAIProvider',
    'create_ai_provider'
]