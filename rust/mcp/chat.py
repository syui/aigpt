# mcp/chat.py
"""
Chat client for aigpt CLI
"""
import sys
import json
import requests
from datetime import datetime
from config import init_directories, load_config, MEMORY_DIR

def save_conversation(user_message, ai_response):
    """会話をファイルに保存"""
    init_directories()
    
    conversation = {
        "timestamp": datetime.now().isoformat(),
        "user": user_message,
        "ai": ai_response
    }
    
    # 日付ごとのファイルに保存
    today = datetime.now().strftime("%Y-%m-%d")
    chat_file = MEMORY_DIR / f"chat_{today}.jsonl"
    
    with open(chat_file, "a", encoding="utf-8") as f:
        f.write(json.dumps(conversation, ensure_ascii=False) + "\n")

def chat_with_ollama(config, message):
    """Ollamaとチャット"""
    try:
        payload = {
            "model": config["model"],
            "prompt": message,
            "stream": False
        }
        
        response = requests.post(config["url"], json=payload, timeout=30)
        response.raise_for_status()
        
        result = response.json()
        return result.get("response", "No response received")
        
    except requests.exceptions.RequestException as e:
        return f"Error connecting to Ollama: {e}"
    except Exception as e:
        return f"Error: {e}"

def chat_with_openai(config, message):
    """OpenAIとチャット"""
    try:
        headers = {
            "Authorization": f"Bearer {config['api_key']}",
            "Content-Type": "application/json"
        }
        
        payload = {
            "model": config["model"],
            "messages": [
                {"role": "user", "content": message}
            ]
        }
        
        response = requests.post(config["url"], json=payload, headers=headers, timeout=30)
        response.raise_for_status()
        
        result = response.json()
        return result["choices"][0]["message"]["content"]
        
    except requests.exceptions.RequestException as e:
        return f"Error connecting to OpenAI: {e}"
    except Exception as e:
        return f"Error: {e}"

def chat_with_mcp(config, message):
    """MCPサーバーとチャット"""
    try:
        payload = {
            "message": message,
            "model": config["model"]
        }
        
        response = requests.post(config["url"], json=payload, timeout=30)
        response.raise_for_status()
        
        result = response.json()
        return result.get("response", "No response received")
        
    except requests.exceptions.RequestException as e:
        return f"Error connecting to MCP server: {e}"
    except Exception as e:
        return f"Error: {e}"

def main():
    if len(sys.argv) != 2:
        print("Usage: python chat.py <message>", file=sys.stderr)
        sys.exit(1)
    
    message = sys.argv[1]
    
    try:
        config = load_config()
        print(f"🤖 Using {config['provider']} with model {config['model']}", file=sys.stderr)
        
        # プロバイダに応じてチャット実行
        if config["provider"] == "ollama":
            response = chat_with_ollama(config, message)
        elif config["provider"] == "openai":
            response = chat_with_openai(config, message)
        elif config["provider"] == "mcp":
            response = chat_with_mcp(config, message)
        else:
            response = f"Unsupported provider: {config['provider']}"
        
        # 会話を保存
        save_conversation(message, response)
        
        # レスポンスを出力
        print(response)
        
    except Exception as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
