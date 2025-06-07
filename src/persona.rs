use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::config::Config;
use crate::memory::{MemoryManager, MemoryStats, Memory};
use crate::relationship::{RelationshipTracker, Relationship as RelationshipData, RelationshipStats};
use crate::ai_provider::{AIProviderClient, ChatMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    config: Config,
    #[serde(skip)]
    memory_manager: Option<MemoryManager>,
    #[serde(skip)]
    relationship_tracker: Option<RelationshipTracker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaState {
    pub current_mood: String,
    pub fortune_value: i32,
    pub breakthrough_triggered: bool,
    pub base_personality: HashMap<String, f64>,
}


impl Persona {
    pub fn new(config: &Config) -> Result<Self> {
        let memory_manager = MemoryManager::new(config)?;
        let relationship_tracker = RelationshipTracker::new(config)?;
        
        Ok(Persona {
            config: config.clone(),
            memory_manager: Some(memory_manager),
            relationship_tracker: Some(relationship_tracker),
        })
    }
    
    pub fn get_current_state(&self) -> Result<PersonaState> {
        // Load fortune
        let fortune_value = self.load_today_fortune()?;
        
        // Create base personality
        let mut base_personality = HashMap::new();
        base_personality.insert("curiosity".to_string(), 0.7);
        base_personality.insert("empathy".to_string(), 0.8);
        base_personality.insert("creativity".to_string(), 0.6);
        base_personality.insert("analytical".to_string(), 0.9);
        base_personality.insert("emotional".to_string(), 0.4);
        
        // Determine mood based on fortune
        let current_mood = match fortune_value {
            1..=3 => "Contemplative",
            4..=6 => "Neutral", 
            7..=8 => "Optimistic",
            9..=10 => "Energetic",
            _ => "Unknown",
        };
        
        Ok(PersonaState {
            current_mood: current_mood.to_string(),
            fortune_value,
            breakthrough_triggered: fortune_value >= 9,
            base_personality,
        })
    }
    
    pub fn get_relationship(&self, user_id: &str) -> Option<&RelationshipData> {
        self.relationship_tracker.as_ref()
            .and_then(|tracker| tracker.get_relationship(user_id))
    }
    
    pub fn process_interaction(&mut self, user_id: &str, message: &str) -> Result<(String, f64)> {
        // Add memory
        if let Some(memory_manager) = &mut self.memory_manager {
            memory_manager.add_memory(user_id, message, 0.5)?;
        }
        
        // Calculate sentiment (simple keyword-based for now)
        let sentiment = self.calculate_sentiment(message);
        
        // Update relationship
        let relationship_delta = if let Some(relationship_tracker) = &mut self.relationship_tracker {
            relationship_tracker.process_interaction(user_id, sentiment)?
        } else {
            0.0
        };
        
        // Generate response (simple for now)
        let response = format!("I understand your message: '{}'", message);
        
        Ok((response, relationship_delta))
    }
    
    pub async fn process_ai_interaction(&mut self, user_id: &str, message: &str, provider: Option<String>, model: Option<String>) -> Result<(String, f64)> {
        // Add memory for user message
        if let Some(memory_manager) = &mut self.memory_manager {
            memory_manager.add_memory(user_id, message, 0.5)?;
        }
        
        // Calculate sentiment
        let sentiment = self.calculate_sentiment(message);
        
        // Update relationship
        let relationship_delta = if let Some(relationship_tracker) = &mut self.relationship_tracker {
            relationship_tracker.process_interaction(user_id, sentiment)?
        } else {
            0.0
        };
        
        // Generate AI response
        let ai_config = self.config.get_ai_config(provider, model)?;
        let ai_client = AIProviderClient::new(ai_config);
        
        // Build conversation context
        let mut messages = Vec::new();
        
        // Get recent memories for context
        if let Some(memory_manager) = &mut self.memory_manager {
            let recent_memories = memory_manager.get_memories(user_id, 5);
            if !recent_memories.is_empty() {
                let context = recent_memories.iter()
                    .map(|m| m.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                messages.push(ChatMessage::system(format!("Previous conversation context:\n{}", context)));
            }
        }
        
        // Add current message
        messages.push(ChatMessage::user(message));
        
        // Generate system prompt based on personality and relationship
        let system_prompt = self.generate_system_prompt(user_id);
        
        // Get AI response
        let response = match ai_client.chat(messages, Some(system_prompt)).await {
            Ok(chat_response) => chat_response.content,
            Err(_) => {
                // Fallback to simple response if AI fails
                format!("I understand your message: '{}'", message)
            }
        };
        
        // Store AI response in memory
        if let Some(memory_manager) = &mut self.memory_manager {
            memory_manager.add_memory(user_id, &format!("AI: {}", response), 0.3)?;
        }
        
        Ok((response, relationship_delta))
    }
    
    fn generate_system_prompt(&self, user_id: &str) -> String {
        let mut prompt = String::from("You are a helpful AI assistant with a unique personality. ");
        
        // Add personality based on current state
        if let Ok(state) = self.get_current_state() {
            prompt.push_str(&format!("Your current mood is {}. ", state.current_mood));
            
            if state.breakthrough_triggered {
                prompt.push_str("You are feeling particularly inspired today! ");
            }
            
            // Add personality traits
            let mut traits = Vec::new();
            for (trait_name, value) in &state.base_personality {
                if *value > 0.7 {
                    traits.push(trait_name.clone());
                }
            }
            
            if !traits.is_empty() {
                prompt.push_str(&format!("Your dominant traits are: {}. ", traits.join(", ")));
            }
        }
        
        // Add relationship context
        if let Some(relationship) = self.get_relationship(user_id) {
            match relationship.status.to_string().as_str() {
                "new" => prompt.push_str("This is a new relationship, be welcoming but cautious. "),
                "friend" => prompt.push_str("You have a friendly relationship with this user. "),
                "close_friend" => prompt.push_str("This is a close friend, be warm and personal. "),
                "broken" => prompt.push_str("This relationship is strained, be formal and distant. "),
                _ => {}
            }
        }
        
        prompt.push_str("Keep responses concise and natural. Avoid being overly formal or robotic.");
        
        prompt
    }
    
    fn calculate_sentiment(&self, message: &str) -> f64 {
        // Simple sentiment analysis based on keywords
        let positive_words = ["good", "great", "awesome", "love", "like", "happy", "thank"];
        let negative_words = ["bad", "hate", "awful", "terrible", "angry", "sad"];
        
        let message_lower = message.to_lowercase();
        let positive_count = positive_words.iter()
            .filter(|word| message_lower.contains(*word))
            .count() as f64;
        let negative_count = negative_words.iter()
            .filter(|word| message_lower.contains(*word))
            .count() as f64;
        
        (positive_count - negative_count).max(-1.0).min(1.0)
    }
    
    pub fn get_memories(&mut self, user_id: &str, limit: usize) -> Vec<String> {
        if let Some(memory_manager) = &mut self.memory_manager {
            memory_manager.get_memories(user_id, limit)
                .into_iter()
                .map(|m| m.content.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn search_memories(&self, user_id: &str, keywords: &[String]) -> Vec<String> {
        if let Some(memory_manager) = &self.memory_manager {
            memory_manager.search_memories(user_id, keywords)
                .into_iter()
                .map(|m| m.content.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn get_memory_stats(&self, user_id: &str) -> Option<MemoryStats> {
        self.memory_manager.as_ref()
            .map(|manager| manager.get_memory_stats(user_id))
    }
    
    pub fn get_relationship_stats(&self) -> Option<RelationshipStats> {
        self.relationship_tracker.as_ref()
            .map(|tracker| tracker.get_relationship_stats())
    }
    
    pub fn add_memory(&mut self, memory: Memory) -> Result<()> {
        if let Some(memory_manager) = &mut self.memory_manager {
            memory_manager.add_memory(&memory.user_id, &memory.content, memory.importance)?;
        }
        Ok(())
    }
    
    pub fn update_relationship(&mut self, user_id: &str, delta: f64) -> Result<()> {
        if let Some(relationship_tracker) = &mut self.relationship_tracker {
            relationship_tracker.process_interaction(user_id, delta)?;
        }
        Ok(())
    }
    
    pub fn daily_maintenance(&mut self) -> Result<()> {
        // Apply time decay to relationships
        if let Some(relationship_tracker) = &mut self.relationship_tracker {
            relationship_tracker.apply_time_decay()?;
        }
        
        Ok(())
    }
    
    fn load_today_fortune(&self) -> Result<i32> {
        // Try to load existing fortune for today
        if let Ok(content) = std::fs::read_to_string(self.config.fortune_file()) {
            if let Ok(fortune_data) = serde_json::from_str::<serde_json::Value>(&content) {
                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                if let Some(fortune) = fortune_data.get(&today) {
                    if let Some(value) = fortune.as_i64() {
                        return Ok(value as i32);
                    }
                }
            }
        }
        
        // Generate new fortune for today (1-10)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let mut hasher = DefaultHasher::new();
        today.hash(&mut hasher);
        let hash = hasher.finish();
        
        let fortune = (hash % 10) as i32 + 1;
        
        // Save fortune
        let mut fortune_data = if let Ok(content) = std::fs::read_to_string(self.config.fortune_file()) {
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        
        fortune_data[today] = serde_json::json!(fortune);
        
        if let Ok(content) = serde_json::to_string_pretty(&fortune_data) {
            let _ = std::fs::write(self.config.fortune_file(), content);
        }
        
        Ok(fortune)
    }
    
    pub fn list_all_relationships(&self) -> HashMap<String, RelationshipData> {
        if let Some(tracker) = &self.relationship_tracker {
            tracker.list_all_relationships().clone()
        } else {
            HashMap::new()
        }
    }
    
    pub async fn process_message(&mut self, user_id: &str, message: &str) -> Result<ChatMessage> {
        let (_response, _delta) = self.process_ai_interaction(user_id, message, None, None).await?;
        Ok(ChatMessage::assistant(&_response))
    }
    
    pub fn get_fortune(&self) -> Result<i32> {
        self.load_today_fortune()
    }
    
    pub fn generate_new_fortune(&self) -> Result<i32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let mut hasher = DefaultHasher::new();
        today.hash(&mut hasher);
        let hash = hasher.finish();
        
        let fortune = (hash % 10) as i32 + 1;
        
        // Save fortune
        let mut fortune_data = if let Ok(content) = std::fs::read_to_string(self.config.fortune_file()) {
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        
        fortune_data[today] = serde_json::json!(fortune);
        
        if let Ok(content) = serde_json::to_string_pretty(&fortune_data) {
            let _ = std::fs::write(self.config.fortune_file(), content);
        }
        
        Ok(fortune)
    }
}