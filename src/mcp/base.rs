use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::core::{Memory, MemoryStore, UserAnalysis};

pub struct BaseMCPServer {
    store: MemoryStore,
}

impl BaseMCPServer {
    pub fn new() -> Result<Self> {
        let store = MemoryStore::default()?;
        Ok(BaseMCPServer { store })
    }

    pub fn run(&self) -> Result<()> {
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
                        let response = self.handle_request(request);
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

    fn handle_request(&self, request: Value) -> Value {
        let method = request["method"].as_str().unwrap_or("");
        let id = request["id"].clone();

        match method {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(request, id),
            _ => self.handle_unknown_method(id),
        }
    }

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
                    "version": "0.2.0"
                }
            }
        })
    }

    fn handle_tools_list(&self, id: Value) -> Value {
        let tools = self.get_available_tools();
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        })
    }

    fn get_available_tools(&self) -> Vec<Value> {
        vec![
            json!({
                "name": "create_memory",
                "description": "Create a new memory entry (Layer 1: simple storage)",
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
                "name": "create_ai_memory",
                "description": "Create a memory with AI interpretation and priority score (Layer 2)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Original content of the memory"
                        },
                        "ai_interpretation": {
                            "type": "string",
                            "description": "AI's creative interpretation of the content (optional)"
                        },
                        "priority_score": {
                            "type": "number",
                            "description": "Priority score from 0.0 (low) to 1.0 (high) (optional)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        }
                    },
                    "required": ["content"]
                }
            }),
            json!({
                "name": "get_memory",
                "description": "Get a memory by ID",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Memory ID"
                        }
                    },
                    "required": ["id"]
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
                "name": "list_memories",
                "description": "List all memories",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
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
                "name": "save_user_analysis",
                "description": "Save a Big Five personality analysis based on user's memories (Layer 3)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "openness": {
                            "type": "number",
                            "description": "Openness to Experience (0.0-1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "conscientiousness": {
                            "type": "number",
                            "description": "Conscientiousness (0.0-1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "extraversion": {
                            "type": "number",
                            "description": "Extraversion (0.0-1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "agreeableness": {
                            "type": "number",
                            "description": "Agreeableness (0.0-1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "neuroticism": {
                            "type": "number",
                            "description": "Neuroticism (0.0-1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "summary": {
                            "type": "string",
                            "description": "AI-generated summary of the personality analysis"
                        }
                    },
                    "required": ["openness", "conscientiousness", "extraversion", "agreeableness", "neuroticism", "summary"]
                }
            }),
            json!({
                "name": "get_user_analysis",
                "description": "Get the most recent Big Five personality analysis (Layer 3)",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
        ]
    }

    fn handle_tools_call(&self, request: Value, id: Value) -> Value {
        let tool_name = request["params"]["name"].as_str().unwrap_or("");
        let arguments = &request["params"]["arguments"];

        let result = self.execute_tool(tool_name, arguments);

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

    fn execute_tool(&self, tool_name: &str, arguments: &Value) -> Value {
        match tool_name {
            "create_memory" => self.tool_create_memory(arguments),
            "create_ai_memory" => self.tool_create_ai_memory(arguments),
            "get_memory" => self.tool_get_memory(arguments),
            "search_memories" => self.tool_search_memories(arguments),
            "list_memories" => self.tool_list_memories(),
            "update_memory" => self.tool_update_memory(arguments),
            "delete_memory" => self.tool_delete_memory(arguments),
            "save_user_analysis" => self.tool_save_user_analysis(arguments),
            "get_user_analysis" => self.tool_get_user_analysis(),
            _ => json!({
                "success": false,
                "error": format!("Unknown tool: {}", tool_name)
            }),
        }
    }

    fn tool_create_memory(&self, arguments: &Value) -> Value {
        let content = arguments["content"].as_str().unwrap_or("");
        let memory = Memory::new(content.to_string());

        match self.store.create(&memory) {
            Ok(()) => json!({
                "success": true,
                "id": memory.id,
                "message": "Memory created successfully"
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

    fn tool_create_ai_memory(&self, arguments: &Value) -> Value {
        let content = arguments["content"].as_str().unwrap_or("");
        let ai_interpretation = arguments["ai_interpretation"]
            .as_str()
            .map(|s| s.to_string());
        let priority_score = arguments["priority_score"].as_f64().map(|f| f as f32);

        let memory = Memory::new_ai(content.to_string(), ai_interpretation, priority_score);

        match self.store.create(&memory) {
            Ok(()) => json!({
                "success": true,
                "id": memory.id,
                "message": "AI memory created successfully",
                "has_interpretation": memory.ai_interpretation.is_some(),
                "has_score": memory.priority_score.is_some()
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

    fn tool_get_memory(&self, arguments: &Value) -> Value {
        let id = arguments["id"].as_str().unwrap_or("");

        match self.store.get(id) {
            Ok(memory) => json!({
                "success": true,
                "memory": {
                    "id": memory.id,
                    "content": memory.content,
                    "ai_interpretation": memory.ai_interpretation,
                    "priority_score": memory.priority_score,
                    "created_at": memory.created_at,
                    "updated_at": memory.updated_at
                }
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

    fn tool_search_memories(&self, arguments: &Value) -> Value {
        let query = arguments["query"].as_str().unwrap_or("");

        match self.store.search(query) {
            Ok(memories) => json!({
                "success": true,
                "memories": memories.into_iter().map(|m| json!({
                    "id": m.id,
                    "content": m.content,
                    "ai_interpretation": m.ai_interpretation,
                    "priority_score": m.priority_score,
                    "created_at": m.created_at,
                    "updated_at": m.updated_at
                })).collect::<Vec<_>>()
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

    fn tool_list_memories(&self) -> Value {
        match self.store.list() {
            Ok(memories) => json!({
                "success": true,
                "memories": memories.into_iter().map(|m| json!({
                    "id": m.id,
                    "content": m.content,
                    "ai_interpretation": m.ai_interpretation,
                    "priority_score": m.priority_score,
                    "created_at": m.created_at,
                    "updated_at": m.updated_at
                })).collect::<Vec<_>>()
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

    fn tool_update_memory(&self, arguments: &Value) -> Value {
        let id = arguments["id"].as_str().unwrap_or("");
        let content = arguments["content"].as_str().unwrap_or("");

        match self.store.get(id) {
            Ok(mut memory) => {
                memory.update_content(content.to_string());
                match self.store.update(&memory) {
                    Ok(()) => json!({
                        "success": true,
                        "message": "Memory updated successfully"
                    }),
                    Err(e) => json!({
                        "success": false,
                        "error": e.to_string()
                    }),
                }
            }
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

    fn tool_delete_memory(&self, arguments: &Value) -> Value {
        let id = arguments["id"].as_str().unwrap_or("");

        match self.store.delete(id) {
            Ok(()) => json!({
                "success": true,
                "message": "Memory deleted successfully"
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

    // ========== Layer 3: User Analysis Tools ==========

    fn tool_save_user_analysis(&self, arguments: &Value) -> Value {
        let openness = arguments["openness"].as_f64().unwrap_or(0.5) as f32;
        let conscientiousness = arguments["conscientiousness"].as_f64().unwrap_or(0.5) as f32;
        let extraversion = arguments["extraversion"].as_f64().unwrap_or(0.5) as f32;
        let agreeableness = arguments["agreeableness"].as_f64().unwrap_or(0.5) as f32;
        let neuroticism = arguments["neuroticism"].as_f64().unwrap_or(0.5) as f32;
        let summary = arguments["summary"].as_str().unwrap_or("").to_string();

        let analysis = UserAnalysis::new(
            openness,
            conscientiousness,
            extraversion,
            agreeableness,
            neuroticism,
            summary,
        );

        match self.store.save_analysis(&analysis) {
            Ok(()) => json!({
                "success": true,
                "id": analysis.id,
                "message": "User analysis saved successfully",
                "dominant_trait": analysis.dominant_trait()
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

    fn tool_get_user_analysis(&self) -> Value {
        match self.store.get_latest_analysis() {
            Ok(Some(analysis)) => json!({
                "success": true,
                "analysis": {
                    "id": analysis.id,
                    "openness": analysis.openness,
                    "conscientiousness": analysis.conscientiousness,
                    "extraversion": analysis.extraversion,
                    "agreeableness": analysis.agreeableness,
                    "neuroticism": analysis.neuroticism,
                    "summary": analysis.summary,
                    "dominant_trait": analysis.dominant_trait(),
                    "analyzed_at": analysis.analyzed_at
                }
            }),
            Ok(None) => json!({
                "success": true,
                "analysis": null,
                "message": "No analysis found. Run personality analysis first."
            }),
            Err(e) => json!({
                "success": false,
                "error": e.to_string()
            }),
        }
    }

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
