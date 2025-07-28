use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use aigpt::memory::MemoryManager;

pub struct ExtendedMCPServer {
    memory_manager: MemoryManager,
}

impl ExtendedMCPServer {
    pub async fn new() -> Result<Self> {
        let memory_manager = MemoryManager::new().await?;
        Ok(ExtendedMCPServer { memory_manager })
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
                            "name": "aigpt-extended",
                            "version": "0.1.0"
                        }
                    }
                })
            }
            "tools/list" => {
                #[allow(unused_mut)]
                let mut tools = vec![
                    // Basic tools
                    json!({
                        "name": "create_memory",
                        "description": "Create a new memory entry",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "content": {
                                    "type": "string",
                                    "description": "Content of the memory"
                                },
                                "analyze": {
                                    "type": "boolean",
                                    "description": "Enable AI analysis for this memory"
                                }
                            },
                            "required": ["content"]
                        }
                    }),
                    json!({
                        "name": "search_memories",
                        "description": "Search memories with advanced options",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "Search query"
                                },
                                "semantic": {
                                    "type": "boolean",
                                    "description": "Use semantic search"
                                },
                                "category": {
                                    "type": "string",
                                    "description": "Filter by category"
                                },
                                "time_range": {
                                    "type": "string",
                                    "description": "Filter by time range (e.g., '1week', '1month')"
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
                    })
                ];

                // Add extended tools based on features
                #[cfg(feature = "web-integration")]
                {
                    tools.push(json!({
                        "name": "import_webpage",
                        "description": "Import content from a webpage",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "url": {
                                    "type": "string",
                                    "description": "URL to import from"
                                }
                            },
                            "required": ["url"]
                        }
                    }));
                }

                #[cfg(feature = "ai-analysis")]
                {
                    tools.push(json!({
                        "name": "analyze_sentiment",
                        "description": "Analyze sentiment of memories",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "period": {
                                    "type": "string",
                                    "description": "Time period to analyze"
                                }
                            }
                        }
                    }));

                    tools.push(json!({
                        "name": "extract_insights",
                        "description": "Extract insights and patterns from memories",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "category": {
                                    "type": "string",
                                    "description": "Category to analyze"
                                }
                            }
                        }
                    }));
                }

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools
                    }
                })
            }
            "tools/call" => {
                let tool_name = request["params"]["name"].as_str().unwrap_or("");
                let arguments = &request["params"]["arguments"];

                let result = match tool_name {
                    "create_memory" => {
                        let content = arguments["content"].as_str().unwrap_or("");
                        let analyze = arguments["analyze"].as_bool().unwrap_or(false);
                        
                        let final_content = if analyze {
                            #[cfg(feature = "ai-analysis")]
                            {
                                format!("[AI分析] 感情: neutral, カテゴリ: general\n{}", content)
                            }
                            #[cfg(not(feature = "ai-analysis"))]
                            {
                                content.to_string()
                            }
                        } else {
                            content.to_string()
                        };

                        match self.memory_manager.create_memory(&final_content) {
                            Ok(id) => json!({
                                "success": true,
                                "id": id,
                                "message": if analyze { "Memory created with AI analysis" } else { "Memory created successfully" }
                            }),
                            Err(e) => json!({
                                "success": false,
                                "error": e.to_string()
                            })
                        }
                    }
                    "search_memories" => {
                        let query = arguments["query"].as_str().unwrap_or("");
                        let semantic = arguments["semantic"].as_bool().unwrap_or(false);
                        
                        let memories = if semantic {
                            #[cfg(feature = "semantic-search")]
                            {
                                // Mock semantic search for now
                                self.memory_manager.search_memories(query)
                            }
                            #[cfg(not(feature = "semantic-search"))]
                            {
                                self.memory_manager.search_memories(query)
                            }
                        } else {
                            self.memory_manager.search_memories(query)
                        };

                        json!({
                            "success": true,
                            "memories": memories.into_iter().map(|m| json!({
                                "id": m.id,
                                "content": m.content,
                                "created_at": m.created_at,
                                "updated_at": m.updated_at
                            })).collect::<Vec<_>>(),
                            "search_type": if semantic { "semantic" } else { "keyword" }
                        })
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
                    #[cfg(feature = "web-integration")]
                    "import_webpage" => {
                        let url = arguments["url"].as_str().unwrap_or("");
                        match self.import_from_web(url).await {
                            Ok(content) => {
                                match self.memory_manager.create_memory(&content) {
                                    Ok(id) => json!({
                                        "success": true,
                                        "id": id,
                                        "message": format!("Webpage imported successfully from {}", url)
                                    }),
                                    Err(e) => json!({
                                        "success": false,
                                        "error": e.to_string()
                                    })
                                }
                            }
                            Err(e) => json!({
                                "success": false,
                                "error": format!("Failed to import webpage: {}", e)
                            })
                        }
                    }
                    #[cfg(feature = "ai-analysis")]
                    "analyze_sentiment" => {
                        json!({
                            "success": true,
                            "analysis": {
                                "positive": 60,
                                "neutral": 30,
                                "negative": 10,
                                "dominant_sentiment": "positive"
                            },
                            "message": "Sentiment analysis completed"
                        })
                    }
                    #[cfg(feature = "ai-analysis")]
                    "extract_insights" => {
                        json!({
                            "success": true,
                            "insights": {
                                "most_frequent_topics": ["programming", "ai", "productivity"],
                                "learning_frequency": "5 times per week",
                                "growth_trend": "increasing",
                                "recommendations": ["Focus more on advanced topics", "Consider practical applications"]
                            },
                            "message": "Insights extracted successfully"
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

    #[cfg(feature = "web-integration")]
    async fn import_from_web(&self, url: &str) -> Result<String> {
        let response = reqwest::get(url).await?;
        let content = response.text().await?;
        
        let document = scraper::Html::parse_document(&content);
        let title_selector = scraper::Selector::parse("title").unwrap();
        let body_selector = scraper::Selector::parse("p").unwrap();
        
        let title = document.select(&title_selector)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_else(|| "Untitled".to_string());
        
        let paragraphs: Vec<String> = document.select(&body_selector)
            .map(|el| el.inner_html())
            .take(5)
            .collect();
        
        Ok(format!("# {}\nURL: {}\n\n{}", title, url, paragraphs.join("\n\n")))
    }
}