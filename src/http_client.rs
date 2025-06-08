use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::collections::HashMap;

/// Service configuration for unified service management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub health_endpoint: String,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000".to_string(),
            timeout: Duration::from_secs(30),
            health_endpoint: "/health".to_string(),
        }
    }
}

/// HTTP client for inter-service communication
pub struct ServiceClient {
    client: Client,
    service_registry: HashMap<String, ServiceConfig>,
}

impl ServiceClient {
    pub fn new() -> Self {
        Self::with_default_services()
    }

    /// Create ServiceClient with default ai ecosystem services
    pub fn with_default_services() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        let mut service_registry = HashMap::new();
        
        // Register default ai ecosystem services
        service_registry.insert("ai.card".to_string(), ServiceConfig {
            base_url: "http://localhost:8000".to_string(),
            timeout: Duration::from_secs(30),
            health_endpoint: "/health".to_string(),
        });
        
        service_registry.insert("ai.log".to_string(), ServiceConfig {
            base_url: "http://localhost:8002".to_string(),
            timeout: Duration::from_secs(30), 
            health_endpoint: "/health".to_string(),
        });
        
        service_registry.insert("ai.bot".to_string(), ServiceConfig {
            base_url: "http://localhost:8003".to_string(),
            timeout: Duration::from_secs(30),
            health_endpoint: "/health".to_string(),
        });
        
        Self { client, service_registry }
    }

    /// Create ServiceClient with custom service registry
    pub fn with_services(service_registry: HashMap<String, ServiceConfig>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, service_registry }
    }

    /// Register a new service configuration
    pub fn register_service(&mut self, name: String, config: ServiceConfig) {
        self.service_registry.insert(name, config);
    }

    /// Get service configuration by name
    pub fn get_service_config(&self, service: &str) -> Result<&ServiceConfig> {
        self.service_registry.get(service)
            .ok_or_else(|| anyhow!("Unknown service: {}", service))
    }

    /// Universal service method call
    pub async fn call_service_method<T: Serialize>(
        &self,
        service: &str,
        method: &str,
        params: &T
    ) -> Result<Value> {
        let config = self.get_service_config(service)?;
        let url = format!("{}/{}", config.base_url.trim_end_matches('/'), method.trim_start_matches('/'));
        
        self.post_request(&url, &serde_json::to_value(params)?).await
    }

    /// Universal service GET call
    pub async fn call_service_get(&self, service: &str, endpoint: &str) -> Result<Value> {
        let config = self.get_service_config(service)?;
        let url = format!("{}/{}", config.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        self.get_request(&url).await
    }

    /// Check if a service is available
    pub async fn check_service_status(&self, base_url: &str) -> Result<ServiceStatus> {
        let url = format!("{}/health", base_url.trim_end_matches('/'));
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(ServiceStatus::Available)
                } else {
                    Ok(ServiceStatus::Error(format!("HTTP {}", response.status())))
                }
            }
            Err(e) => Ok(ServiceStatus::Unavailable(e.to_string())),
        }
    }

    /// Make a GET request to a service
    pub async fn get_request(&self, url: &str) -> Result<Value> {
        let response = self.client
            .get(url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Request failed with status: {}", response.status()));
        }

        let json: Value = response.json().await?;
        Ok(json)
    }

    /// Make a POST request to a service
    pub async fn post_request(&self, url: &str, body: &Value) -> Result<Value> {
        let response = self.client
            .post(url)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Request failed with status: {}", response.status()));
        }

        let json: Value = response.json().await?;
        Ok(json)
    }

    /// Get user's card collection from ai.card service
    pub async fn get_user_cards(&self, user_did: &str) -> Result<Value> {
        let endpoint = format!("api/v1/cards/user/{}", user_did);
        self.call_service_get("ai.card", &endpoint).await
    }

    /// Draw a card for user from ai.card service
    pub async fn draw_card(&self, user_did: &str, is_paid: bool) -> Result<Value> {
        let params = serde_json::json!({
            "user_did": user_did,
            "is_paid": is_paid
        });

        self.call_service_method("ai.card", "api/v1/cards/draw", &params).await
    }

    /// Get card statistics from ai.card service
    pub async fn get_card_stats(&self) -> Result<Value> {
        self.call_service_get("ai.card", "api/v1/cards/gacha-stats").await
    }

    // MARK: - ai.log service methods

    /// Create a new blog post
    pub async fn create_blog_post<T: Serialize>(&self, params: &T) -> Result<Value> {
        self.call_service_method("ai.log", "api/v1/posts", params).await
    }

    /// Get list of blog posts
    pub async fn get_blog_posts(&self) -> Result<Value> {
        self.call_service_get("ai.log", "api/v1/posts").await
    }

    /// Build the blog
    pub async fn build_blog(&self) -> Result<Value> {
        self.call_service_method("ai.log", "api/v1/build", &serde_json::json!({})).await
    }

    /// Translate document using ai.log service
    pub async fn translate_document<T: Serialize>(&self, params: &T) -> Result<Value> {
        self.call_service_method("ai.log", "api/v1/translate", params).await
    }

    /// Generate documentation using ai.log service
    pub async fn generate_docs<T: Serialize>(&self, params: &T) -> Result<Value> {
        self.call_service_method("ai.log", "api/v1/docs", params).await
    }
}

