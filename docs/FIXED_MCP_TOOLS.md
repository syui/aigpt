# Fixed MCP Tools Issue

## Summary

The issue where AI wasn't calling card tools has been fixed. The problem was:

1. The `chat` command wasn't creating an MCP client when using OpenAI
2. The system prompt in `build_context_prompt` didn't mention available tools

## Changes Made

### 1. Updated `/Users/syui/ai/gpt/src/aigpt/cli.py` (chat command)

Added MCP client creation for OpenAI provider:

```python
# Get config instance
config_instance = Config()

# Get defaults from config if not provided
if not provider:
    provider = config_instance.get("default_provider", "ollama")
if not model:
    if provider == "ollama":
        model = config_instance.get("providers.ollama.default_model", "qwen2.5")
    else:
        model = config_instance.get("providers.openai.default_model", "gpt-4o-mini")

# Create AI provider with MCP client if needed
ai_provider = None
mcp_client = None

try:
    # Create MCP client for OpenAI provider
    if provider == "openai":
        mcp_client = MCPClient(config_instance)
        if mcp_client.available:
            console.print(f"[dim]MCP client connected to {mcp_client.active_server}[/dim]")
    
    ai_provider = create_ai_provider(provider=provider, model=model, mcp_client=mcp_client)
    console.print(f"[dim]Using {provider} with model {model}[/dim]\n")
except Exception as e:
    console.print(f"[yellow]Warning: Could not create AI provider: {e}[/yellow]")
    console.print("[yellow]Falling back to simple responses[/yellow]\n")
```

### 2. Updated `/Users/syui/ai/gpt/src/aigpt/persona.py` (build_context_prompt method)

Added tool instructions to the system prompt:

```python
context_prompt += f"""IMPORTANT: You have access to the following tools:
- Memory tools: get_memories, search_memories, get_contextual_memories
- Relationship tools: get_relationship
- Card game tools: card_get_user_cards, card_draw_card, card_analyze_collection

When asked about cards, collections, or anything card-related, YOU MUST use the card tools.
For "カードコレクションを見せて" or similar requests, use card_get_user_cards with did='{user_id}'.

Respond to this message while staying true to your personality and the established relationship context:

User: {current_message}

AI:"""
```

## Test Results

After the fix:

```bash
$ aigpt chat syui "カードコレクションを見せて"

🔍 [MCP Client] Checking availability...
✅ [MCP Client] ai_gpt server connected successfully
✅ [MCP Client] ai.card tools detected and available
MCP client connected to ai_gpt
Using openai with model gpt-4o-mini

🔧 [OpenAI] 1 tools called:
  - card_get_user_cards({"did":"syui"})
🌐 [MCP] Executing card_get_user_cards...
✅ [MCP] Result: {'error': 'カード一覧の取得に失敗しました'}...
```

The AI is now correctly calling the `card_get_user_cards` tool! The error is expected because the ai.card server needs to be running on port 8000.

## How to Use

1. Start the MCP server:
   ```bash
   aigpt server --port 8001
   ```

2. (Optional) Start the ai.card server:
   ```bash
   cd card && ./start_server.sh
   ```

3. Use the chat command with OpenAI:
   ```bash
   aigpt chat syui "カードコレクションを見せて"
   ```

The AI will now automatically use the card tools when asked about cards!

## Test Script

A test script `/Users/syui/ai/gpt/test_openai_tools.py` is available to test OpenAI API tool calls directly.