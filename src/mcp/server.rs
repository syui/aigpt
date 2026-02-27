use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::core::{reader, writer};

pub struct MCPServer;

impl MCPServer {
    pub fn new() -> Self {
        MCPServer
    }

    pub fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        let reader = stdin.lock();
        let lines = reader.lines();

        for line_result in lines {
            match line_result {
                Ok(line) => {
                    let trimmed = line.trim().to_string();
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
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                }
            }),
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
                    "version": "0.3.0"
                }
            }
        })
    }

    fn handle_tools_list(&self, id: Value) -> Value {
        let tools = vec![
            json!({
                "name": "read_core",
                "description": "Read core.md - the AI's identity and instructions",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            json!({
                "name": "read_memory",
                "description": "Read memory.md - the AI's accumulated memories",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            json!({
                "name": "save_memory",
                "description": "Overwrite memory.md with new content",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Content to write to memory.md"
                        }
                    },
                    "required": ["content"]
                }
            }),
            json!({
                "name": "compress",
                "description": "Compress conversation into memory. AI decides what to keep, tool writes the result to memory.md",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "conversation": {
                            "type": "string",
                            "description": "Compressed memory content to save"
                        }
                    },
                    "required": ["conversation"]
                }
            }),
        ];

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        })
    }

    fn handle_tools_call(&self, request: Value, id: Value) -> Value {
        let tool_name = request["params"]["name"].as_str().unwrap_or("");
        let arguments = &request["params"]["arguments"];

        let result = match tool_name {
            "read_core" => self.tool_read_core(),
            "read_memory" => self.tool_read_memory(),
            "save_memory" => self.tool_save_memory(arguments),
            "compress" => self.tool_compress(arguments),
            _ => json!({
                "error": format!("Unknown tool: {}", tool_name)
            }),
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

    fn tool_read_core(&self) -> Value {
        match reader::read_core() {
            Ok(content) => json!({ "content": content }),
            Err(e) => json!({ "error": e.to_string() }),
        }
    }

    fn tool_read_memory(&self) -> Value {
        match reader::read_memory() {
            Ok(content) => json!({ "content": content }),
            Err(e) => json!({ "error": e.to_string() }),
        }
    }

    fn tool_save_memory(&self, arguments: &Value) -> Value {
        let content = arguments["content"].as_str().unwrap_or("");
        match writer::save_memory(content) {
            Ok(()) => json!({ "success": true }),
            Err(e) => json!({ "error": e.to_string() }),
        }
    }

    fn tool_compress(&self, arguments: &Value) -> Value {
        let conversation = arguments["conversation"].as_str().unwrap_or("");
        match writer::save_memory(conversation) {
            Ok(()) => json!({ "success": true }),
            Err(e) => json!({ "error": e.to_string() }),
        }
    }
}
