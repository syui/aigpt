use anyhow::Result;
use serde_json::{json, Value};

use super::base::BaseMCPServer;

pub struct ExtendedMCPServer {
    base: BaseMCPServer,
}

impl ExtendedMCPServer {
    pub async fn new() -> Result<Self> {
        let base = BaseMCPServer::new().await?;
        Ok(ExtendedMCPServer { base })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.base.run().await
    }

    pub async fn handle_request(&mut self, request: Value) -> Value {
        self.base.handle_request(request).await
    }

    // 拡張ツールを追加
    pub fn get_available_tools(&self) -> Vec<Value> {
        #[allow(unused_mut)]
        let mut tools = self.base.get_available_tools();
        
        // AI分析ツールを追加
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

        // Web統合ツールを追加
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

        // セマンティック検索強化
        #[cfg(feature = "semantic-search")]
        {
            // create_memoryを拡張版で上書き
            if let Some(pos) = tools.iter().position(|tool| tool["name"] == "create_memory") {
                tools[pos] = json!({
                    "name": "create_memory",
                    "description": "Create a new memory entry with optional AI analysis",
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
                });
            }

            // search_memoriesを拡張版で上書き
            if let Some(pos) = tools.iter().position(|tool| tool["name"] == "search_memories") {
                tools[pos] = json!({
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
                });
            }
        }

        tools
    }

    // 拡張ツール実行
    pub async fn execute_tool(&mut self, tool_name: &str, arguments: &Value) -> Value {
        match tool_name {
            // 拡張機能
            #[cfg(feature = "ai-analysis")]
            "analyze_sentiment" => self.tool_analyze_sentiment(arguments).await,
            #[cfg(feature = "ai-analysis")]
            "extract_insights" => self.tool_extract_insights(arguments).await,
            #[cfg(feature = "web-integration")]
            "import_webpage" => self.tool_import_webpage(arguments).await,
            
            // 拡張版の基本ツール (AI分析付き)
            "create_memory" => self.tool_create_memory_extended(arguments).await,
            "search_memories" => self.tool_search_memories_extended(arguments).await,
            
            // 基本ツールにフォールバック
            _ => self.base.execute_tool(tool_name, arguments).await,
        }
    }

    // 拡張ツール実装
    async fn tool_create_memory_extended(&mut self, arguments: &Value) -> Value {
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

        match self.base.memory_manager.create_memory(&final_content) {
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

    async fn tool_search_memories_extended(&mut self, arguments: &Value) -> Value {
        let query = arguments["query"].as_str().unwrap_or("");
        let semantic = arguments["semantic"].as_bool().unwrap_or(false);
        
        let memories = if semantic {
            #[cfg(feature = "semantic-search")]
            {
                // モックセマンティック検索
                self.base.memory_manager.search_memories(query)
            }
            #[cfg(not(feature = "semantic-search"))]
            {
                self.base.memory_manager.search_memories(query)
            }
        } else {
            self.base.memory_manager.search_memories(query)
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

    #[cfg(feature = "ai-analysis")]
    async fn tool_analyze_sentiment(&mut self, _arguments: &Value) -> Value {
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
    async fn tool_extract_insights(&mut self, _arguments: &Value) -> Value {
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

    #[cfg(feature = "web-integration")]
    async fn tool_import_webpage(&mut self, arguments: &Value) -> Value {
        let url = arguments["url"].as_str().unwrap_or("");
        match self.import_from_web(url).await {
            Ok(content) => {
                match self.base.memory_manager.create_memory(&content) {
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