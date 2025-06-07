use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

use crate::ai_provider::{AIConfig, AIProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub default_provider: String,
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub default_model: String,
    pub host: Option<String>,
    pub api_key: Option<String>,
}

impl Config {
    pub fn new(data_dir: Option<PathBuf>) -> Result<Self> {
        let data_dir = data_dir.unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("syui")
                .join("ai")
                .join("gpt")
        });
        
        // Ensure data directory exists
        std::fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;
        
        // Create default providers
        let mut providers = HashMap::new();
        
        providers.insert("ollama".to_string(), ProviderConfig {
            default_model: "qwen2.5".to_string(),
            host: Some("http://localhost:11434".to_string()),
            api_key: None,
        });
        
        providers.insert("openai".to_string(), ProviderConfig {
            default_model: "gpt-4o-mini".to_string(),
            host: None,
            api_key: std::env::var("OPENAI_API_KEY").ok(),
        });
        
        Ok(Config {
            data_dir,
            default_provider: "ollama".to_string(),
            providers,
        })
    }
    
    pub fn get_provider(&self, provider_name: &str) -> Option<&ProviderConfig> {
        self.providers.get(provider_name)
    }
    
    pub fn get_ai_config(&self, provider: Option<String>, model: Option<String>) -> Result<AIConfig> {
        let provider_name = provider.as_deref().unwrap_or(&self.default_provider);
        let provider_config = self.get_provider(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", provider_name))?;
        
        let ai_provider: AIProvider = provider_name.parse()?;
        let model_name = model.unwrap_or_else(|| provider_config.default_model.clone());
        
        Ok(AIConfig {
            provider: ai_provider,
            model: model_name,
            api_key: provider_config.api_key.clone(),
            base_url: provider_config.host.clone(),
            max_tokens: Some(2048),
            temperature: Some(0.7),
        })
    }
    
    pub fn memory_file(&self) -> PathBuf {
        self.data_dir.join("memories.json")
    }
    
    pub fn relationships_file(&self) -> PathBuf {
        self.data_dir.join("relationships.json")
    }
    
    pub fn fortune_file(&self) -> PathBuf {
        self.data_dir.join("fortune.json")
    }
    
    pub fn transmission_file(&self) -> PathBuf {
        self.data_dir.join("transmissions.json")
    }
    
    pub fn scheduler_tasks_file(&self) -> PathBuf {
        self.data_dir.join("scheduler_tasks.json")
    }
    
    pub fn scheduler_history_file(&self) -> PathBuf {
        self.data_dir.join("scheduler_history.json")
    }
}