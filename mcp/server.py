# mcp/server.py
"""
Enhanced MCP Server with Memory for aigpt CLI
"""
import json
import os
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Optional
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import uvicorn

# データモデル
class ChatMessage(BaseModel):
    message: str
    model: Optional[str] = None

class MemoryQuery(BaseModel):
    query: str
    limit: Optional[int] = 10

class ConversationImport(BaseModel):
    conversation_data: Dict[str, Any]

# 設定
BASE_DIR = Path.home() / ".config" / "aigpt"
MEMORY_DIR = BASE_DIR / "memory"
CHATGPT_MEMORY_DIR = MEMORY_DIR / "chatgpt"

def init_directories():
    """必要なディレクトリを作成"""
    BASE_DIR.mkdir(parents=True, exist_ok=True)
    MEMORY_DIR.mkdir(parents=True, exist_ok=True)
    CHATGPT_MEMORY_DIR.mkdir(parents=True, exist_ok=True)

class MemoryManager:
    """記憶管理クラス"""
    
    def __init__(self):
        init_directories()
    
    def parse_chatgpt_conversation(self, conversation_data: Dict[str, Any]) -> List[Dict[str, Any]]:
        """ChatGPTの会話データを解析してメッセージを抽出"""
        messages = []
        mapping = conversation_data.get("mapping", {})
        
        # メッセージを時系列順に並べる
        message_nodes = []
        for node_id, node in mapping.items():
            message = node.get("message")
            if message and message.get("content", {}).get("parts"):
                parts = message["content"]["parts"]
                if parts and parts[0].strip():  # 空でないメッセージのみ
                    message_nodes.append({
                        "id": node_id,
                        "create_time": message.get("create_time", 0),
                        "author_role": message["author"]["role"],
                        "content": parts[0],
                        "parent": node.get("parent")
                    })
        
        # 作成時間でソート
        message_nodes.sort(key=lambda x: x["create_time"] or 0)
        
        for msg in message_nodes:
            if msg["author_role"] in ["user", "assistant"]:
                messages.append({
                    "role": msg["author_role"],
                    "content": msg["content"],
                    "timestamp": msg["create_time"],
                    "message_id": msg["id"]
                })
        
        return messages
    
    def save_chatgpt_memory(self, conversation_data: Dict[str, Any]) -> str:
        """ChatGPTの会話を記憶として保存"""
        title = conversation_data.get("title", "untitled")
        create_time = conversation_data.get("create_time", datetime.now().timestamp())
        
        # メッセージを解析
        messages = self.parse_chatgpt_conversation(conversation_data)
        
        if not messages:
            raise ValueError("No valid messages found in conversation")
        
        # 保存データを作成
        memory_data = {
            "title": title,
            "source": "chatgpt",
            "import_time": datetime.now().isoformat(),
            "original_create_time": create_time,
            "messages": messages,
            "summary": self.generate_summary(messages)
        }
        
        # ファイル名を生成（タイトルをサニタイズ）
        safe_title = "".join(c for c in title if c.isalnum() or c in (' ', '-', '_')).rstrip()
        timestamp = datetime.fromtimestamp(create_time).strftime("%Y%m%d_%H%M%S")
        filename = f"{timestamp}_{safe_title[:50]}.json"
        
        filepath = CHATGPT_MEMORY_DIR / filename
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(memory_data, f, ensure_ascii=False, indent=2)
        
        return str(filepath)
    
    def generate_summary(self, messages: List[Dict[str, Any]]) -> str:
        """会話の要約を生成"""
        if not messages:
            return "Empty conversation"
        
        # 簡単な要約を生成（実際のAIによる要約は後で実装可能）
        user_messages = [msg for msg in messages if msg["role"] == "user"]
        assistant_messages = [msg for msg in messages if msg["role"] == "assistant"]
        
        summary = f"Conversation with {len(user_messages)} user messages and {len(assistant_messages)} assistant responses. "
        
        if user_messages:
            first_user_msg = user_messages[0]["content"][:100]
            summary += f"Started with: {first_user_msg}..."
        
        return summary
    
    def search_memories(self, query: str, limit: int = 10) -> List[Dict[str, Any]]:
        """記憶を検索"""
        results = []
        
        # ChatGPTの記憶を検索
        for filepath in CHATGPT_MEMORY_DIR.glob("*.json"):
            try:
                with open(filepath, 'r', encoding='utf-8') as f:
                    memory_data = json.load(f)
                
                # 簡単なキーワード検索
                search_text = f"{memory_data.get('title', '')} {memory_data.get('summary', '')}"
                for msg in memory_data.get('messages', []):
                    search_text += f" {msg.get('content', '')}"
                
                if query.lower() in search_text.lower():
                    results.append({
                        "filepath": str(filepath),
                        "title": memory_data.get("title"),
                        "summary": memory_data.get("summary"),
                        "source": memory_data.get("source"),
                        "import_time": memory_data.get("import_time"),
                        "message_count": len(memory_data.get("messages", []))
                    })
                    
                    if len(results) >= limit:
                        break
                        
            except Exception as e:
                print(f"Error reading memory file {filepath}: {e}")
                continue
        
        return results
    
    def get_memory_detail(self, filepath: str) -> Dict[str, Any]:
        """記憶の詳細を取得"""
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            raise ValueError(f"Error reading memory file: {e}")
    
    def list_all_memories(self) -> List[Dict[str, Any]]:
        """すべての記憶をリスト"""
        memories = []
        
        for filepath in CHATGPT_MEMORY_DIR.glob("*.json"):
            try:
                with open(filepath, 'r', encoding='utf-8') as f:
                    memory_data = json.load(f)
                
                memories.append({
                    "filepath": str(filepath),
                    "title": memory_data.get("title"),
                    "summary": memory_data.get("summary"),
                    "source": memory_data.get("source"),
                    "import_time": memory_data.get("import_time"),
                    "message_count": len(memory_data.get("messages", []))
                })
            except Exception as e:
                print(f"Error reading memory file {filepath}: {e}")
                continue
        
        # インポート時間でソート
        memories.sort(key=lambda x: x.get("import_time", ""), reverse=True)
        return memories

