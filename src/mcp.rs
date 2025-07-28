use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::memory::MemoryManager;

pub struct MCPServer {
    memory_manager: MemoryManager,
}

impl MCPServer {
    pub async fn new() -> Result<Self> {
        let memory_manager = MemoryManager::new().await?;
        Ok(MCPServer { memory_manager })
    }

    pub async fn run(&mut self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        
        // Set up line-based reading
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
                Err(_) => {
                    // EOF or error, exit gracefully
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn handle_request(&mut self, request: Value) -> Value {
        let method = request["method"].as_str().unwrap_or("");
        let id = request["id"].clone();

        match method {
            "initialize" => {
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
            "tools/list" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            {
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
                            },
                            {
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
                            },
                            {
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
                            },
                            {
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
                            },
                            {
                                "name": "list_conversations",
                                "description": "List all imported conversations",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                }
                            }
                        ]
                    }
                })
            }
            "tools/call" => {
                let tool_name = request["params"]["name"].as_str().unwrap_or("");
                let arguments = &request["params"]["arguments"];

                let result = match tool_name {
                    "create_memory" => {
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
                    "update_memory" => {
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
                    "delete_memory" => {
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
                    "search_memories" => {
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
                    "list_conversations" => {
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
                    _ => json!({
                        "success": false,
                        "error": format!("Unknown tool: {}", tool_name)
                    })
                };

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
            _ => {
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
    }
}