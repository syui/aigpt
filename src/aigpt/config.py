"""Configuration management for ai.gpt"""

import json
import os
from pathlib import Path
from typing import Optional, Dict, Any
import logging


class Config:
    """Manages configuration settings"""
    
    def __init__(self, config_dir: Optional[Path] = None):
        if config_dir is None:
            config_dir = Path.home() / ".config" / "syui" / "ai" / "gpt"
        
        self.config_dir = config_dir
        self.config_file = config_dir / "config.json"
        self.data_dir = config_dir / "data"
        
        # Create directories if they don't exist
        self.config_dir.mkdir(parents=True, exist_ok=True)
        self.data_dir.mkdir(parents=True, exist_ok=True)
        
        self.logger = logging.getLogger(__name__)
        self._config: Dict[str, Any] = {}
        self._load_config()
    
    def _load_config(self):
        """Load configuration from file"""
        if self.config_file.exists():
            try:
                with open(self.config_file, 'r', encoding='utf-8') as f:
                    self._config = json.load(f)
            except Exception as e:
                self.logger.error(f"Failed to load config: {e}")
                self._config = {}
        else:
            # Initialize with default config
            self._config = {
                "providers": {
                    "openai": {
                        "api_key": None,
                        "default_model": "gpt-4o-mini"
                    },
                    "ollama": {
                        "host": "http://localhost:11434",
                        "default_model": "qwen2.5"
                    }
                },
                "atproto": {
                    "handle": None,
                    "password": None,
                    "host": "https://bsky.social"
                },
                "default_provider": "ollama"
            }
            self._save_config()
    
    def _save_config(self):
        """Save configuration to file"""
        try:
            with open(self.config_file, 'w', encoding='utf-8') as f:
                json.dump(self._config, f, indent=2)
        except Exception as e:
            self.logger.error(f"Failed to save config: {e}")
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get configuration value using dot notation"""
        keys = key.split('.')
        value = self._config
        
        for k in keys:
            if isinstance(value, dict) and k in value:
                value = value[k]
            else:
                return default
        
        return value
    
    def set(self, key: str, value: Any):
        """Set configuration value using dot notation"""
        keys = key.split('.')
        config = self._config
        
        # Navigate to the parent dictionary
        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            config = config[k]
        
        # Set the value
        config[keys[-1]] = value
        self._save_config()
    
    def delete(self, key: str) -> bool:
        """Delete configuration value"""
        keys = key.split('.')
        config = self._config
        
        # Navigate to the parent dictionary
        for k in keys[:-1]:
            if k not in config:
                return False
            config = config[k]
        
        # Delete the key if it exists
        if keys[-1] in config:
            del config[keys[-1]]
            self._save_config()
            return True
        
        return False
    
    def list_keys(self, prefix: str = "") -> list[str]:
        """List all configuration keys with optional prefix"""
        def _get_keys(config: dict, current_prefix: str = "") -> list[str]:
            keys = []
            for k, v in config.items():
                full_key = f"{current_prefix}.{k}" if current_prefix else k
                if isinstance(v, dict):
                    keys.extend(_get_keys(v, full_key))
                else:
                    keys.append(full_key)
            return keys
        
        all_keys = _get_keys(self._config)
        
        if prefix:
            return [k for k in all_keys if k.startswith(prefix)]
        return all_keys
    
    def get_api_key(self, provider: str) -> Optional[str]:
        """Get API key for a specific provider"""
        key = self.get(f"providers.{provider}.api_key")
        
        # Also check environment variables
        if not key and provider == "openai":
            key = os.getenv("OPENAI_API_KEY")
        
        return key
    
    def get_provider_config(self, provider: str) -> Dict[str, Any]:
        """Get complete configuration for a provider"""
        return self.get(f"providers.{provider}", {})