# server.py
from fastapi import FastAPI, Body
from fastapi_mcp import FastApiMCP
from pydantic import BaseModel
from memory_store import save_message, load_logs, search_memory as do_search_memory

app = FastAPI()
mcp = FastApiMCP(app, name="aigpt-agent", description="MCP Server for AI memory")

class ChatInput(BaseModel):
    message: str

class MemoryInput(BaseModel):
    sender: str
    message: str

class MemoryQuery(BaseModel):
    query: str

@app.post("/chat", operation_id="chat")
async def chat(input: ChatInput):
    save_message("user", input.message)
    response = f"AI: 「{input.message}」を受け取りました！"
    save_message("ai", response)
    return {"response": response}

@app.post("/memory", operation_id="save_memory")
async def memory_post(input: MemoryInput):
    save_message(input.sender, input.message)
    return {"status": "saved"}

@app.get("/memory", operation_id="get_memory")
async def memory_get():
    return {"messages": load_messages()}

@app.post("/ask_message", operation_id="ask_message")
async def ask_message(input: MemoryQuery):
    results = search_memory(input.query)
    return {
        "response": f"🔎 記憶から {len(results)} 件ヒット:\n" + "\n".join([f"{r['sender']}: {r['message']}" for r in results])
    }

@app.post("/memory/search", operation_id="memory")
async def memory_search(query: MemoryQuery):
    hits = do_search_memory(query.query)
    if not hits:
        return {"result": "🔍 記憶の中に該当する内容は見つかりませんでした。"}
    summary = "\n".join([f"{e['sender']}: {e['message']}" for e in hits])
    return {"result": f"🔎 見つかった記憶:\n{summary}"}

mcp.mount()

if __name__ == "__main__":
    import uvicorn
    print("🚀 Starting MCP server...")
    uvicorn.run(app, host="127.0.0.1", port=5000)
