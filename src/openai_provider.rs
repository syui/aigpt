use anyhow::Result;
use async_openai::{
    types::{
        ChatCompletionRequestMessage, 
        CreateChatCompletionRequestArgs, ChatCompletionTool, ChatCompletionToolType,
        FunctionObject, ChatCompletionRequestToolMessage,
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestUserMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionToolChoiceOption
    },
    Client,
};
use serde_json::{json, Value};

use crate::http_client::ServiceClient;

/// OpenAI provider with MCP tools support (matching Python implementation)
pub struct OpenAIProvider {
    client: Client<async_openai::config::OpenAIConfig>,
    model: String,
    service_client: ServiceClient,
    system_prompt: Option<String>,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_key(api_key);
        let client = Client::with_config(config);
        
        Self {
            client,
            model: model.unwrap_or_else(|| "gpt-4".to_string()),
            service_client: ServiceClient::new(),
            system_prompt: None,
        }
    }

    pub fn with_system_prompt(api_key: String, model: Option<String>, system_prompt: Option<String>) -> Self {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_key(api_key);
        let client = Client::with_config(config);
        
        Self {
            client,
            model: model.unwrap_or_else(|| "gpt-4".to_string()),
            service_client: ServiceClient::new(),
            system_prompt,
        }
    }

    /// Generate OpenAI tools from MCP endpoints (matching Python implementation)
    fn get_mcp_tools(&self) -> Vec<ChatCompletionTool> {
        let tools = vec![
            // Memory tools
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "get_memories".to_string(),
                    description: Some("Get past conversation memories".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "取得する記憶の数",
                                "default": 5
                            }
                        }
                    })),
                },
            },
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "search_memories".to_string(),
                    description: Some("Search memories for specific topics or keywords".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "keywords": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "検索キーワードの配列"
                            }
                        },
                        "required": ["keywords"]
                    })),
                },
            },
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "get_contextual_memories".to_string(),
                    description: Some("Get contextual memories related to a query".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "検索クエリ"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "取得する記憶の数",
                                "default": 5
                            }
                        },
                        "required": ["query"]
                    })),
                },
            },
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "get_relationship".to_string(),
                    description: Some("Get relationship information with a specific user".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "user_id": {
                                "type": "string",
                                "description": "ユーザーID"
                            }
                        },
                        "required": ["user_id"]
                    })),
                },
            },
            // ai.card tools
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "card_get_user_cards".to_string(),
                    description: Some("Get user's card collection".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "did": {
                                "type": "string",
                                "description": "ユーザーのDID"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "取得するカード数の上限",
                                "default": 10
                            }
                        },
                        "required": ["did"]
                    })),
                },
            },
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "card_draw_card".to_string(),
                    description: Some("Draw a card from the gacha system".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "did": {
                                "type": "string",
                                "description": "ユーザーのDID"
                            },
                            "is_paid": {
                                "type": "boolean",
                                "description": "有料ガチャかどうか",
                                "default": false
                            }
                        },
                        "required": ["did"]
                    })),
                },
            },
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "card_analyze_collection".to_string(),
                    description: Some("Analyze user's card collection".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "did": {
                                "type": "string",
                                "description": "ユーザーのDID"
                            }
                        },
                        "required": ["did"]
                    })),
                },
            },
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "card_get_gacha_stats".to_string(),
                    description: Some("Get gacha statistics".to_string()),
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {}
                    })),
                },
            },
        ];

        tools
    }

    /// Chat interface with MCP function calling support (matching Python implementation)
    pub async fn chat_with_mcp(&self, prompt: String, user_id: String) -> Result<String> {
        let tools = self.get_mcp_tools();
        
        
        let system_content = self.system_prompt.as_deref().unwrap_or(
            "You are an AI assistant with access to memory, relationship data, and card game systems. Use the available tools when appropriate to provide accurate and contextual responses."
        );
        

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage {
                        content: system_content.to_string().into(),
                        name: None,
                    }
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: prompt.clone().into(),
                        name: None,
                    }
                ),
            ])
            .tools(tools.clone())
            .tool_choice(ChatCompletionToolChoiceOption::Auto)
            .max_tokens(2000u16)
            .temperature(0.7)
            .build()?;


        let response = self.client.chat().create(request).await?;
        let message = &response.choices[0].message;


        // Handle tool calls
        if let Some(tool_calls) = &message.tool_calls {
            if tool_calls.is_empty() {
                println!("🔧 [OpenAI] No tools called (empty array)");
            } else {
                println!("🔧 [OpenAI] {} tools called:", tool_calls.len());
                for tc in tool_calls {
                    println!("  - {}({})", tc.function.name, tc.function.arguments);
                }
            }
        } else {
            println!("🔧 [OpenAI] No tools called (no tool_calls field)");
        }

        // Process tool calls if any
        if let Some(tool_calls) = &message.tool_calls {
            if !tool_calls.is_empty() {

            let mut messages = vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage {
                        content: system_content.to_string().into(),
                        name: None,
                    }
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: prompt.into(),
                        name: None,
                    }
                ),
                #[allow(deprecated)]
                ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage {
                        content: message.content.clone(),
                        name: None,
                        tool_calls: message.tool_calls.clone(),
                        function_call: None,
                    }
                ),
            ];

            // Execute each tool call
            for tool_call in tool_calls {
                println!("🌐 [MCP] Executing {}...", tool_call.function.name);
                let tool_result = self.execute_mcp_tool(tool_call, &user_id).await?;
                let result_preview = serde_json::to_string(&tool_result)?;
                let preview = if result_preview.chars().count() > 100 {
                    format!("{}...", result_preview.chars().take(100).collect::<String>())
                } else {
                    result_preview.clone()
                };
                println!("✅ [MCP] Result: {}", preview);

                messages.push(ChatCompletionRequestMessage::Tool(
                    ChatCompletionRequestToolMessage {
                        content: serde_json::to_string(&tool_result)?,
                        tool_call_id: tool_call.id.clone(),
                    }
                ));
            }

            // Get final response with tool outputs
            let final_request = CreateChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(messages)
                .max_tokens(2000u16)
                .temperature(0.7)
                .build()?;

            let final_response = self.client.chat().create(final_request).await?;
                Ok(final_response.choices[0].message.content.as_ref().unwrap_or(&"".to_string()).clone())
            } else {
                // No tools were called
                Ok(message.content.as_ref().unwrap_or(&"".to_string()).clone())
            }
        } else {
            // No tool_calls field at all
            Ok(message.content.as_ref().unwrap_or(&"".to_string()).clone())
        }
    }

    /// Execute MCP tool call (matching Python implementation)
    async fn execute_mcp_tool(&self, tool_call: &async_openai::types::ChatCompletionMessageToolCall, context_user_id: &str) -> Result<Value> {
        let function_name = &tool_call.function.name;
        let arguments: Value = serde_json::from_str(&tool_call.function.arguments)?;

        match function_name.as_str() {
            "get_memories" => {
                let limit = arguments.get("limit").and_then(|v| v.as_i64()).unwrap_or(5);
                
                // MCP server call to get memories
                match self.service_client.get_request(&format!("http://localhost:8080/memories/{}", context_user_id)).await {
                    Ok(result) => {
                        // Extract the actual memory content from MCP response
                        if let Some(content) = result.get("result").and_then(|r| r.get("content")) {
                            if let Some(text_content) = content.get(0).and_then(|c| c.get("text")) {
                                // Parse the text content as JSON (it's a serialized array)
                                if let Ok(memories_array) = serde_json::from_str::<Vec<String>>(text_content.as_str().unwrap_or("[]")) {
                                    let limited_memories: Vec<String> = memories_array.into_iter().take(limit as usize).collect();
                                    Ok(json!({
                                        "memories": limited_memories,
                                        "count": limited_memories.len()
                                    }))
                                } else {
                                    Ok(json!({
                                        "memories": [text_content.as_str().unwrap_or("No memories found")],
                                        "count": 1
                                    }))
                                }
                            } else {
                                Ok(json!({"memories": [], "count": 0, "info": "No memories available"}))
                            }
                        } else {
                            Ok(json!({"memories": [], "count": 0, "info": "No response from memory service"}))
                        }
                    }
                    Err(e) => {
                        Ok(json!({"error": format!("Failed to retrieve memories: {}", e)}))
                    }
                }
            }
            "search_memories" => {
                let keywords = arguments.get("keywords").and_then(|v| v.as_array()).unwrap_or(&vec![]).clone();
                
                // Convert keywords to strings
                let keyword_strings: Vec<String> = keywords.iter()
                    .filter_map(|k| k.as_str().map(|s| s.to_string()))
                    .collect();
                
                if keyword_strings.is_empty() {
                    return Ok(json!({"error": "No keywords provided for search"}));
                }
                
                // MCP server call to search memories
                let search_request = json!({
                    "keywords": keyword_strings
                });
                
                match self.service_client.post_request(
                    &format!("http://localhost:8080/memories/{}/search", context_user_id),
                    &search_request
                ).await {
                    Ok(result) => {
                        // Extract the actual memory content from MCP response
                        if let Some(content) = result.get("result").and_then(|r| r.get("content")) {
                            if let Some(text_content) = content.get(0).and_then(|c| c.get("text")) {
                                // Parse the search results
                                if let Ok(search_result) = serde_json::from_str::<Vec<Value>>(text_content.as_str().unwrap_or("[]")) {
                                    let memory_contents: Vec<String> = search_result.iter()
                                        .filter_map(|item| item.get("content").and_then(|c| c.as_str().map(|s| s.to_string())))
                                        .collect();
                                    
                                    Ok(json!({
                                        "memories": memory_contents,
                                        "count": memory_contents.len(),
                                        "keywords": keyword_strings
                                    }))
                                } else {
                                    Ok(json!({
                                        "memories": [],
                                        "count": 0,
                                        "info": format!("No memories found for keywords: {}", keyword_strings.join(", "))
                                    }))
                                }
                            } else {
                                Ok(json!({"memories": [], "count": 0, "info": "No search results available"}))
                            }
                        } else {
                            Ok(json!({"memories": [], "count": 0, "info": "No response from search service"}))
                        }
                    }
                    Err(e) => {
                        Ok(json!({"error": format!("Failed to search memories: {}", e)}))
                    }
                }
            }
            "get_contextual_memories" => {
                let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = arguments.get("limit").and_then(|v| v.as_i64()).unwrap_or(5);
                
                if query.is_empty() {
                    return Ok(json!({"error": "No query provided for contextual search"}));
                }
                
                // MCP server call to get contextual memories
                let contextual_request = json!({
                    "query": query,
                    "limit": limit
                });
                
                match self.service_client.post_request(
                    &format!("http://localhost:8080/memories/{}/contextual", context_user_id),
                    &contextual_request
                ).await {
                    Ok(result) => {
                        // Extract the actual memory content from MCP response
                        if let Some(content) = result.get("result").and_then(|r| r.get("content")) {
                            if let Some(text_content) = content.get(0).and_then(|c| c.get("text")) {
                                // Parse contextual search results
                                if text_content.as_str().unwrap_or("").contains("Found") {
                                    // Extract memories from the formatted text response
                                    let text = text_content.as_str().unwrap_or("");
                                    if let Some(json_start) = text.find('[') {
                                        if let Ok(memories_result) = serde_json::from_str::<Vec<Value>>(&text[json_start..]) {
                                            let memory_contents: Vec<String> = memories_result.iter()
                                                .filter_map(|item| item.get("content").and_then(|c| c.as_str().map(|s| s.to_string())))
                                                .collect();
                                            
                                            Ok(json!({
                                                "memories": memory_contents,
                                                "count": memory_contents.len(),
                                                "query": query
                                            }))
                                        } else {
                                            Ok(json!({
                                                "memories": [],
                                                "count": 0,
                                                "info": format!("No contextual memories found for: {}", query)
                                            }))
                                        }
                                    } else {
                                        Ok(json!({
                                            "memories": [],
                                            "count": 0,
                                            "info": format!("No contextual memories found for: {}", query)
                                        }))
                                    }
                                } else {
                                    Ok(json!({
                                        "memories": [],
                                        "count": 0,
                                        "info": format!("No contextual memories found for: {}", query)
                                    }))
                                }
                            } else {
                                Ok(json!({"memories": [], "count": 0, "info": "No contextual results available"}))
                            }
                        } else {
                            Ok(json!({"memories": [], "count": 0, "info": "No response from contextual search service"}))
                        }
                    }
                    Err(e) => {
                        Ok(json!({"error": format!("Failed to get contextual memories: {}", e)}))
                    }
                }
            }
            "get_relationship" => {
                let target_user_id = arguments.get("user_id").and_then(|v| v.as_str()).unwrap_or(context_user_id);
                
                // MCP server call to get relationship status
                match self.service_client.get_request(&format!("http://localhost:8080/status/{}", target_user_id)).await {
                    Ok(result) => {
                        // Extract relationship information from MCP response
                        if let Some(content) = result.get("result").and_then(|r| r.get("content")) {
                            if let Some(text_content) = content.get(0).and_then(|c| c.get("text")) {
                                // Parse the status response to extract relationship data
                                if let Ok(status_data) = serde_json::from_str::<Value>(text_content.as_str().unwrap_or("{}")) {
                                    if let Some(relationship) = status_data.get("relationship") {
                                        Ok(json!({
                                            "relationship": relationship,
                                            "user_id": target_user_id
                                        }))
                                    } else {
                                        Ok(json!({
                                            "info": format!("No relationship found for user: {}", target_user_id),
                                            "user_id": target_user_id
                                        }))
                                    }
                                } else {
                                    Ok(json!({
                                        "info": format!("Could not parse relationship data for user: {}", target_user_id),
                                        "user_id": target_user_id
                                    }))
                                }
                            } else {
                                Ok(json!({"info": "No relationship data available", "user_id": target_user_id}))
                            }
                        } else {
                            Ok(json!({"info": "No response from relationship service", "user_id": target_user_id}))
                        }
                    }
                    Err(e) => {
                        Ok(json!({"error": format!("Failed to get relationship: {}", e)}))
                    }
                }
            }
            // ai.card tools
            "card_get_user_cards" => {
                let did = arguments.get("did").and_then(|v| v.as_str()).unwrap_or(context_user_id);
                let _limit = arguments.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
                
                match self.service_client.get_user_cards(did).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        println!("❌ ai.card API error: {}", e);
                        Ok(json!({
                            "error": "ai.cardサーバーが起動していません",
                            "message": "カードシステムを使用するには、ai.cardサーバーを起動してください"
                        }))
                    }
                }
            }
            "card_draw_card" => {
                let did = arguments.get("did").and_then(|v| v.as_str()).unwrap_or(context_user_id);
                let is_paid = arguments.get("is_paid").and_then(|v| v.as_bool()).unwrap_or(false);
                
                match self.service_client.draw_card(did, is_paid).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        println!("❌ ai.card API error: {}", e);
                        Ok(json!({
                            "error": "ai.cardサーバーが起動していません",
                            "message": "カードシステムを使用するには、ai.cardサーバーを起動してください"
                        }))
                    }
                }
            }
            "card_analyze_collection" => {
                let did = arguments.get("did").and_then(|v| v.as_str()).unwrap_or(context_user_id);
                // TODO: Implement collection analysis endpoint
                Ok(json!({
                    "info": "コレクション分析機能は実装中です",
                    "user_did": did
                }))
            }
            "card_get_gacha_stats" => {
                // TODO: Implement gacha stats endpoint
                Ok(json!({"info": "ガチャ統計機能は実装中です"}))
            }
            _ => {
                Ok(json!({
                    "error": format!("Unknown tool: {}", function_name)
                }))
            }
        }
    }
}
