use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{
    extract::{Path as AxumPath, State},
    http::Method,
    response::Json,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Any};

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

// HTTP MCP Server state
pub type AppState = Arc<Mutex<MCPServer>>;

// MCP HTTP request types for REST-style endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPHttpRequest {
    pub user_id: Option<String>,
    pub message: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub query: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub command: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MCPHttpResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct MCPServer {
    config: Config,
    persona: Persona,
    transmission_controller: TransmissionController,
    scheduler: AIScheduler,
    service_client: ServiceClient,
    service_detector: ServiceDetector,
    user_id: String,
    data_dir: Option<std::path::PathBuf>,
}

impl MCPServer {
    pub fn new(config: Config, user_id: String, data_dir: Option<std::path::PathBuf>) -> Result<Self> {
        let persona = Persona::new(&config)?;
        let transmission_controller = TransmissionController::new(config.clone())?;
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
            user_id,
            data_dir,
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
            // HTTP endpoint直接呼び出し対応
            method => self.handle_direct_tool_call(method, request.params).await,
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

    async fn handle_direct_tool_call(&mut self, tool_name: &str, params: Value) -> Result<Value> {
        // HTTP endpointからの直接呼び出し用（パラメータ構造が異なる）
        match tool_name {
            "get_status" => self.tool_get_status(params).await,
            "chat_with_ai" => self.tool_chat_with_ai(params).await,
            "get_relationships" => self.tool_get_relationships(params).await,
            "get_memories" => self.tool_get_memories(params).await,
            "get_contextual_memories" => self.tool_get_contextual_memories(params).await,
            "search_memories" => self.tool_search_memories(params).await,
            "create_summary" => self.tool_create_summary(params).await,
            "create_core_memory" => self.tool_create_core_memory(params).await,
            "execute_command" => self.tool_execute_command(params).await,
            "analyze_file" => self.tool_analyze_file(params).await,
            "write_file" => self.tool_write_file(params).await,
            "list_files" => self.tool_list_files(params).await,
            "check_transmissions" => self.tool_check_transmissions(params).await,
            "run_maintenance" => self.tool_run_maintenance(params).await,
            "run_scheduler" => self.tool_run_scheduler(params).await,
            "get_scheduler_status" => self.tool_get_scheduler_status(params).await,
            "get_transmission_history" => self.tool_get_transmission_history(params).await,
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
        let upcoming_tasks: Vec<_> = self.scheduler.get_tasks()
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
        
        // Print available tools
        for tool in self.get_tools() {
            println!("  - {}: {}", ColorExt::cyan(tool.name.as_str()), tool.description);
        }
        
        println!("✅ MCP Server ready for requests");
        
        // Create shared state
        let app_state: AppState = Arc::new(Mutex::new(
            MCPServer::new(
                self.config.clone(),
                self.user_id.clone(),
                self.data_dir.clone(),
            )?
        ));
        
        // Create router with CORS
        let app = Router::new()
            // MCP Core endpoints
            .route("/mcp/tools", get(list_tools))
            .route("/mcp/call/:tool_name", post(call_tool))
            
            // AI Chat endpoints
            .route("/chat", post(chat_with_ai_handler))
            .route("/status", get(get_status_handler))
            .route("/status/:user_id", get(get_status_with_user_handler))
            
            // Memory endpoints
            .route("/memories/:user_id", get(get_memories_handler))
            .route("/memories/:user_id/search", post(search_memories_handler))
            .route("/memories/:user_id/contextual", post(get_contextual_memories_handler))
            .route("/memories/:user_id/summary", post(create_summary_handler))
            .route("/memories/:user_id/core", post(create_core_memory_handler))
            
            // Relationship endpoints
            .route("/relationships", get(get_relationships_handler))
            
            // System endpoints
            .route("/transmissions", get(check_transmissions_handler))
            .route("/maintenance", post(run_maintenance_handler))
            .route("/scheduler", post(run_scheduler_handler))
            .route("/scheduler/status", get(get_scheduler_status_handler))
            .route("/scheduler/history", get(get_transmission_history_handler))
            
            // File operations
            .route("/files", get(list_files_handler))
            .route("/files/analyze", post(analyze_file_handler))
            .route("/files/write", post(write_file_handler))
            
            // Shell execution
            .route("/execute", post(execute_command_handler))
            
            // AI Card and AI Log proxy endpoints
            .route("/card/user_cards/:user_id", get(get_user_cards_handler))
            .route("/card/draw", post(draw_card_handler))
            .route("/card/stats", get(get_card_stats_handler))
            .route("/log/posts", get(get_blog_posts_handler))
            .route("/log/posts", post(create_blog_post_handler))
            .route("/log/build", post(build_blog_handler))
            
            .layer(
                ServiceBuilder::new()
                    .layer(
                        CorsLayer::new()
                            .allow_origin(Any)
                            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                            .allow_headers(Any),
                    )
            )
            .with_state(app_state);
        
        // Start the server
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .context("Failed to bind to address")?;
            
        println!("🌐 HTTP MCP Server listening on http://0.0.0.0:{}", port);
        
        axum::serve(listener, app)
            .await
            .context("Server error")?;
            
        Ok(())
    }
    
    pub async fn run(&mut self, port: u16) -> Result<()> {
        self.start_server(port).await
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

// HTTP Handlers for MCP endpoints

// MCP Core handlers
async fn list_tools(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    let tools = server.get_tools();
    
    Json(MCPHttpResponse {
        success: true,
        result: Some(json!({
            "tools": tools
        })),
        error: None,
    })
}

async fn call_tool(
    State(state): State<AppState>,
    AxumPath(tool_name): AxumPath<String>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let mut server = state.lock().await;
    
    // Create MCP request from HTTP request
    let mcp_request = MCPRequest {
        method: tool_name,
        params: json!(request),
        id: Some(json!("http_request")),
    };
    
    let response = server.handle_request(mcp_request).await;
    
    Json(MCPHttpResponse {
        success: response.error.is_none(),
        result: response.result,
        error: response.error.map(|e| e.message),
    })
}

// AI Chat handlers
async fn chat_with_ai_handler(
    State(state): State<AppState>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let mut server = state.lock().await;
    
    let user_id = request.user_id.unwrap_or_else(|| "default_user".to_string());
    let message = request.message.unwrap_or_default();
    
    let args = json!({
        "user_id": user_id,
        "message": message,
        "provider": request.provider,
        "model": request.model
    });
    
    match server.tool_chat_with_ai(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn get_status_handler(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    match server.tool_get_status(json!({})).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn get_status_with_user_handler(
    State(state): State<AppState>,
    AxumPath(user_id): AxumPath<String>,
) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    let args = json!({
        "user_id": user_id
    });
    
    match server.tool_get_status(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

// Memory handlers
async fn get_memories_handler(
    State(state): State<AppState>,
    AxumPath(user_id): AxumPath<String>,
) -> Json<MCPHttpResponse> {
    let mut server = state.lock().await;
    
    let args = json!({
        "user_id": user_id,
        "limit": 10
    });
    
    match server.tool_get_memories(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn search_memories_handler(
    State(state): State<AppState>,
    AxumPath(user_id): AxumPath<String>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    let args = json!({
        "user_id": user_id,
        "query": request.query.unwrap_or_default(),
        "keywords": request.keywords.unwrap_or_default()
    });
    
    match server.tool_search_memories(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn get_contextual_memories_handler(
    State(state): State<AppState>,
    AxumPath(user_id): AxumPath<String>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    let args = json!({
        "user_id": user_id,
        "query": request.query.unwrap_or_default(),
        "limit": request.limit.unwrap_or(10)
    });
    
    match server.tool_get_contextual_memories(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn create_summary_handler(
    State(state): State<AppState>,
    AxumPath(user_id): AxumPath<String>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let mut server = state.lock().await;
    
    let args = json!({
        "user_id": user_id,
        "content": request.content.unwrap_or_default()
    });
    
    match server.tool_create_summary(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn create_core_memory_handler(
    State(state): State<AppState>,
    AxumPath(user_id): AxumPath<String>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let mut server = state.lock().await;
    
    let args = json!({
        "user_id": user_id,
        "content": request.content.unwrap_or_default()
    });
    
    match server.tool_create_core_memory(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

// Relationship handlers
async fn get_relationships_handler(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    match server.tool_get_relationships(json!({})).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

// System handlers
async fn check_transmissions_handler(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let mut server = state.lock().await;
    
    match server.tool_check_transmissions(json!({})).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn run_maintenance_handler(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let mut server = state.lock().await;
    
    match server.tool_run_maintenance(json!({})).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn run_scheduler_handler(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let mut server = state.lock().await;
    
    match server.tool_run_scheduler(json!({})).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn get_scheduler_status_handler(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    match server.tool_get_scheduler_status(json!({})).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn get_transmission_history_handler(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    let args = json!({
        "limit": 10
    });
    
    match server.tool_get_transmission_history(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

// File operation handlers
async fn list_files_handler(State(state): State<AppState>) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    let args = json!({
        "path": ".",
        "pattern": "*"
    });
    
    match server.tool_list_files(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn analyze_file_handler(
    State(state): State<AppState>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    let args = json!({
        "file_path": request.file_path.unwrap_or_default()
    });
    
    match server.tool_analyze_file(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn write_file_handler(
    State(state): State<AppState>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    let args = json!({
        "file_path": request.file_path.unwrap_or_default(),
        "content": request.content.unwrap_or_default()
    });
    
    match server.tool_write_file(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

// Shell execution handler
async fn execute_command_handler(
    State(state): State<AppState>,
    Json(request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    let server = state.lock().await;
    
    let args = json!({
        "command": request.command.unwrap_or_default()
    });
    
    match server.tool_execute_command(args).await {
        Ok(result) => Json(MCPHttpResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(MCPHttpResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

// AI Card proxy handlers (TODO: Fix ServiceClient method visibility)
async fn get_user_cards_handler(
    State(_state): State<AppState>,
    AxumPath(user_id): AxumPath<String>,
) -> Json<MCPHttpResponse> {
    // TODO: Implement proper ai.card service integration
    Json(MCPHttpResponse {
        success: false,
        result: None,
        error: Some(format!("AI Card service integration not yet implemented for user: {}", user_id)),
    })
}

async fn draw_card_handler(
    State(_state): State<AppState>,
    Json(_request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    // TODO: Implement proper ai.card service integration
    Json(MCPHttpResponse {
        success: false,
        result: None,
        error: Some("AI Card draw service integration not yet implemented".to_string()),
    })
}

async fn get_card_stats_handler(State(_state): State<AppState>) -> Json<MCPHttpResponse> {
    // TODO: Implement proper ai.card service integration
    Json(MCPHttpResponse {
        success: false,
        result: None,
        error: Some("AI Card stats service integration not yet implemented".to_string()),
    })
}

// AI Log proxy handlers (placeholder - these would need to be implemented)
async fn get_blog_posts_handler(State(_state): State<AppState>) -> Json<MCPHttpResponse> {
    // TODO: Implement ai.log service integration
    Json(MCPHttpResponse {
        success: false,
        result: None,
        error: Some("AI Log service integration not yet implemented".to_string()),
    })
}

async fn create_blog_post_handler(
    State(_state): State<AppState>,
    Json(_request): Json<MCPHttpRequest>,
) -> Json<MCPHttpResponse> {
    // TODO: Implement ai.log service integration
    Json(MCPHttpResponse {
        success: false,
        result: None,
        error: Some("AI Log service integration not yet implemented".to_string()),
    })
}

async fn build_blog_handler(State(_state): State<AppState>) -> Json<MCPHttpResponse> {
    // TODO: Implement ai.log service integration
    Json(MCPHttpResponse {
        success: false,
        result: None,
        error: Some("AI Log service integration not yet implemented".to_string()),
    })
}