"""Shared AI Provider implementation for ai ecosystem"""

import os
import json
import logging
from typing import Optional, Dict, List, Any, Protocol
from abc import abstractmethod
import httpx
from openai import OpenAI
import ollama


class AIProvider(Protocol):
    """Protocol for AI providers"""
    
    @abstractmethod
    async def chat(self, prompt: str, system_prompt: Optional[str] = None) -> str:
        """Generate a response based on prompt"""
        pass


class OllamaProvider:
    """Ollama AI provider - shared implementation"""
    
    def __init__(self, model: str = "qwen3", host: Optional[str] = None, config_system_prompt: Optional[str] = None):
        self.model = model
        # Use environment variable OLLAMA_HOST if available
        self.host = host or os.getenv('OLLAMA_HOST', 'http://127.0.0.1:11434')
        # Ensure proper URL format
        if not self.host.startswith('http'):
            self.host = f'http://{self.host}'
        self.client = ollama.Client(host=self.host, timeout=60.0)
        self.logger = logging.getLogger(__name__)
        self.logger.info(f"OllamaProvider initialized with host: {self.host}, model: {self.model}")
        self.config_system_prompt = config_system_prompt
    
    async def chat(self, prompt: str, system_prompt: Optional[str] = None) -> str:
        """Simple chat interface"""
        try:
            messages = []
            # Use provided system_prompt, fall back to config_system_prompt
            final_system_prompt = system_prompt or self.config_system_prompt
            if final_system_prompt:
                messages.append({"role": "system", "content": final_system_prompt})
            messages.append({"role": "user", "content": prompt})
            
            response = self.client.chat(
                model=self.model,
                messages=messages,
                options={
                    "num_predict": 2000,
                    "temperature": 0.7,
                    "top_p": 0.9,
                },
                stream=False
            )
            return self._clean_response(response['message']['content'])
        except Exception as e:
            self.logger.error(f"Ollama chat failed (host: {self.host}): {e}")
            return "I'm having trouble connecting to the AI model."
    
    def _clean_response(self, response: str) -> str:
        """Clean response by removing think tags and other unwanted content"""
        import re
        # Remove <think></think> tags and their content
        response = re.sub(r'<think>.*?</think>', '', response, flags=re.DOTALL)
        # Remove any remaining whitespace at the beginning/end
        response = response.strip()
        return response


class OpenAIProvider:
    """OpenAI API provider - shared implementation"""
    
    def __init__(self, model: str = "gpt-4o-mini", api_key: Optional[str] = None, 
                 config_system_prompt: Optional[str] = None, mcp_client=None):
        self.model = model
        self.api_key = api_key or os.getenv("OPENAI_API_KEY")
        if not self.api_key:
            raise ValueError("OpenAI API key not provided")
        self.client = OpenAI(api_key=self.api_key)
        self.logger = logging.getLogger(__name__)
        self.config_system_prompt = config_system_prompt
        self.mcp_client = mcp_client
    
    async def chat(self, prompt: str, system_prompt: Optional[str] = None) -> str:
        """Simple chat interface without MCP tools"""
        try:
            messages = []
            # Use provided system_prompt, fall back to config_system_prompt
            final_system_prompt = system_prompt or self.config_system_prompt
            if final_system_prompt:
                messages.append({"role": "system", "content": final_system_prompt})
            messages.append({"role": "user", "content": prompt})
            
            response = self.client.chat.completions.create(
                model=self.model,
                messages=messages,
                max_tokens=2000,
                temperature=0.7
            )
            return response.choices[0].message.content
        except Exception as e:
            self.logger.error(f"OpenAI chat failed: {e}")
            return "I'm having trouble connecting to the AI model."
    
    def _get_mcp_tools(self) -> List[Dict[str, Any]]:
        """Override this method in subclasses to provide MCP tools"""
        return []
    
    async def chat_with_mcp(self, prompt: str, **kwargs) -> str:
        """Chat interface with MCP function calling support
        
        This method should be overridden in subclasses to provide
        specific MCP functionality.
        """
        if not self.mcp_client:
            return await self.chat(prompt)
        
        # Default implementation - subclasses should override
        return await self.chat(prompt)
    
    async def _execute_mcp_tool(self, tool_call, **kwargs) -> Dict[str, Any]:
        """Execute MCP tool call - override in subclasses"""
        return {"error": "MCP tool execution not implemented"}


def create_ai_provider(provider: str = "ollama", model: Optional[str] = None, 
                      config_system_prompt: Optional[str] = None, mcp_client=None, **kwargs) -> AIProvider:
    """Factory function to create AI providers"""
    if provider == "ollama":
        model = model or "qwen3"
        return OllamaProvider(model=model, config_system_prompt=config_system_prompt, **kwargs)
    elif provider == "openai":
        model = model or "gpt-4o-mini"
        return OpenAIProvider(model=model, config_system_prompt=config_system_prompt, 
                            mcp_client=mcp_client, **kwargs)
    else:
        raise ValueError(f"Unknown provider: {provider}")