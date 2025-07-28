use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::memory::MemoryManager;

pub struct BaseMCPServer {
    pub memory_manager: MemoryManager,
}

impl BaseMCPServer {
    pub async fn new() -> Result<Self> {
        let memory_manager = MemoryManager::new().await?;
        Ok(BaseMCPServer { memory_manager })
    }

    pub async fn run(&mut self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        
        let reader = stdin.lock();
        let lines = reader.lines();
        
        for line_result in lines {
            match line_result {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    
                    if let Ok(request) = serde_json::from_str::<Value>(&trimmed) {
                        let response = self.handle_request(request).await;
                        let response_str = serde_json::to_string(&response)?;
                        stdout.write_all(response_str.as_bytes())?;
                        stdout.write_all(b"\n")?;
                        stdout.flush()?;
                    }
                }
                Err(_) => break,
            }
        }
        
        Ok(())
    }

    pub async fn handle_request(&mut self, request: Value) -> Value {
        let method = request["method"].as_str().unwrap_or("");
        let id = request["id"].clone();

        match method {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(request, id).await,
            _ => self.handle_unknown_method(id),
        }
    }

    // 初期化ハンドラ
    fn handle_initialize(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "aigpt",
                    "version": "0.1.0"
                }
            }
        })
    }

    // ツールリストハンドラ (拡張可能)
    pub fn handle_tools_list(&self, id: Value) -> Value {
        let tools = self.get_available_tools();
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        })
    }

    // 基本ツール定義 (拡張で上書き可能)
    pub fn get_available_tools(&self) -> Vec<Value> {
        vec![
            json!({
                "name": "create_memory",
                "description": "Create a new memory entry",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Content of the memory"
                        }
                    },
                    "required": ["content"]
                }
            }),
            json!({
                "name": "search_memories",
                "description": "Search memories by content",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        }
                    },
                    "required": ["query"]
                }
            }),
            json!({
                "name": "update_memory",
                "description": "Update an existing memory entry",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "ID of the memory to update"
                        },
                        "content": {
                            "type": "string",
                            "description": "New content for the memory"
                        }
                    },
                    "required": ["id", "content"]
                }
            }),
            json!({
                "name": "delete_memory",
                "description": "Delete a memory entry",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "ID of the memory to delete"
                        }
                    },
                    "required": ["id"]
                }
            }),
            json!({
                "name": "list_conversations",
                "description": "List all imported conversations",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            })
        ]
    }

    // ツール呼び出しハンドラ
    async fn handle_tools_call(&mut self, request: Value, id: Value) -> Value {
        let tool_name = request["params"]["name"].as_str().unwrap_or("");
        let arguments = &request["params"]["arguments"];

        let result = self.execute_tool(tool_name, arguments).await;

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": result.to_string()
                }]
            }
        })
    }

    // ツール実行 (拡張で上書き可能)
    pub async fn execute_tool(&mut self, tool_name: &str, arguments: &Value) -> Value {
        match tool_name {
            "create_memory" => self.tool_create_memory(arguments),
            "search_memories" => self.tool_search_memories(arguments),
            "update_memory" => self.tool_update_memory(arguments),
            "delete_memory" => self.tool_delete_memory(arguments),
            "list_conversations" => self.tool_list_conversations(),
            _ => json!({
                "success": false,
                "error": format!("Unknown tool: {}", tool_name)
            })
        }
    }

    // 基本ツール実装
    fn tool_create_memory(&mut self, arguments: &Value) -> Value {
        let content = arguments["content"].as_str().unwrap_or("");
        match self.memory_manager.create_memory(content) {
            Ok(id) => json!({
                "success": true,
                "id": id,
                "message": "Memory created successfully"
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            })
        }
    }

    fn tool_search_memories(&self, arguments: &Value) -> Value {
        let query = arguments["query"].as_str().unwrap_or("");
        let memories = self.memory_manager.search_memories(query);
        json!({
            "success": true,
            "memories": memories.into_iter().map(|m| json!({
                "id": m.id,
                "content": m.content,
                "created_at": m.created_at,
                "updated_at": m.updated_at
            })).collect::<Vec<_>>()
        })
    }

    fn tool_update_memory(&mut self, arguments: &Value) -> Value {
        let id = arguments["id"].as_str().unwrap_or("");
        let content = arguments["content"].as_str().unwrap_or("");
        match self.memory_manager.update_memory(id, content) {
            Ok(()) => json!({
                "success": true,
                "message": "Memory updated successfully"
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            })
        }
    }

    fn tool_delete_memory(&mut self, arguments: &Value) -> Value {
        let id = arguments["id"].as_str().unwrap_or("");
        match self.memory_manager.delete_memory(id) {
            Ok(()) => json!({
                "success": true,
                "message": "Memory deleted successfully"
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            })
        }
    }

    fn tool_list_conversations(&self) -> Value {
        let conversations = self.memory_manager.list_conversations();
        json!({
            "success": true,
            "conversations": conversations.into_iter().map(|c| json!({
                "id": c.id,
                "title": c.title,
                "created_at": c.created_at,
                "message_count": c.message_count
            })).collect::<Vec<_>>()
        })
    }

    // 不明なメソッドハンドラ
    fn handle_unknown_method(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        })
    }
}