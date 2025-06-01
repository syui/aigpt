// src/mcp/memory.rs
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationImportRequest {
    pub conversation_data: Value,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub error: Option<String>,
    #[allow(dead_code)]
    pub message: Option<String>,
    pub filepath: Option<String>,
    pub results: Option<Vec<MemoryResult>>,
    pub memories: Option<Vec<MemoryResult>>,
    #[allow(dead_code)]
    pub count: Option<usize>,
    pub memory: Option<Value>,
    pub response: Option<String>,
    pub memories_used: Option<usize>,
    pub imported_count: Option<usize>,
    pub total_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct MemoryResult {
    #[allow(dead_code)]
    pub filepath: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub source: Option<String>,
    pub import_time: Option<String>,
    pub message_count: Option<usize>,
}

pub struct MemoryClient {
    base_url: String,
    client: reqwest::Client,
}

impl MemoryClient {
    pub fn new(base_url: Option<String>) -> Self {
        let url = base_url.unwrap_or_else(|| "http://127.0.0.1:5000".to_string());
        Self {
            base_url: url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn import_chatgpt_file(&self, filepath: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
        // ファイルを読み込み
        let content = fs::read_to_string(filepath)?;
        let json_data: Value = serde_json::from_str(&content)?;

        // 配列かどうかチェック
        match json_data.as_array() {
            Some(conversations) => {
                // 複数の会話をインポート
                let mut imported_count = 0;
                let total_count = conversations.len();
                
                for conversation in conversations {
                    match self.import_single_conversation(conversation.clone()).await {
                        Ok(response) => {
                            if response.success {
                                imported_count += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ インポートエラー: {}", e);
                        }
                    }
                }

                Ok(ApiResponse {
                    success: true,
                    imported_count: Some(imported_count),
                    total_count: Some(total_count),
                    error: None,
                    message: Some(format!("{}個中{}個の会話をインポートしました", total_count, imported_count)),
                    filepath: None,
                    results: None,
                    memories: None,
                    count: None,
                    memory: None,
                    response: None,
                    memories_used: None,
                })
            }
            None => {
                // 単一の会話をインポート
                self.import_single_conversation(json_data).await
            }
        }
    }

    async fn import_single_conversation(&self, conversation_data: Value) -> Result<ApiResponse, Box<dyn std::error::Error>> {
        let request = ConversationImportRequest { conversation_data };
        
        let response = self.client
            .post(&format!("{}/memory/import/chatgpt", self.base_url))
            .json(&request)
            .send()
            .await?;

        let result: ApiResponse = response.json().await?;
        Ok(result)
    }

    pub async fn search_memories(&self, query: &str, limit: usize) -> Result<ApiResponse, Box<dyn std::error::Error>> {
        let request = MemorySearchRequest {
            query: query.to_string(),
            limit,
        };

        let response = self.client
            .post(&format!("{}/memory/search", self.base_url))
            .json(&request)
            .send()
            .await?;

        let result: ApiResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_memories(&self) -> Result<ApiResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .get(&format!("{}/memory/list", self.base_url))
            .send()
            .await?;

        let result: ApiResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_memory_detail(&self, filepath: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .get(&format!("{}/memory/detail", self.base_url))
            .query(&[("filepath", filepath)])
            .send()
            .await?;

        let result: ApiResponse = response.json().await?;
        Ok(result)
    }

    pub async fn chat_with_memory(&self, message: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
        let request = ChatRequest {
            message: message.to_string(),
            model: None,
        };

        let response = self.client
            .post(&format!("{}/chat", self.base_url))
            .json(&request)
            .send()
            .await?;

        let result: ApiResponse = response.json().await?;
        Ok(result)
    }

    pub async fn is_server_running(&self) -> bool {
        match self.client.get(&self.base_url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

pub async fn handle_import(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(filepath).exists() {
        eprintln!("❌ ファイルが見つかりません: {}", filepath);
        return Ok(());
    }

    let client = MemoryClient::new(None);
    
    // サーバーが起動しているかチェック
    if !client.is_server_running().await {
        eprintln!("❌ MCP Serverが起動していません。先に 'aigpt server run' を実行してください。");
        return Ok(());
    }

    println!("🔄 ChatGPT会話をインポートしています: {}", filepath);
    
    match client.import_chatgpt_file(filepath).await {
        Ok(response) => {
            if response.success {
                if let (Some(imported), Some(total)) = (response.imported_count, response.total_count) {
                    println!("✅ {}個中{}個の会話をインポートしました", total, imported);
                } else {
                    println!("✅ 会話をインポートしました");
                    if let Some(path) = response.filepath {
                        println!("📁 保存先: {}", path);
                    }
                }
            } else {
                eprintln!("❌ インポートに失敗: {:?}", response.error);
            }
        }
        Err(e) => {
            eprintln!("❌ インポートエラー: {}", e);
        }
    }

    Ok(())
}

pub async fn handle_search(query: &str, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let client = MemoryClient::new(None);
    
    if !client.is_server_running().await {
        eprintln!("❌ MCP Serverが起動していません。先に 'aigpt server run' を実行してください。");
        return Ok(());
    }

    println!("🔍 記憶を検索しています: {}", query);
    
    match client.search_memories(query, limit).await {
        Ok(response) => {
            if response.success {
                if let Some(results) = response.results {
                    println!("📚 {}個の記憶が見つかりました:", results.len());
                    for memory in results {
                        println!("  • {}", memory.title.unwrap_or_else(|| "タイトルなし".to_string()));
                        if let Some(summary) = memory.summary {
                            println!("    概要: {}", summary);
                        }
                        if let Some(count) = memory.message_count {
                            println!("    メッセージ数: {}", count);
                        }
                        println!();
                    }
                } else {
                    println!("📚 記憶が見つかりませんでした");
                }
            } else {
                eprintln!("❌ 検索に失敗: {:?}", response.error);
            }
        }
        Err(e) => {
            eprintln!("❌ 検索エラー: {}", e);
        }
    }

    Ok(())
}

pub async fn handle_list() -> Result<(), Box<dyn std::error::Error>> {
    let client = MemoryClient::new(None);
    
    if !client.is_server_running().await {
        eprintln!("❌ MCP Serverが起動していません。先に 'aigpt server run' を実行してください。");
        return Ok(());
    }

    println!("📋 記憶一覧を取得しています...");
    
    match client.list_memories().await {
        Ok(response) => {
            if response.success {
                if let Some(memories) = response.memories {
                    println!("📚 総記憶数: {}", memories.len());
                    for memory in memories {
                        println!("  • {}", memory.title.unwrap_or_else(|| "タイトルなし".to_string()));
                        if let Some(source) = memory.source {
                            println!("    ソース: {}", source);
                        }
                        if let Some(count) = memory.message_count {
                            println!("    メッセージ数: {}", count);
                        }
                        if let Some(import_time) = memory.import_time {
                            println!("    インポート時刻: {}", import_time);
                        }
                        println!();
                    }
                } else {
                    println!("📚 記憶がありません");
                }
            } else {
                eprintln!("❌ 一覧取得に失敗: {:?}", response.error);
            }
        }
        Err(e) => {
            eprintln!("❌ 一覧取得エラー: {}", e);
        }
    }

    Ok(())
}

pub async fn handle_detail(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = MemoryClient::new(None);
    
    if !client.is_server_running().await {
        eprintln!("❌ MCP Serverが起動していません。先に 'aigpt server run' を実行してください。");
        return Ok(());
    }

    println!("📄 記憶の詳細を取得しています: {}", filepath);
    
    match client.get_memory_detail(filepath).await {
        Ok(response) => {
            if response.success {
                if let Some(memory) = response.memory {
                    if let Some(title) = memory.get("title").and_then(|v| v.as_str()) {
                        println!("タイトル: {}", title);
                    }
                    if let Some(source) = memory.get("source").and_then(|v| v.as_str()) {
                        println!("ソース: {}", source);
                    }
                    if let Some(summary) = memory.get("summary").and_then(|v| v.as_str()) {
                        println!("概要: {}", summary);
                    }
                    if let Some(messages) = memory.get("messages").and_then(|v| v.as_array()) {
                        println!("メッセージ数: {}", messages.len());
                        println!("\n最近のメッセージ:");
                        for msg in messages.iter().take(5) {
                            if let (Some(role), Some(content)) = (
                                msg.get("role").and_then(|v| v.as_str()),
                                msg.get("content").and_then(|v| v.as_str())
                            ) {
                                let content_preview = if content.len() > 100 {
                                    format!("{}...", &content[..100])
                                } else {
                                    content.to_string()
                                };
                                println!("  {}: {}", role, content_preview);
                            }
                        }
                    }
                }
            } else {
                eprintln!("❌ 詳細取得に失敗: {:?}", response.error);
            }
        }
        Err(e) => {
            eprintln!("❌ 詳細取得エラー: {}", e);
        }
    }

    Ok(())
}

pub async fn handle_chat_with_memory(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = MemoryClient::new(None);
    
    if !client.is_server_running().await {
        eprintln!("❌ MCP Serverが起動していません。先に 'aigpt server run' を実行してください。");
        return Ok(());
    }

    println!("💬 記憶を活用してチャットしています...");
    
    match client.chat_with_memory(message).await {
        Ok(response) => {
            if response.success {
                if let Some(reply) = response.response {
                    println!("🤖 {}", reply);
                }
                if let Some(memories_used) = response.memories_used {
                    println!("📚 使用した記憶数: {}", memories_used);
                }
            } else {
                eprintln!("❌ チャットに失敗: {:?}", response.error);
            }
        }
        Err(e) => {
            eprintln!("❌ チャットエラー: {}", e);
        }
    }

    Ok(())
}
