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
        let instructions = self.build_instructions();
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
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": instructions
            }
        })
    }

    fn build_instructions(&self) -> String {
        let mut parts = Vec::new();

        if let Ok(core) = reader::read_core() {
            if let Some(text) = core["value"]["content"]["text"].as_str() {
                if !text.is_empty() {
                    parts.push(text.to_string());
                }
            }
        }

        let records = reader::read_memory_all().unwrap_or_default();
        for record in &records {
            if let Some(text) = record["value"]["content"]["text"].as_str() {
                if !text.is_empty() {
                    parts.push(text.to_string());
                }
            }
        }

        parts.join("\n\n")
    }

    fn handle_tools_list(&self, id: Value) -> Value {
        let tools = vec![
            json!({
                "name": "read_core",
                "description": "Read the AI's identity and instructions (core record)",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            json!({
                "name": "read_memory",
                "description": "Read all memory records. Each record is a single memory element.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            json!({
                "name": "save_memory",
                "description": "Add a single memory element as a new record",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "A single memory element to save"
                        }
                    },
                    "required": ["content"]
                }
            }),
            json!({
                "name": "compress",
                "description": "Replace all memory records with a compressed set. Deletes all existing records and creates new ones from the provided items.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "items": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Array of memory elements to keep after compression"
                        }
                    },
                    "required": ["items"]
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
            Ok(record) => record,
            Err(e) => json!({ "error": e.to_string() }),
        }
    }

    fn tool_read_memory(&self) -> Value {
        match reader::read_memory_all() {
            Ok(records) => json!({ "records": records, "count": records.len() }),
            Err(e) => json!({ "error": e.to_string() }),
        }
    }

    fn tool_save_memory(&self, arguments: &Value) -> Value {
        let content = arguments["content"].as_str().unwrap_or("");
        match writer::save_memory(content) {
            Ok(()) => json!({ "success": true, "count": reader::memory_count() }),
            Err(e) => json!({ "error": e.to_string() }),
        }
    }

    fn tool_compress(&self, arguments: &Value) -> Value {
        let items: Vec<String> = arguments["items"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        match writer::compress_memory(&items) {
            Ok(()) => json!({ "success": true, "count": items.len() }),
            Err(e) => json!({ "error": e.to_string() }),
        }
    }
}
