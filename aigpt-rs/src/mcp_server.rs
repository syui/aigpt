use serde::{Deserialize, Serialize};
use anyhow::Result;
use serde_json::Value;
use std::path::Path;

use crate::config::Config;
use crate::persona::Persona;
use crate::transmission::TransmissionController;
use crate::scheduler::AIScheduler;
use crate::http_client::{ServiceClient, ServiceDetector};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub method: String,
    pub params: Value,
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub result: Option<Value>,
    pub error: Option<MCPError>,
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

pub struct MCPServer {
    config: Config,
    persona: Persona,
    transmission_controller: TransmissionController,
    scheduler: AIScheduler,
    service_client: ServiceClient,
    service_detector: ServiceDetector,
}

impl MCPServer {
    pub fn new(config: Config) -> Result<Self> {
        let persona = Persona::new(&config)?;
        let transmission_controller = TransmissionController::new(&config)?;
        let scheduler = AIScheduler::new(&config)?;
        let service_client = ServiceClient::new();
        let service_detector = ServiceDetector::new();
        
        Ok(MCPServer {
            config,
            persona,
            transmission_controller,
            scheduler,
            service_client,
            service_detector,
        })
    }
    
    pub fn get_tools(&self) -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "get_status".to_string(),
                description: "Get AI status including mood, fortune, and personality".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User ID to get relationship status for (optional)"
                        }
                    }
                }),
            },
            MCPTool {
                name: "chat_with_ai".to_string(),
                description: "Send a message to the AI and get a response".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User ID for the conversation"
                        },
                        "message": {
                            "type": "string",
                            "description": "Message to send to the AI"
                        },
                        "provider": {
                            "type": "string",
                            "description": "AI provider to use (ollama/openai) - optional"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use - optional"
                        }
                    },
                    "required": ["user_id", "message"]
                }),
            },
            MCPTool {
                name: "get_relationships".to_string(),
                description: "Get all relationships and their statuses".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "get_memories".to_string(),
                description: "Get memories for a specific user".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User ID to get memories for"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of memories to return (default: 10)"
                        }
                    },
                    "required": ["user_id"]
                }),
            },
            MCPTool {
                name: "get_contextual_memories".to_string(),
                description: "Get memories organized by priority with contextual relevance".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User ID to get memories for"
                        },
                        "query": {
                            "type": "string",
                            "description": "Query text for contextual relevance matching"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of memories to return (default: 10)"
                        }
                    },
                    "required": ["user_id", "query"]
                }),
            },
            MCPTool {
                name: "search_memories".to_string(),
                description: "Search memories by keywords and types".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User ID to search memories for"
                        },
                        "keywords": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Keywords to search for in memory content"
                        },
                        "memory_types": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Memory types to filter by (conversation, core, summary, experience)"
                        }
                    },
                    "required": ["user_id", "keywords"]
                }),
            },
            MCPTool {
                name: "create_summary".to_string(),
                description: "Create AI-powered summary of recent memories".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User ID to create summary for"
                        }
                    },
                    "required": ["user_id"]
                }),
            },
            MCPTool {
                name: "create_core_memory".to_string(),
                description: "Create core memory by analyzing existing memories".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User ID to create core memory for"
                        }
                    },
                    "required": ["user_id"]
                }),
            },
            MCPTool {
                name: "execute_command".to_string(),
                description: "Execute shell commands with safety checks".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "Shell command to execute"
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "Working directory for command execution (optional)"
                        },
                        "timeout": {
                            "type": "integer",
                            "description": "Timeout in seconds (default: 30)"
                        }
                    },
                    "required": ["command"]
                }),
            },
            MCPTool {
                name: "analyze_file".to_string(),
                description: "Analyze files using AI provider".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to file to analyze"
                        },
                        "analysis_type": {
                            "type": "string",
                            "description": "Type of analysis (code, text, structure, security)",
                            "default": "general"
                        }
                    },
                    "required": ["file_path"]
                }),
            },
            MCPTool {
                name: "write_file".to_string(),
                description: "Write content to files with backup functionality".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to file to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write to file"
                        },
                        "create_backup": {
                            "type": "boolean",
                            "description": "Create backup before writing (default: true)"
                        }
                    },
                    "required": ["file_path", "content"]
                }),
            },
            MCPTool {
                name: "list_files".to_string(),
                description: "List files in directory with pattern matching".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "directory": {
                            "type": "string",
                            "description": "Directory to list files from"
                        },
                        "pattern": {
                            "type": "string",
                            "description": "Glob pattern for file filtering (optional)"
                        },
                        "recursive": {
                            "type": "boolean",
                            "description": "Recursive directory traversal (default: false)"
                        }
                    },
                    "required": ["directory"]
                }),
            },
            MCPTool {
                name: "check_transmissions".to_string(),
                description: "Check and execute autonomous transmissions".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "run_maintenance".to_string(),
                description: "Run daily maintenance tasks".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "run_scheduler".to_string(),
                description: "Run scheduled tasks".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "get_scheduler_status".to_string(),
                description: "Get scheduler statistics and upcoming tasks".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "get_transmission_history".to_string(),
                description: "Get recent transmission history".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of transmissions to return (default: 10)"
                        }
                    }
                }),
            },
        ]
    }
    
    pub async fn handle_request(&mut self, request: MCPRequest) -> MCPResponse {
        let result = match request.method.as_str() {
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_tool_call(request.params).await,
            _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
        };
        
        match result {
            Ok(value) => MCPResponse {
                result: Some(value),
                error: None,
                id: request.id,
            },
            Err(e) => MCPResponse {
                result: None,
                error: Some(MCPError {
                    code: -32603,
                    message: e.to_string(),
                    data: None,
                }),
                id: request.id,
            },
        }
    }
    
    async fn handle_list_tools(&self) -> Result<Value> {
        let tools = self.get_tools();
        Ok(serde_json::json!({
            "tools": tools
        }))
    }
    
    async fn handle_tool_call(&mut self, params: Value) -> Result<Value> {
        let tool_name = params["name"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
        let arguments = params["arguments"].clone();
        
        match tool_name {
            "get_status" => self.tool_get_status(arguments).await,
            "chat_with_ai" => self.tool_chat_with_ai(arguments).await,
            "get_relationships" => self.tool_get_relationships(arguments).await,
            "get_contextual_memories" => self.tool_get_contextual_memories(arguments).await,
            "search_memories" => self.tool_search_memories(arguments).await,
            "create_summary" => self.tool_create_summary(arguments).await,
            "create_core_memory" => self.tool_create_core_memory(arguments).await,
            "execute_command" => self.tool_execute_command(arguments).await,
            "analyze_file" => self.tool_analyze_file(arguments).await,
            "write_file" => self.tool_write_file(arguments).await,
            "list_files" => self.tool_list_files(arguments).await,
            "get_memories" => self.tool_get_memories(arguments).await,
            "check_transmissions" => self.tool_check_transmissions(arguments).await,
            "run_maintenance" => self.tool_run_maintenance(arguments).await,
            "run_scheduler" => self.tool_run_scheduler(arguments).await,
            "get_scheduler_status" => self.tool_get_scheduler_status(arguments).await,
            "get_transmission_history" => self.tool_get_transmission_history(arguments).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }
    
    async fn tool_get_status(&self, args: Value) -> Result<Value> {
        let user_id = args["user_id"].as_str();
        let state = self.persona.get_current_state()?;
        
        let mut result = serde_json::json!({
            "mood": state.current_mood,
            "fortune": state.fortune_value,
            "breakthrough_triggered": state.breakthrough_triggered,
            "personality": state.base_personality
        });
        
        if let Some(user_id) = user_id {
            if let Some(relationship) = self.persona.get_relationship(user_id) {
                result["relationship"] = serde_json::json!({
                    "status": relationship.status.to_string(),
                    "score": relationship.score,
                    "threshold": relationship.threshold,
                    "transmission_enabled": relationship.transmission_enabled,
                    "total_interactions": relationship.total_interactions,
                    "is_broken": relationship.is_broken
                });
            }
        }
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result)?
                }
            ]
        }))
    }
    
    async fn tool_chat_with_ai(&mut self, args: Value) -> Result<Value> {
        let user_id = args["user_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;
        let message = args["message"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing message"))?;
        let provider = args["provider"].as_str().map(|s| s.to_string());
        let model = args["model"].as_str().map(|s| s.to_string());
        
        let (response, relationship_delta) = if provider.is_some() || model.is_some() {
            self.persona.process_ai_interaction(user_id, message, provider, model).await?
        } else {
            self.persona.process_interaction(user_id, message)?
        };
        
        let relationship_status = self.persona.get_relationship(user_id)
            .map(|r| serde_json::json!({
                "status": r.status.to_string(),
                "score": r.score,
                "transmission_enabled": r.transmission_enabled
            }));
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("AI Response: {}\n\nRelationship Change: {:+.2}\n\nRelationship Status: {}", 
                             response, 
                             relationship_delta,
                             relationship_status.map(|r| r.to_string()).unwrap_or_else(|| "None".to_string()))
                }
            ]
        }))
    }
    
    async fn tool_get_relationships(&self, _args: Value) -> Result<Value> {
        let relationships = self.persona.list_all_relationships();
        
        let relationships_json: Vec<Value> = relationships.iter()
            .map(|(user_id, rel)| {
                serde_json::json!({
                    "user_id": user_id,
                    "status": rel.status.to_string(),
                    "score": rel.score,
                    "threshold": rel.threshold,
                    "transmission_enabled": rel.transmission_enabled,
                    "total_interactions": rel.total_interactions,
                    "is_broken": rel.is_broken,
                    "last_interaction": rel.last_interaction
                })
            })
            .collect();
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&relationships_json)?
                }
            ]
        }))
    }
    
    async fn tool_get_memories(&mut self, args: Value) -> Result<Value> {
        let user_id = args["user_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;
        let limit = args["limit"].as_u64().unwrap_or(10) as usize;
        
        let memories = self.persona.get_memories(user_id, limit);
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&memories)?
                }
            ]
        }))
    }
    
    async fn tool_check_transmissions(&mut self, _args: Value) -> Result<Value> {
        let autonomous = self.transmission_controller.check_autonomous_transmissions(&mut self.persona).await?;
        let breakthrough = self.transmission_controller.check_breakthrough_transmissions(&mut self.persona).await?;
        let maintenance = self.transmission_controller.check_maintenance_transmissions(&mut self.persona).await?;
        
        let total = autonomous.len() + breakthrough.len() + maintenance.len();
        
        let result = serde_json::json!({
            "autonomous_transmissions": autonomous.len(),
            "breakthrough_transmissions": breakthrough.len(),
            "maintenance_transmissions": maintenance.len(),
            "total_transmissions": total,
            "transmissions": {
                "autonomous": autonomous,
                "breakthrough": breakthrough,
                "maintenance": maintenance
            }
        });
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result)?
                }
            ]
        }))
    }
    
    async fn tool_get_contextual_memories(&self, args: Value) -> Result<Value> {
        let user_id = args["user_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;
        let query = args["query"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query"))?;
        let limit = args["limit"].as_u64().unwrap_or(10) as usize;
        
        // For now, use search_memories as a placeholder for contextual memories
        let keywords = query.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>();
        let memory_results = self.persona.search_memories(user_id, &keywords);
        let memories = memory_results.into_iter().take(limit).collect::<Vec<_>>();
        
        let memories_json: Vec<Value> = memories.iter()
            .enumerate()
            .map(|(id, content)| {
                serde_json::json!({
                    "id": id,
                    "content": content,
                    "level": "conversation",
                    "importance": 0.5,
                    "is_core": false,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "summary": None::<String>,
                    "metadata": {}
                })
            })
            .collect();
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("Found {} contextual memories for query: '{}'\n\n{}", 
                             memories.len(), 
                             query,
                             serde_json::to_string_pretty(&memories_json)?)
                }
            ]
        }))
    }
    
    async fn tool_search_memories(&self, args: Value) -> Result<Value> {
        let user_id = args["user_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;
        let keywords: Vec<String> = args["keywords"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Missing keywords"))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        
        if keywords.is_empty() {
            return Err(anyhow::anyhow!("At least one keyword is required"));
        }
        
        let memories = self.persona.search_memories(user_id, &keywords);
        
        let memories_json: Vec<Value> = memories.iter()
            .enumerate()
            .map(|(id, content)| {
                serde_json::json!({
                    "id": id,
                    "content": content,
                    "level": "conversation",
                    "importance": 0.5,
                    "is_core": false,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "summary": None::<String>,
                    "metadata": {}
                })
            })
            .collect();
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("Found {} memories matching keywords: {}\n\n{}", 
                             memories.len(),
                             keywords.join(", "),
                             serde_json::to_string_pretty(&memories_json)?)
                }
            ]
        }))
    }
    
    async fn tool_create_summary(&mut self, args: Value) -> Result<Value> {
        let user_id = args["user_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;
        
        // Placeholder implementation - in full version this would use AI
        let result = format!("Summary created for user: {}", user_id);
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": result
                }
            ]
        }))
    }
    
    async fn tool_create_core_memory(&mut self, args: Value) -> Result<Value> {
        let user_id = args["user_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;
        
        // Placeholder implementation - in full version this would use AI
        let result = format!("Core memory created for user: {}", user_id);
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": result
                }
            ]
        }))
    }
    
    async fn tool_execute_command(&self, args: Value) -> Result<Value> {
        let command = args["command"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing command"))?;
        let working_dir = args["working_dir"].as_str();
        let _timeout = args["timeout"].as_u64().unwrap_or(30);

        // Security check - block dangerous commands
        if self.is_dangerous_command(command) {
            return Ok(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("Command blocked for security reasons: {}", command)
                    }
                ]
            }));
        }

        use std::process::Command;

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command]);
            c
        };

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let status = output.status;

                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Command: {}\nStatus: {}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}", 
                                           command, status, stdout, stderr)
                        }
                    ]
                }))
            }
            Err(e) => {
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Failed to execute command '{}': {}", command, e)
                        }
                    ]
                }))
            }
        }
    }

    fn is_dangerous_command(&self, command: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf", "rmdir", "del /q", "format", "fdisk",
            "dd if=", "mkfs", "shutdown", "reboot", "halt",
            "sudo rm", "sudo dd", "chmod 777", "chown root",
            "> /dev/", "curl", "wget", "nc ", "netcat",
        ];

        dangerous_patterns.iter().any(|pattern| command.contains(pattern))
    }

    async fn tool_analyze_file(&self, args: Value) -> Result<Value> {
        let file_path = args["file_path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
        let analysis_type = args["analysis_type"].as_str().unwrap_or("general");

        use std::fs;
        use std::path::Path;

        let path = Path::new(file_path);
        if !path.exists() {
            return Ok(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("File not found: {}", file_path)
                    }
                ]
            }));
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                let file_size = content.len();
                let line_count = content.lines().count();
                let analysis = match analysis_type {
                    "code" => self.analyze_code_content(&content),
                    "structure" => self.analyze_file_structure(&content),
                    "security" => self.analyze_security(&content),
                    _ => self.analyze_general_content(&content),
                };

                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("File Analysis: {}\nType: {}\nSize: {} bytes\nLines: {}\n\n{}", 
                                           file_path, analysis_type, file_size, line_count, analysis)
                        }
                    ]
                }))
            }
            Err(e) => {
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Failed to read file '{}': {}", file_path, e)
                        }
                    ]
                }))
            }
        }
    }

    fn analyze_code_content(&self, content: &str) -> String {
        let mut analysis = String::new();
        
        // Basic code analysis
        if content.contains("fn ") || content.contains("function ") {
            analysis.push_str("Language: Likely Rust or JavaScript\n");
        }
        if content.contains("def ") {
            analysis.push_str("Language: Likely Python\n");
        }
        if content.contains("class ") {
            analysis.push_str("Contains class definitions\n");
        }
        if content.contains("import ") || content.contains("use ") {
            analysis.push_str("Contains import/use statements\n");
        }
        
        analysis.push_str(&format!("Functions/methods found: {}\n", 
            content.matches("fn ").count() + content.matches("def ").count() + content.matches("function ").count()));
        
        analysis
    }

    fn analyze_file_structure(&self, content: &str) -> String {
        format!("Structure analysis:\n- Characters: {}\n- Words: {}\n- Lines: {}\n- Paragraphs: {}",
            content.len(),
            content.split_whitespace().count(),
            content.lines().count(),
            content.split("\n\n").count())
    }

    fn analyze_security(&self, content: &str) -> String {
        let mut issues = Vec::new();
        
        if content.contains("password") || content.contains("secret") {
            issues.push("Contains potential password/secret references");
        }
        if content.contains("api_key") || content.contains("token") {
            issues.push("Contains potential API keys or tokens");
        }
        if content.contains("eval(") || content.contains("exec(") {
            issues.push("Contains potentially dangerous eval/exec calls");
        }
        
        if issues.is_empty() {
            "No obvious security issues found".to_string()
        } else {
            format!("Security concerns:\n- {}", issues.join("\n- "))
        }
    }

    fn analyze_general_content(&self, content: &str) -> String {
        format!("General analysis:\n- File appears to be text-based\n- Contains {} characters\n- {} lines total",
            content.len(), content.lines().count())
    }

    async fn tool_write_file(&self, args: Value) -> Result<Value> {
        let file_path = args["file_path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
        let content = args["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        let create_backup = args["create_backup"].as_bool().unwrap_or(true);

        use std::fs;
        use std::path::Path;

        let path = Path::new(file_path);
        
        // Create backup if file exists and backup is requested
        if create_backup && path.exists() {
            let backup_path = format!("{}.backup", file_path);
            if let Err(e) = fs::copy(file_path, &backup_path) {
                return Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Failed to create backup: {}", e)
                        }
                    ]
                }));
            }
        }

        match fs::write(path, content) {
            Ok(()) => {
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Successfully wrote {} bytes to: {}", content.len(), file_path)
                        }
                    ]
                }))
            }
            Err(e) => {
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Failed to write file '{}': {}", file_path, e)
                        }
                    ]
                }))
            }
        }
    }

    async fn tool_list_files(&self, args: Value) -> Result<Value> {
        let directory = args["directory"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing directory"))?;
        let pattern = args["pattern"].as_str();
        let recursive = args["recursive"].as_bool().unwrap_or(false);

        use std::fs;
        use std::path::Path;

        let dir_path = Path::new(directory);
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("Directory not found or not a directory: {}", directory)
                    }
                ]
            }));
        }

        let mut files = Vec::new();
        if recursive {
            self.collect_files_recursive(dir_path, pattern, &mut files);
        } else {
            if let Ok(entries) = fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    
                    if let Some(pat) = pattern {
                        if !file_name.contains(pat) {
                            continue;
                        }
                    }
                    
                    files.push(format!("{} ({})", 
                        path.display(), 
                        if path.is_dir() { "directory" } else { "file" }));
                }
            }
        }

        files.sort();
        let result = if files.is_empty() {
            "No files found matching criteria".to_string()
        } else {
            format!("Found {} items:\n{}", files.len(), files.join("\n"))
        };

        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": result
                }
            ]
        }))
    }

    fn collect_files_recursive(&self, dir: &Path, pattern: Option<&str>, files: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                
                if path.is_dir() {
                    self.collect_files_recursive(&path, pattern, files);
                } else {
                    if let Some(pat) = pattern {
                        if !file_name.contains(pat) {
                            continue;
                        }
                    }
                    files.push(path.display().to_string());
                }
            }
        }
    }
    
    async fn tool_run_maintenance(&mut self, _args: Value) -> Result<Value> {
        self.persona.daily_maintenance()?;
        let maintenance_transmissions = self.transmission_controller.check_maintenance_transmissions(&mut self.persona).await?;
        
        let stats = self.persona.get_relationship_stats();
        
        let result = serde_json::json!({
            "maintenance_completed": true,
            "maintenance_transmissions_sent": maintenance_transmissions.len(),
            "relationship_stats": stats
        });
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result)?
                }
            ]
        }))
    }
    
    async fn tool_run_scheduler(&mut self, _args: Value) -> Result<Value> {
        let executions = self.scheduler.run_scheduled_tasks(&mut self.persona, &mut self.transmission_controller).await?;
        let stats = self.scheduler.get_scheduler_stats();
        
        let result = serde_json::json!({
            "tasks_executed": executions.len(),
            "executions": executions,
            "scheduler_stats": {
                "total_tasks": stats.total_tasks,
                "enabled_tasks": stats.enabled_tasks,
                "due_tasks": stats.due_tasks,
                "total_executions": stats.total_executions,
                "today_executions": stats.today_executions,
                "success_rate": stats.success_rate,
                "avg_duration_ms": stats.avg_duration_ms
            }
        });
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result)?
                }
            ]
        }))
    }
    
    async fn tool_get_scheduler_status(&self, _args: Value) -> Result<Value> {
        let stats = self.scheduler.get_scheduler_stats();
        let upcoming_tasks: Vec<_> = self.scheduler.list_tasks()
            .values()
            .filter(|task| task.enabled)
            .take(10)
            .map(|task| {
                serde_json::json!({
                    "id": task.id,
                    "task_type": task.task_type.to_string(),
                    "next_run": task.next_run,
                    "interval_hours": task.interval_hours,
                    "run_count": task.run_count
                })
            })
            .collect();
        
        let recent_executions = self.scheduler.get_execution_history(Some(5));
        
        let result = serde_json::json!({
            "scheduler_stats": {
                "total_tasks": stats.total_tasks,
                "enabled_tasks": stats.enabled_tasks,
                "due_tasks": stats.due_tasks,
                "total_executions": stats.total_executions,
                "today_executions": stats.today_executions,
                "success_rate": stats.success_rate,
                "avg_duration_ms": stats.avg_duration_ms
            },
            "upcoming_tasks": upcoming_tasks,
            "recent_executions": recent_executions
        });
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result)?
                }
            ]
        }))
    }
    
    async fn tool_get_transmission_history(&self, args: Value) -> Result<Value> {
        let limit = args["limit"].as_u64().unwrap_or(10) as usize;
        let transmissions = self.transmission_controller.get_recent_transmissions(limit);
        
        Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&transmissions)?
                }
            ]
        }))
    }
    
    pub async fn start_server(&mut self, port: u16) -> Result<()> {
        println!("🚀 Starting MCP Server on port {}", port);
        println!("📋 Available tools: {}", self.get_tools().len());
        
        // In a real implementation, this would start an HTTP/WebSocket server
        // For now, we'll just print the available tools
        for tool in self.get_tools() {
            println!("  - {}: {}", tool.name.cyan(), tool.description);
        }
        
        println!("✅ MCP Server ready for requests");
        
        // Placeholder for actual server implementation
        Ok(())
    }
}

// Helper trait for colored output (placeholder)
trait ColorExt {
    fn cyan(&self) -> String;
}

impl ColorExt for str {
    fn cyan(&self) -> String {
        self.to_string() // In real implementation, would add ANSI color codes
    }
}