# scripts/config.py
# scripts/config.py
import os
from pathlib import Path

# ディレクトリ設定
BASE_DIR = Path.home() / ".config" / "aigpt"
MEMORY_DIR = BASE_DIR / "memory"
SUMMARY_DIR = MEMORY_DIR / "summary"

def init_directories():
    BASE_DIR.mkdir(parents=True, exist_ok=True)
    MEMORY_DIR.mkdir(parents=True, exist_ok=True)
    SUMMARY_DIR.mkdir(parents=True, exist_ok=True)

def load_config():
    provider = os.getenv("PROVIDER", "ollama")
    model = os.getenv("MODEL", "syui/ai" if provider == "ollama" else "gpt-4o-mini")
    api_key = os.getenv("OPENAI_API_KEY", "")

    if provider == "ollama":
        return {
            "provider": "ollama",
            "model": model,
            "url": f"{os.getenv('OLLAMA_HOST', 'http://localhost:11434')}/api/generate"
        }
    elif provider == "openai":
        return {
            "provider": "openai",
            "model": model,
            "api_key": api_key,
            "url": f"{os.getenv('OPENAI_API_BASE', 'https://api.openai.com/v1')}/chat/completions"
        }
    elif provider == "mcp":
        return {
            "provider": "mcp",
            "model": model,
            "url": os.getenv("MCP_URL", "http://localhost:5000/chat")
        }
    else:
        raise ValueError(f"Unsupported provider: {provider}")
