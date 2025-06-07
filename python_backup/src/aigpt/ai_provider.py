"""AI Provider integration for response generation"""

import os
import json
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
    
    def __init__(self, model: str = "qwen2.5", host: Optional[str] = None):
        self.model = model
        # Use environment variable OLLAMA_HOST if available, otherwise use config or default
        self.host = host or os.getenv('OLLAMA_HOST', 'http://127.0.0.1:11434')
        # Ensure proper URL format
        if not self.host.startswith('http'):
            self.host = f'http://{self.host}'
        self.client = ollama.Client(host=self.host, timeout=60.0)  # 60秒タイムアウト
        self.logger = logging.getLogger(__name__)
        self.logger.info(f"OllamaProvider initialized with host: {self.host}, model: {self.model}")
        
        # Load system prompt from config
        try:
            config = Config()
            self.config_system_prompt = config.get('providers.ollama.system_prompt')
        except:
            self.config_system_prompt = None
    
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

{system_prompt or self.config_system_prompt or 'Respond naturally based on your current state and memories.'}"""
        
        try:
            response = self.client.chat(
                model=self.model,
                messages=[
                    {"role": "system", "content": full_system_prompt},
                    {"role": "user", "content": prompt}
                ]
            )
            return self._clean_response(response['message']['content'])
        except Exception as e:
            self.logger.error(f"Ollama generation failed: {e}")
            return self._fallback_response(persona_state)
    
    def chat(self, prompt: str, max_tokens: int = 2000) -> str:
        """Simple chat interface"""
        try:
            messages = []
            if self.config_system_prompt:
                messages.append({"role": "system", "content": self.config_system_prompt})
            messages.append({"role": "user", "content": prompt})
            
            response = self.client.chat(
                model=self.model,
                messages=messages,
                options={
                    "num_predict": max_tokens,
                    "temperature": 0.7,
                    "top_p": 0.9,
                },
                stream=False  # ストリーミング無効化で安定性向上
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
    """OpenAI API provider with MCP function calling support"""
    
    def __init__(self, model: str = "gpt-4o-mini", api_key: Optional[str] = None, mcp_client=None):
        self.model = model
        # Try to get API key from config first
        config = Config()
        self.api_key = api_key or config.get_api_key("openai") or os.getenv("OPENAI_API_KEY")
        if not self.api_key:
            raise ValueError("OpenAI API key not provided. Set it with: aigpt config set providers.openai.api_key YOUR_KEY")
        self.client = OpenAI(api_key=self.api_key)
        self.logger = logging.getLogger(__name__)
        self.mcp_client = mcp_client  # For MCP function calling
        
        # Load system prompt from config
        try:
            self.config_system_prompt = config.get('providers.openai.system_prompt')
        except:
            self.config_system_prompt = None
    
    def _get_mcp_tools(self) -> List[Dict[str, Any]]:
        """Generate OpenAI tools from MCP endpoints"""
        if not self.mcp_client or not self.mcp_client.available:
            return []
        
        tools = [
            {
                "type": "function",
                "function": {
                    "name": "get_memories",
                    "description": "過去の会話記憶を取得します。「覚えている」「前回」「以前」などの質問で必ず使用してください",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "取得する記憶の数",
                                "default": 5
                            }
                        }
                    }
                }
            },
            {
                "type": "function", 
                "function": {
                    "name": "search_memories",
                    "description": "特定のトピックについて話した記憶を検索します。「プログラミングについて」「○○について話した」などの質問で使用してください",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "keywords": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "検索キーワードの配列"
                            }
                        },
                        "required": ["keywords"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "get_contextual_memories", 
                    "description": "クエリに関連する文脈的記憶を取得します",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "検索クエリ"
                            },
                            "limit": {
                                "type": "integer", 
                                "description": "取得する記憶の数",
                                "default": 5
                            }
                        },
                        "required": ["query"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "get_relationship",
                    "description": "特定ユーザーとの関係性情報を取得します",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "user_id": {
                                "type": "string",
                                "description": "ユーザーID"
                            }
                        },
                        "required": ["user_id"]
                    }
                }
            }
        ]
        
        # Add ai.card tools if available
        if hasattr(self.mcp_client, 'has_card_tools') and self.mcp_client.has_card_tools:
            card_tools = [
                {
                    "type": "function",
                    "function": {
                        "name": "card_get_user_cards",
                        "description": "ユーザーが所有するカードの一覧を取得します",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "did": {
                                    "type": "string",
                                    "description": "ユーザーのDID"
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "取得するカード数の上限",
                                    "default": 10
                                }
                            },
                            "required": ["did"]
                        }
                    }
                },
                {
                    "type": "function",
                    "function": {
                        "name": "card_draw_card",
                        "description": "ガチャを引いてカードを取得します",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "did": {
                                    "type": "string",
                                    "description": "ユーザーのDID"
                                },
                                "is_paid": {
                                    "type": "boolean",
                                    "description": "有料ガチャかどうか",
                                    "default": False
                                }
                            },
                            "required": ["did"]
                        }
                    }
                },
                {
                    "type": "function",
                    "function": {
                        "name": "card_analyze_collection",
                        "description": "ユーザーのカードコレクションを分析します",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "did": {
                                    "type": "string",
                                    "description": "ユーザーのDID"
                                }
                            },
                            "required": ["did"]
                        }
                    }
                },
                {
                    "type": "function",
                    "function": {
                        "name": "card_get_gacha_stats",
                        "description": "ガチャの統計情報を取得します",
                        "parameters": {
                            "type": "object",
                            "properties": {}
                        }
                    }
                }
            ]
            tools.extend(card_tools)
        
        return tools
    
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

{system_prompt or self.config_system_prompt or 'Respond naturally based on your current state and memories. Be authentic to your mood and personality.'}"""
        
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
    
    async def chat_with_mcp(self, prompt: str, max_tokens: int = 2000, user_id: str = "user") -> str:
        """Chat interface with MCP function calling support"""
        if not self.mcp_client or not self.mcp_client.available:
            return self.chat(prompt, max_tokens)
        
        try:
            # Prepare tools
            tools = self._get_mcp_tools()
            
            # Initial request with tools
            response = self.client.chat.completions.create(
                model=self.model,
                messages=[
                    {"role": "system", "content": self.config_system_prompt or "あなたは記憶システムと関係性データ、カードゲームシステムにアクセスできます。過去の会話、記憶、関係性について質問された時は、必ずツールを使用して正確な情報を取得してください。「覚えている」「前回」「以前」「について話した」「関係」などのキーワードがあれば積極的にツールを使用してください。カード関連の質問（「カード」「コレクション」「ガチャ」「見せて」「持っている」など）では、必ずcard_get_user_cardsやcard_analyze_collectionなどのツールを使用してください。didパラメータには現在会話しているユーザーのID（例：'syui'）を使用してください。"},
                    {"role": "user", "content": prompt}
                ],
                tools=tools,
                tool_choice="auto",
                max_tokens=max_tokens,
                temperature=0.7
            )
            
            message = response.choices[0].message
            
            # Handle tool calls
            if message.tool_calls:
                print(f"🔧 [OpenAI] {len(message.tool_calls)} tools called:")
                for tc in message.tool_calls:
                    print(f"  - {tc.function.name}({tc.function.arguments})")
                
                messages = [
                    {"role": "system", "content": self.config_system_prompt or "必要に応じて利用可能なツールを使って、より正確で詳細な回答を提供してください。"},
                    {"role": "user", "content": prompt},
                    {
                        "role": "assistant", 
                        "content": message.content,
                        "tool_calls": [tc.model_dump() for tc in message.tool_calls]
                    }
                ]
                
                # Execute each tool call
                for tool_call in message.tool_calls:
                    print(f"🌐 [MCP] Executing {tool_call.function.name}...")
                    tool_result = await self._execute_mcp_tool(tool_call, user_id)
                    print(f"✅ [MCP] Result: {str(tool_result)[:100]}...")
                    messages.append({
                        "role": "tool",
                        "tool_call_id": tool_call.id,
                        "name": tool_call.function.name,
                        "content": json.dumps(tool_result, ensure_ascii=False)
                    })
                
                # Get final response with tool outputs
                final_response = self.client.chat.completions.create(
                    model=self.model,
                    messages=messages,
                    max_tokens=max_tokens,
                    temperature=0.7
                )
                
                return final_response.choices[0].message.content
            else:
                return message.content
                
        except Exception as e:
            self.logger.error(f"OpenAI MCP chat failed: {e}")
            return f"申し訳ありません。エラーが発生しました: {e}"
    
    async def _execute_mcp_tool(self, tool_call, context_user_id: str = "user") -> Dict[str, Any]:
        """Execute MCP tool call"""
        try:
            import json
            function_name = tool_call.function.name
            arguments = json.loads(tool_call.function.arguments)
            
            if function_name == "get_memories":
                limit = arguments.get("limit", 5)
                return await self.mcp_client.get_memories(limit) or {"error": "記憶の取得に失敗しました"}
            
            elif function_name == "search_memories":
                keywords = arguments.get("keywords", [])
                return await self.mcp_client.search_memories(keywords) or {"error": "記憶の検索に失敗しました"}
            
            elif function_name == "get_contextual_memories":
                query = arguments.get("query", "")
                limit = arguments.get("limit", 5)
                return await self.mcp_client.get_contextual_memories(query, limit) or {"error": "文脈記憶の取得に失敗しました"}
            
            elif function_name == "get_relationship":
                # 引数のuser_idがない場合はコンテキストから取得
                user_id = arguments.get("user_id", context_user_id)
                if not user_id or user_id == "user":
                    user_id = context_user_id
                # デバッグ用ログ
                print(f"🔍 [DEBUG] get_relationship called with user_id: '{user_id}' (context: '{context_user_id}')")
                result = await self.mcp_client.get_relationship(user_id)
                print(f"🔍 [DEBUG] MCP result: {result}")
                return result or {"error": "関係性の取得に失敗しました"}
            
            # ai.card tools
            elif function_name == "card_get_user_cards":
                did = arguments.get("did", context_user_id)
                limit = arguments.get("limit", 10)
                result = await self.mcp_client.card_get_user_cards(did, limit)
                # Check if ai.card server is not running
                if result and result.get("error") == "ai.card server is not running":
                    return {
                        "error": "ai.cardサーバーが起動していません",
                        "message": "カードシステムを使用するには、別のターミナルで以下のコマンドを実行してください:\ncd card && ./start_server.sh"
                    }
                return result or {"error": "カード一覧の取得に失敗しました"}
            
            elif function_name == "card_draw_card":
                did = arguments.get("did", context_user_id)
                is_paid = arguments.get("is_paid", False)
                result = await self.mcp_client.card_draw_card(did, is_paid)
                if result and result.get("error") == "ai.card server is not running":
                    return {
                        "error": "ai.cardサーバーが起動していません",
                        "message": "カードシステムを使用するには、別のターミナルで以下のコマンドを実行してください:\ncd card && ./start_server.sh"
                    }
                return result or {"error": "ガチャに失敗しました"}
            
            elif function_name == "card_analyze_collection":
                did = arguments.get("did", context_user_id)
                result = await self.mcp_client.card_analyze_collection(did)
                if result and result.get("error") == "ai.card server is not running":
                    return {
                        "error": "ai.cardサーバーが起動していません",
                        "message": "カードシステムを使用するには、別のターミナルで以下のコマンドを実行してください:\ncd card && ./start_server.sh"
                    }
                return result or {"error": "コレクション分析に失敗しました"}
            
            elif function_name == "card_get_gacha_stats":
                result = await self.mcp_client.card_get_gacha_stats()
                if result and result.get("error") == "ai.card server is not running":
                    return {
                        "error": "ai.cardサーバーが起動していません",
                        "message": "カードシステムを使用するには、別のターミナルで以下のコマンドを実行してください:\ncd card && ./start_server.sh"
                    }
                return result or {"error": "ガチャ統計の取得に失敗しました"}
            
            else:
                return {"error": f"未知のツール: {function_name}"}
                
        except Exception as e:
            return {"error": f"ツール実行エラー: {str(e)}"}

    def chat(self, prompt: str, max_tokens: int = 2000) -> str:
        """Simple chat interface without MCP tools"""
        try:
            messages = []
            if self.config_system_prompt:
                messages.append({"role": "system", "content": self.config_system_prompt})
            messages.append({"role": "user", "content": prompt})
            
            response = self.client.chat.completions.create(
                model=self.model,
                messages=messages,
                max_tokens=max_tokens,
                temperature=0.7
            )
            return response.choices[0].message.content
        except Exception as e:
            self.logger.error(f"OpenAI chat failed: {e}")
            return "I'm having trouble connecting to the AI model."
    
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


def create_ai_provider(provider: str = "ollama", model: Optional[str] = None, mcp_client=None, **kwargs) -> AIProvider:
    """Factory function to create AI providers"""
    if provider == "ollama":
        # Get model from config if not provided
        if model is None:
            try:
                from .config import Config
                config = Config()
                model = config.get('providers.ollama.default_model', 'qwen2.5')
            except:
                model = 'qwen2.5'  # Fallback to default
        
        # Try to get host from config if not provided in kwargs
        if 'host' not in kwargs:
            try:
                from .config import Config
                config = Config()
                config_host = config.get('providers.ollama.host')
                if config_host:
                    kwargs['host'] = config_host
            except:
                pass  # Use environment variable or default
        return OllamaProvider(model=model, **kwargs)
    elif provider == "openai":
        # Get model from config if not provided
        if model is None:
            try:
                from .config import Config
                config = Config()
                model = config.get('providers.openai.default_model', 'gpt-4o-mini')
            except:
                model = 'gpt-4o-mini'  # Fallback to default
        return OpenAIProvider(model=model, mcp_client=mcp_client, **kwargs)
    else:
        raise ValueError(f"Unknown provider: {provider}")