/// Service status enum
#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Available,
    Unavailable(String),
    Error(String),
}

impl ServiceStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, ServiceStatus::Available)
    }
}

/// Service detector for ai ecosystem services
pub struct ServiceDetector {
    client: ServiceClient,
}

impl ServiceDetector {
    pub fn new() -> Self {
        Self {
            client: ServiceClient::new(),
        }
    }

    /// Check all ai ecosystem services
    pub async fn detect_services(&self) -> ServiceMap {
        let mut services = ServiceMap::default();

        // Check ai.card service
        if let Ok(status) = self.client.check_service_status("http://localhost:8000").await {
            services.ai_card = Some(ServiceInfo {
                base_url: "http://localhost:8000".to_string(),
                status,
            });
        }

        // Check ai.log service  
        if let Ok(status) = self.client.check_service_status("http://localhost:8001").await {
            services.ai_log = Some(ServiceInfo {
                base_url: "http://localhost:8001".to_string(),
                status,
            });
        }

        // Check ai.bot service
        if let Ok(status) = self.client.check_service_status("http://localhost:8002").await {
            services.ai_bot = Some(ServiceInfo {
                base_url: "http://localhost:8002".to_string(),
                status,
            });
        }

        services
    }

    /// Get available services only
    pub async fn get_available_services(&self) -> Vec<String> {
        let services = self.detect_services().await;
        let mut available = Vec::new();

        if let Some(card) = &services.ai_card {
            if card.status.is_available() {
                available.push("ai.card".to_string());
            }
        }

        if let Some(log) = &services.ai_log {
            if log.status.is_available() {
                available.push("ai.log".to_string());
            }
        }

        if let Some(bot) = &services.ai_bot {
            if bot.status.is_available() {
                available.push("ai.bot".to_string());
            }
        }

        available
    }

    /// Get card collection statistics
    pub async fn get_card_stats(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match self.client.get_request("http://localhost:8000/api/v1/cards/gacha-stats").await {
            Ok(stats) => Ok(stats),
            Err(e) => Err(e.into()),
        }
    }

    /// Draw a card for user
    pub async fn draw_card(&self, user_did: &str, is_paid: bool) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "user_did": user_did,
            "is_paid": is_paid
        });

        match self.client.post_request("http://localhost:8000/api/v1/cards/draw", &payload).await {
            Ok(card) => Ok(card),
            Err(e) => Err(e.into()),
        }
    }

    /// Get user's card collection
    pub async fn get_user_cards(&self, user_did: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let url = format!("http://localhost:8000/api/v1/cards/collection?did={}", user_did);
        match self.client.get_request(&url).await {
            Ok(collection) => Ok(collection),
            Err(e) => Err(e.into()),
        }
    }

    /// Get contextual memories for conversation mode
    pub async fn get_contextual_memories(&self, _user_id: &str, _limit: usize) -> Result<Vec<crate::memory::Memory>, Box<dyn std::error::Error>> {
        // This is a simplified version - in a real implementation this would call the MCP server
        // For now, we'll return an empty vec to make compilation work
        Ok(Vec::new())
    }

    /// Search memories by query
    pub async fn search_memories(&self, _query: &str, _limit: usize) -> Result<Vec<crate::memory::Memory>, Box<dyn std::error::Error>> {
        // This is a simplified version - in a real implementation this would call the MCP server
        // For now, we'll return an empty vec to make compilation work
        Ok(Vec::new())
    }

    /// Create context summary
    pub async fn create_summary(&self, user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        // This is a simplified version - in a real implementation this would call the MCP server
        // For now, we'll return a placeholder summary
        Ok(format!("Context summary for user: {}", user_id))
    }
}

/// Service information
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub base_url: String,
    pub status: ServiceStatus,
}

/// Map of all ai ecosystem services
#[derive(Debug, Clone, Default)]
pub struct ServiceMap {
    pub ai_card: Option<ServiceInfo>,
    pub ai_log: Option<ServiceInfo>,
    pub ai_bot: Option<ServiceInfo>,
}

impl ServiceMap {
    /// Get service info by name
    pub fn get_service(&self, name: &str) -> Option<&ServiceInfo> {
        match name {
            "ai.card" => self.ai_card.as_ref(),
            "ai.log" => self.ai_log.as_ref(),
            "ai.bot" => self.ai_bot.as_ref(),
            _ => None,
        }
    }

    /// Check if a service is available
    pub fn is_service_available(&self, name: &str) -> bool {
        self.get_service(name)
            .map(|info| info.status.is_available())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_client_creation() {
        let _client = ServiceClient::new();
        // Basic test to ensure client can be created
        assert!(true);
    }

    #[test]
    fn test_service_status() {
        let status = ServiceStatus::Available;
        assert!(status.is_available());

        let status = ServiceStatus::Unavailable("Connection refused".to_string());
        assert!(!status.is_available());
    }

    #[test]
    fn test_service_map() {
        let mut map = ServiceMap::default();
        assert!(!map.is_service_available("ai.card"));

        map.ai_card = Some(ServiceInfo {
            base_url: "http://localhost:8000".to_string(),
            status: ServiceStatus::Available,
        });

        assert!(map.is_service_available("ai.card"));
        assert!(!map.is_service_available("ai.log"));
    }
}