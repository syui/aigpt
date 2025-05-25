# mcp/memory_client.py
"""
Memory client for importing and managing ChatGPT conversations
"""
import sys
import json
import requests
from pathlib import Path
from typing import Dict, Any, List

class MemoryClient:
    """記憶機能のクライアント"""
    
    def __init__(self, server_url: str = "http://127.0.0.1:5000"):
        self.server_url = server_url.rstrip('/')
    
    def import_chatgpt_file(self, filepath: str) -> Dict[str, Any]:
        """ChatGPTのエクスポートファイルをインポート"""
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                data = json.load(f)
            
            # ファイルが配列の場合（複数の会話）
            if isinstance(data, list):
                results = []
                for conversation in data:
                    result = self._import_single_conversation(conversation)
                    results.append(result)
                return {
                    "success": True,
                    "imported_count": len([r for r in results if r.get("success")]),
                    "total_count": len(results),
                    "results": results
                }
            else:
                # 単一の会話
                return self._import_single_conversation(data)
                
        except FileNotFoundError:
            return {"success": False, "error": f"File not found: {filepath}"}
        except json.JSONDecodeError as e:
            return {"success": False, "error": f"Invalid JSON: {e}"}
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def _import_single_conversation(self, conversation_data: Dict[str, Any]) -> Dict[str, Any]:
        """単一の会話をインポート"""
        try:
            response = requests.post(
                f"{self.server_url}/memory/import/chatgpt",
                json={"conversation_data": conversation_data},
                timeout=30
            )
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            return {"success": False, "error": f"Server error: {e}"}
    
    def search_memories(self, query: str, limit: int = 10) -> Dict[str, Any]:
        """記憶を検索"""
        try:
            response = requests.post(
                f"{self.server_url}/memory/search",
                json={"query": query, "limit": limit},
                timeout=30
            )
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            return {"success": False, "error": f"Server error: {e}"}
    
    def list_memories(self) -> Dict[str, Any]:
        """記憶一覧を取得"""
        try:
            response = requests.get(f"{self.server_url}/memory/list", timeout=30)
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            return {"success": False, "error": f"Server error: {e}"}
    
    def get_memory_detail(self, filepath: str) -> Dict[str, Any]:
        """記憶の詳細を取得"""
        try:
            response = requests.get(
                f"{self.server_url}/memory/detail",
                params={"filepath": filepath},
                timeout=30
            )
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            return {"success": False, "error": f"Server error: {e}"}
    
    def chat_with_memory(self, message: str, model: str = None) -> Dict[str, Any]:
        """記憶を活用してチャット"""
        try:
            payload = {"message": message}
            if model:
                payload["model"] = model
                
            response = requests.post(
                f"{self.server_url}/chat",
                json=payload,
                timeout=30
            )
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            return {"success": False, "error": f"Server error: {e}"}

def main():
    """コマンドライン インターフェース"""
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python memory_client.py import <chatgpt_export.json>")
        print("  python memory_client.py search <query>")
        print("  python memory_client.py list")
        print("  python memory_client.py detail <filepath>")
        print("  python memory_client.py chat <message>")
        sys.exit(1)
    
    client = MemoryClient()
    command = sys.argv[1]
    
    try:
        if command == "import" and len(sys.argv) == 3:
            filepath = sys.argv[2]
            print(f"🔄 Importing ChatGPT conversations from {filepath}...")
            result = client.import_chatgpt_file(filepath)
            
            if result.get("success"):
                if "imported_count" in result:
                    print(f"✅ Imported {result['imported_count']}/{result['total_count']} conversations")
                else:
                    print("✅ Conversation imported successfully")
                    print(f"📁 Saved to: {result.get('filepath', 'Unknown')}")
            else:
                print(f"❌ Import failed: {result.get('error')}")
        
        elif command == "search" and len(sys.argv) == 3:
            query = sys.argv[2]
            print(f"🔍 Searching for: {query}")
            result = client.search_memories(query)
            
            if result.get("success"):
                memories = result.get("results", [])
                print(f"📚 Found {len(memories)} memories:")
                for memory in memories:
                    print(f"  • {memory.get('title', 'Untitled')}")
                    print(f"    Summary: {memory.get('summary', 'No summary')}")
                    print(f"    Messages: {memory.get('message_count', 0)}")
                    print()
            else:
                print(f"❌ Search failed: {result.get('error')}")
        
        elif command == "list":
            print("📋 Listing all memories...")
            result = client.list_memories()
            
            if result.get("success"):
                memories = result.get("memories", [])
                print(f"📚 Total memories: {len(memories)}")
                for memory in memories:
                    print(f"  • {memory.get('title', 'Untitled')}")
                    print(f"    Source: {memory.get('source', 'Unknown')}")
                    print(f"    Messages: {memory.get('message_count', 0)}")
                    print(f"    Imported: {memory.get('import_time', 'Unknown')}")
                    print()
            else:
                print(f"❌ List failed: {result.get('error')}")
        
        elif command == "detail" and len(sys.argv) == 3:
            filepath = sys.argv[2]
            print(f"📄 Getting details for: {filepath}")
            result = client.get_memory_detail(filepath)
            
            if result.get("success"):
                memory = result.get("memory", {})
                print(f"Title: {memory.get('title', 'Untitled')}")
                print(f"Source: {memory.get('source', 'Unknown')}")
                print(f"Summary: {memory.get('summary', 'No summary')}")
                print(f"Messages: {len(memory.get('messages', []))}")
                print()
                print("Recent messages:")
                for msg in memory.get('messages', [])[:5]:
                    role = msg.get('role', 'unknown')
                    content = msg.get('content', '')[:100]
                    print(f"  {role}: {content}...")
            else:
                print(f"❌ Detail failed: {result.get('error')}")
        
        elif command == "chat" and len(sys.argv) == 3:
            message = sys.argv[2]
            print(f"💬 Chatting with memory: {message}")
            result = client.chat_with_memory(message)
            
            if result.get("success"):
                print(f"🤖 Response: {result.get('response')}")
                print(f"📚 Memories used: {result.get('memories_used', 0)}")
            else:
                print(f"❌ Chat failed: {result.get('error')}")
        
        else:
            print("❌ Invalid command or arguments")
            sys.exit(1)
            
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