# FastAPI アプリケーション
app = FastAPI(title="AigptMCP Server with Memory", version="1.0.0")
memory_manager = MemoryManager()

@app.post("/memory/import/chatgpt")
async def import_chatgpt_conversation(data: ConversationImport):
    """ChatGPTの会話をインポート"""
    try:
        filepath = memory_manager.save_chatgpt_memory(data.conversation_data)
        return {
            "success": True,
            "message": "Conversation imported successfully",
            "filepath": filepath
        }
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@app.post("/memory/search")
async def search_memories(query: MemoryQuery):
    """記憶を検索"""
    try:
        results = memory_manager.search_memories(query.query, query.limit)
        return {
            "success": True,
            "results": results,
            "count": len(results)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/memory/list")
async def list_memories():
    """すべての記憶をリスト"""
    try:
        memories = memory_manager.list_all_memories()
        return {
            "success": True,
            "memories": memories,
            "count": len(memories)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/memory/detail")
async def get_memory_detail(filepath: str):
    """記憶の詳細を取得"""
    try:
        detail = memory_manager.get_memory_detail(filepath)
        return {
            "success": True,
            "memory": detail
        }
    except Exception as e:
        raise HTTPException(status_code=404, detail=str(e))

@app.post("/chat")
async def chat_endpoint(data: ChatMessage):
    """チャット機能（記憶を活用）"""
    try:
        # 関連する記憶を検索
        memories = memory_manager.search_memories(data.message, limit=3)
        
        # メモリのコンテキストを構築
        memory_context = ""
        if memories:
            memory_context = "\n# Related memories:\n"
            for memory in memories:
                memory_context += f"- {memory['title']}: {memory['summary']}\n"
        
        # 実際のチャット処理（他のプロバイダーに転送）
        enhanced_message = data.message
        if memory_context:
            enhanced_message = f"{data.message}\n\n{memory_context}"
        
        return {
            "success": True,
            "response": f"Enhanced response with memory context: {enhanced_message}",
            "memories_used": len(memories)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/")
async def root():
    """ヘルスチェック"""
    return {
        "service": "AigptMCP Server with Memory",
        "status": "running",
        "memory_dir": str(MEMORY_DIR),
        "endpoints": [
            "/memory/import/chatgpt",
            "/memory/search",
            "/memory/list",
            "/memory/detail",
            "/chat"
        ]
    }

if __name__ == "__main__":
    print("🚀 AigptMCP Server with Memory starting...")
    print(f"📁 Memory directory: {MEMORY_DIR}")
    uvicorn.run(app, host="127.0.0.1", port=5000)
