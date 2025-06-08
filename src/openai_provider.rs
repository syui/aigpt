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

    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    /// Generate OpenAI tools from MCP endpoints (matching Python implementation)
    fn get_mcp_tools(&self) -> Vec<ChatCompletionTool> {
        let tools = vec![
            // Memory tools
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: "get_memories".to_string(),
                    description: Some("過去の会話記憶を取得します。「覚えている」「前回」「以前」などの質問で必ず使用してください".to_string()),
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
                    description: Some("特定のトピックについて話した記憶を検索します。「プログラミングについて」「○○について話した」などの質問で使用してください".to_string()),
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
                    description: Some("クエリに関連する文脈的記憶を取得します".to_string()),
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
                    description: Some("特定ユーザーとの関係性情報を取得します".to_string()),
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
                    description: Some("ユーザーが所有するカードの一覧を取得します".to_string()),
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
                    description: Some("ガチャを引いてカードを取得します".to_string()),
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
                    description: Some("ユーザーのカードコレクションを分析します".to_string()),
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
                    description: Some("ガチャの統計情報を取得します".to_string()),
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
            "あなたは記憶システムと関係性データ、カードゲームシステムにアクセスできるAIです。\n\n【重要】以下の場合は必ずツールを使用してください：\n\n1. カード関連の質問:\n- 「カード」「コレクション」「ガチャ」「見せて」「持っている」「状況」「どんなカード」などのキーワードがある場合\n- card_get_user_cardsツールを使用してユーザーのカード情報を取得\n\n2. 記憶・関係性の質問:\n- 「覚えている」「前回」「以前」「について話した」「関係」などのキーワードがある場合\n- 適切なメモリツールを使用\n\n3. パラメータの設定:\n- didパラメータには現在会話しているユーザーのID（例：'syui'）を使用\n- ツールを積極的に使用して正確な情報を提供してください\n\nユーザーが何かを尋ねた時は、まず関連するツールがあるかを考え、適切なツールを使用してから回答してください。"
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
            .tools(tools)
            .tool_choice(ChatCompletionToolChoiceOption::Auto)
            .max_tokens(2000u16)
            .temperature(0.7)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let message = &response.choices[0].message;

        // Handle tool calls
        if let Some(tool_calls) = &message.tool_calls {
            if tool_calls.is_empty() {
                println!("🔧 [OpenAI] No tools called");
            } else {
                println!("🔧 [OpenAI] {} tools called:", tool_calls.len());
                for tc in tool_calls {
                    println!("  - {}({})", tc.function.name, tc.function.arguments);
                }
            }
        } else {
            println!("🔧 [OpenAI] No tools called");
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
                // TODO: Implement actual MCP call
                Ok(json!({"info": "記憶機能は実装中です"}))
            }
            "search_memories" => {
                let _keywords = arguments.get("keywords").and_then(|v| v.as_array());
                // TODO: Implement actual MCP call
                Ok(json!({"info": "記憶検索機能は実装中です"}))
            }
            "get_contextual_memories" => {
                let _query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let _limit = arguments.get("limit").and_then(|v| v.as_i64()).unwrap_or(5);
                // TODO: Implement actual MCP call
                Ok(json!({"info": "文脈記憶機能は実装中です"}))
            }
            "get_relationship" => {
                let _user_id = arguments.get("user_id").and_then(|v| v.as_str()).unwrap_or(context_user_id);
                // TODO: Implement actual MCP call
                Ok(json!({"info": "関係性機能は実装中です"}))
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