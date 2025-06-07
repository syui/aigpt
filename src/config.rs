use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

use crate::ai_provider::{AIConfig, AIProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub data_dir: PathBuf,
    pub default_provider: String,
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub atproto: Option<AtprotoConfig>,
    #[serde(default)]
    pub mcp: Option<McpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub default_model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtprotoConfig {
    pub handle: Option<String>,
    pub password: Option<String>,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(deserialize_with = "string_to_bool")]
    pub enabled: bool,
    #[serde(deserialize_with = "string_to_bool")]
    pub auto_detect: bool,
    pub servers: HashMap<String, McpServerConfig>,
}

fn string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::custom("expected 'true' or 'false'")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub base_url: String,
    pub name: String,
    #[serde(deserialize_with = "string_to_f64")]
    pub timeout: f64,
    pub endpoints: HashMap<String, String>,
}

fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
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
        
        let config_path = data_dir.join("config.json");
        
        // Try to load existing config
        if config_path.exists() {
            let config_str = std::fs::read_to_string(&config_path)
                .context("Failed to read config.json")?;
            
            // Check if file is empty
            if config_str.trim().is_empty() {
                eprintln!("Config file is empty, will recreate from source");
            } else {
                match serde_json::from_str::<Config>(&config_str) {
                    Ok(mut config) => {
                        config.data_dir = data_dir;
                        // Check for environment variables if API keys are empty
                        if let Some(openai_config) = config.providers.get_mut("openai") {
                            if openai_config.api_key.as_ref().map_or(true, |key| key.is_empty()) {
                                openai_config.api_key = std::env::var("OPENAI_API_KEY").ok();
                            }
                        }
                        return Ok(config);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse existing config.json: {}", e);
                        eprintln!("Will try to reload from source...");
                    }
                }
            }
        }
        
        // Check if we need to migrate from JSON
        // Try multiple locations for the JSON file
        let possible_json_paths = vec![
            PathBuf::from("../config.json"),  // Relative to aigpt-rs directory
            PathBuf::from("config.json"),     // Current directory
            PathBuf::from("gpt/config.json"), // From project root
            PathBuf::from("/Users/syui/ai/ai/gpt/config.json"), // Absolute path
        ];
        
        for json_path in possible_json_paths {
            if json_path.exists() {
                eprintln!("Found config.json at: {}", json_path.display());
                eprintln!("Copying configuration...");
                // Copy configuration file and parse it
                std::fs::copy(&json_path, &config_path)
                    .context("Failed to copy config.json")?;
                
                let config_str = std::fs::read_to_string(&config_path)
                    .context("Failed to read copied config.json")?;
                
                println!("Config JSON content preview: {}", &config_str[..std::cmp::min(200, config_str.len())]);
                
                let mut config: Config = serde_json::from_str(&config_str)
                    .context("Failed to parse config.json")?;
                config.data_dir = data_dir;
                // Check for environment variables if API keys are empty
                if let Some(openai_config) = config.providers.get_mut("openai") {
                    if openai_config.api_key.as_ref().map_or(true, |key| key.is_empty()) {
                        openai_config.api_key = std::env::var("OPENAI_API_KEY").ok();
                    }
                }
                eprintln!("Copy complete! Config saved to: {}", config_path.display());
                return Ok(config);
            }
        }
        
        // Create default config
        let config = Self::default_config(data_dir);
        
        // Save default config
        let json_str = serde_json::to_string_pretty(&config)
            .context("Failed to serialize default config")?;
        std::fs::write(&config_path, json_str)
            .context("Failed to write default config.json")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = self.data_dir.join("config.json");
        let json_str = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(&config_path, json_str)
            .context("Failed to write config.json")?;
        Ok(())
    }
    
    fn default_config(data_dir: PathBuf) -> Self {
        let mut providers = HashMap::new();
        
        providers.insert("ollama".to_string(), ProviderConfig {
            default_model: "qwen2.5".to_string(),
            host: Some("http://localhost:11434".to_string()),
            api_key: None,
            system_prompt: None,
        });
        
        providers.insert("openai".to_string(), ProviderConfig {
            default_model: "gpt-4o-mini".to_string(),
            host: None,
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            system_prompt: None,
        });
        
        Config {
            data_dir,
            default_provider: "ollama".to_string(),
            providers,
            atproto: None,
            mcp: None,
        }
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