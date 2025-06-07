use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    OpenAI,
    Ollama,
    Claude,
}

impl std::fmt::Display for AIProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIProvider::OpenAI => write!(f, "openai"),
            AIProvider::Ollama => write!(f, "ollama"),
            AIProvider::Claude => write!(f, "claude"),
        }
    }
}

impl std::str::FromStr for AIProvider {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "openai" | "gpt" => Ok(AIProvider::OpenAI),
            "ollama" => Ok(AIProvider::Ollama),
            "claude" => Ok(AIProvider::Claude),
            _ => Err(anyhow!("Unknown AI provider: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: AIProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl Default for AIConfig {
    fn default() -> Self {
        AIConfig {
            provider: AIProvider::Ollama,
            model: "llama2".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            max_tokens: Some(2048),
            temperature: Some(0.7),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub tokens_used: Option<u32>,
    pub model: String,
}

pub struct AIProviderClient {
    config: AIConfig,
    http_client: reqwest::Client,
}

impl AIProviderClient {
    pub fn new(config: AIConfig) -> Self {
        let http_client = reqwest::Client::new();
        
        AIProviderClient {
            config,
            http_client,
        }
    }
    
    pub async fn chat(&self, messages: Vec<ChatMessage>, system_prompt: Option<String>) -> Result<ChatResponse> {
        match self.config.provider {
            AIProvider::OpenAI => self.chat_openai(messages, system_prompt).await,
            AIProvider::Ollama => self.chat_ollama(messages, system_prompt).await,
            AIProvider::Claude => self.chat_claude(messages, system_prompt).await,
        }
    }
    
    async fn chat_openai(&self, messages: Vec<ChatMessage>, system_prompt: Option<String>) -> Result<ChatResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow!("OpenAI API key required"))?;
        
        let mut request_messages = Vec::new();
        
        // Add system prompt if provided
        if let Some(system) = system_prompt {
            request_messages.push(serde_json::json!({
                "role": "system",
                "content": system
            }));
        }
        
        // Add conversation messages
        for msg in messages {
            request_messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content
            }));
        }
        
        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": request_messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature
        });
        
        let response = self.http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid OpenAI response format"))?
            .to_string();
        
        let tokens_used = response_json["usage"]["total_tokens"]
            .as_u64()
            .map(|t| t as u32);
        
        Ok(ChatResponse {
            content,
            tokens_used,
            model: self.config.model.clone(),
        })
    }
    
    async fn chat_ollama(&self, messages: Vec<ChatMessage>, system_prompt: Option<String>) -> Result<ChatResponse> {
        let default_url = "http://localhost:11434".to_string();
        let base_url = self.config.base_url.as_ref()
            .unwrap_or(&default_url);
        
        let mut request_messages = Vec::new();
        
        // Add system prompt if provided
        if let Some(system) = system_prompt {
            request_messages.push(serde_json::json!({
                "role": "system",
                "content": system
            }));
        }
        
        // Add conversation messages
        for msg in messages {
            request_messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content
            }));
        }
        
        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": request_messages,
            "stream": false
        });
        
        let url = format!("{}/api/chat", base_url);
        let response = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama API error: {}", error_text));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        let content = response_json["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Ollama response format"))?
            .to_string();
        
        Ok(ChatResponse {
            content,
            tokens_used: None, // Ollama doesn't typically return token counts
            model: self.config.model.clone(),
        })
    }
    
    async fn chat_claude(&self, _messages: Vec<ChatMessage>, _system_prompt: Option<String>) -> Result<ChatResponse> {
        // Claude API implementation would go here
        // For now, return a placeholder
        Err(anyhow!("Claude provider not yet implemented"))
    }
    
    pub fn get_model(&self) -> &str {
        &self.config.model
    }
    
    pub fn get_provider(&self) -> &AIProvider {
        &self.config.provider
    }
}

// Convenience functions for creating common message types
impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        ChatMessage {
            role: "user".to_string(),
            content: content.into(),
        }
    }
    
    pub fn assistant(content: impl Into<String>) -> Self {
        ChatMessage {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
    
    pub fn system(content: impl Into<String>) -> Self {
        ChatMessage {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}