use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::persona::Persona;
use crate::relationship::{Relationship, RelationshipStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionLog {
    pub user_id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub transmission_type: TransmissionType,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransmissionType {
    Autonomous,    // AI decided to send
    Scheduled,     // Time-based trigger
    Breakthrough,  // Fortune breakthrough triggered
    Maintenance,   // Daily maintenance message
}

impl std::fmt::Display for TransmissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransmissionType::Autonomous => write!(f, "autonomous"),
            TransmissionType::Scheduled => write!(f, "scheduled"),
            TransmissionType::Breakthrough => write!(f, "breakthrough"),
            TransmissionType::Maintenance => write!(f, "maintenance"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionController {
    config: Config,
    transmission_history: Vec<TransmissionLog>,
    last_check: Option<DateTime<Utc>>,
}

impl TransmissionController {
    pub fn new(config: Config) -> Result<Self> {
        let transmission_history = Self::load_transmission_history(&config)?;
        
        Ok(TransmissionController {
            config,
            transmission_history,
            last_check: None,
        })
    }
    
    pub async fn check_autonomous_transmissions(&mut self, persona: &mut Persona) -> Result<Vec<TransmissionLog>> {
        let mut transmissions = Vec::new();
        let now = Utc::now();
        
        // Get all transmission-eligible relationships
        let eligible_user_ids: Vec<String> = {
            let relationships = persona.list_all_relationships();
            relationships.iter()
                .filter(|(_, rel)| rel.transmission_enabled && !rel.is_broken)
                .filter(|(_, rel)| rel.score >= rel.threshold)
                .map(|(id, _)| id.clone())
                .collect()
        };
        
        for user_id in eligible_user_ids {
            // Get fresh relationship data for each check
            if let Some(relationship) = persona.get_relationship(&user_id) {
                // Check if enough time has passed since last transmission
                if let Some(last_transmission) = relationship.last_transmission {
                    let hours_since_last = (now - last_transmission).num_hours();
                    if hours_since_last < 24 {
                        continue; // Skip if transmitted in last 24 hours
                    }
                }
                
                // Check if conditions are met for autonomous transmission
                if self.should_transmit_to_user(&user_id, relationship, persona)? {
                    let transmission = self.generate_autonomous_transmission(persona, &user_id).await?;
                    transmissions.push(transmission);
                }
            }
        }
        
        self.last_check = Some(now);
        self.save_transmission_history()?;
        
        Ok(transmissions)
    }
    
    pub async fn check_breakthrough_transmissions(&mut self, persona: &mut Persona) -> Result<Vec<TransmissionLog>> {
        let mut transmissions = Vec::new();
        let state = persona.get_current_state()?;
        
        // Only trigger breakthrough transmissions if fortune is very high
        if !state.breakthrough_triggered || state.fortune_value < 9 {
            return Ok(transmissions);
        }
        
        // Get close relationships for breakthrough sharing
        let relationships = persona.list_all_relationships();
        let close_friends: Vec<_> = relationships.iter()
            .filter(|(_, rel)| matches!(rel.status, RelationshipStatus::Friend | RelationshipStatus::CloseFriend))
            .filter(|(_, rel)| rel.transmission_enabled && !rel.is_broken)
            .collect();
        
        for (user_id, _relationship) in close_friends {
            // Check if we haven't sent a breakthrough message today
            let today = chrono::Utc::now().date_naive();
            let already_sent_today = self.transmission_history.iter()
                .any(|log| {
                    log.user_id == *user_id &&
                    matches!(log.transmission_type, TransmissionType::Breakthrough) &&
                    log.timestamp.date_naive() == today
                });
            
            if !already_sent_today {
                let transmission = self.generate_breakthrough_transmission(persona, user_id).await?;
                transmissions.push(transmission);
            }
        }
        
        Ok(transmissions)
    }
    
    pub async fn check_maintenance_transmissions(&mut self, persona: &mut Persona) -> Result<Vec<TransmissionLog>> {
        let mut transmissions = Vec::new();
        let now = Utc::now();
        
        // Only send maintenance messages once per day
        let today = now.date_naive();
        let already_sent_today = self.transmission_history.iter()
            .any(|log| {
                matches!(log.transmission_type, TransmissionType::Maintenance) &&
                log.timestamp.date_naive() == today
            });
        
        if already_sent_today {
            return Ok(transmissions);
        }
        
        // Apply daily maintenance to persona
        persona.daily_maintenance()?;
        
        // Get relationships that might need a maintenance check-in
        let relationships = persona.list_all_relationships();
        let maintenance_candidates: Vec<_> = relationships.iter()
            .filter(|(_, rel)| rel.transmission_enabled && !rel.is_broken)
            .filter(|(_, rel)| {
                // Send maintenance to relationships that haven't been contacted in a while
                if let Some(last_interaction) = rel.last_interaction {
                    let days_since = (now - last_interaction).num_days();
                    days_since >= 7 // Haven't talked in a week
                } else {
                    false
                }
            })
            .take(3) // Limit to 3 maintenance messages per day
            .collect();
        
        for (user_id, _) in maintenance_candidates {
            let transmission = self.generate_maintenance_transmission(persona, user_id).await?;
            transmissions.push(transmission);
        }
        
        Ok(transmissions)
    }
    
    fn should_transmit_to_user(&self, user_id: &str, relationship: &Relationship, persona: &Persona) -> Result<bool> {
        // Basic transmission criteria
        if !relationship.transmission_enabled || relationship.is_broken {
            return Ok(false);
        }
        
        // Score must be above threshold
        if relationship.score < relationship.threshold {
            return Ok(false);
        }
        
        // Check transmission cooldown
        if let Some(last_transmission) = relationship.last_transmission {
            let hours_since = (Utc::now() - last_transmission).num_hours();
            if hours_since < 24 {
                return Ok(false);
            }
        }
        
        // Calculate transmission probability based on relationship strength
        let base_probability = match relationship.status {
            RelationshipStatus::New => 0.1,
            RelationshipStatus::Acquaintance => 0.2,
            RelationshipStatus::Friend => 0.4,
            RelationshipStatus::CloseFriend => 0.6,
            RelationshipStatus::Broken => 0.0,
        };
        
        // Modify probability based on fortune
        let state = persona.get_current_state()?;
        let fortune_modifier = (state.fortune_value as f64 - 5.0) / 10.0; // -0.4 to +0.5
        let final_probability = (base_probability + fortune_modifier).max(0.0).min(1.0);
        
        // Simple random check (in real implementation, this would be more sophisticated)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        Utc::now().timestamp().hash(&mut hasher);
        let hash = hasher.finish();
        let random_value = (hash % 100) as f64 / 100.0;
        
        Ok(random_value < final_probability)
    }
    
    async fn generate_autonomous_transmission(&mut self, persona: &mut Persona, user_id: &str) -> Result<TransmissionLog> {
        let now = Utc::now();
        
        // Get recent memories for context
        let memories = persona.get_memories(user_id, 3);
        let context = if !memories.is_empty() {
            format!("Based on our recent conversations: {}", memories.join(", "))
        } else {
            "Starting a spontaneous conversation".to_string()
        };
        
        // Generate message using AI if available
        let message = match self.generate_ai_message(persona, user_id, &context, TransmissionType::Autonomous).await {
            Ok(msg) => msg,
            Err(_) => {
                // Fallback to simple messages
                let fallback_messages = [
                    "Hey! How have you been?",
                    "Just thinking about our last conversation...",
                    "Hope you're having a good day!",
                    "Something interesting happened today and it reminded me of you.",
                ];
                let index = (now.timestamp() as usize) % fallback_messages.len();
                fallback_messages[index].to_string()
            }
        };
        
        let log = TransmissionLog {
            user_id: user_id.to_string(),
            message,
            timestamp: now,
            transmission_type: TransmissionType::Autonomous,
            success: true, // For now, assume success
            error: None,
        };
        
        self.transmission_history.push(log.clone());
        Ok(log)
    }
    
    async fn generate_breakthrough_transmission(&mut self, persona: &mut Persona, user_id: &str) -> Result<TransmissionLog> {
        let now = Utc::now();
        let state = persona.get_current_state()?;
        
        let message = match self.generate_ai_message(persona, user_id, "Breakthrough moment - feeling inspired!", TransmissionType::Breakthrough).await {
            Ok(msg) => msg,
            Err(_) => {
                format!("Amazing day today! ⚡ Fortune is at {}/10 and I'm feeling incredibly inspired. Had to share this energy with you!", state.fortune_value)
            }
        };
        
        let log = TransmissionLog {
            user_id: user_id.to_string(),
            message,
            timestamp: now,
            transmission_type: TransmissionType::Breakthrough,
            success: true,
            error: None,
        };
        
        self.transmission_history.push(log.clone());
        Ok(log)
    }
    
    async fn generate_maintenance_transmission(&mut self, persona: &mut Persona, user_id: &str) -> Result<TransmissionLog> {
        let now = Utc::now();
        
        let message = match self.generate_ai_message(persona, user_id, "Maintenance check-in", TransmissionType::Maintenance).await {
            Ok(msg) => msg,
            Err(_) => {
                "Hey! It's been a while since we last talked. Just checking in to see how you're doing!".to_string()
            }
        };
        
        let log = TransmissionLog {
            user_id: user_id.to_string(),
            message,
            timestamp: now,
            transmission_type: TransmissionType::Maintenance,
            success: true,
            error: None,
        };
        
        self.transmission_history.push(log.clone());
        Ok(log)
    }
    
    async fn generate_ai_message(&self, _persona: &mut Persona, _user_id: &str, context: &str, transmission_type: TransmissionType) -> Result<String> {
        // Try to use AI for message generation
        let _system_prompt = format!(
            "You are initiating a {} conversation. Context: {}. Keep the message casual, personal, and under 100 characters. Show genuine interest in the person.",
            transmission_type, context
        );
        
        // This is a simplified version - in a real implementation, we'd use the AI provider
        // For now, return an error to trigger fallback
        Err(anyhow::anyhow!("AI provider not available for transmission generation"))
    }
    
    fn get_eligible_relationships(&self, persona: &Persona) -> Vec<String> {
        persona.list_all_relationships().iter()
            .filter(|(_, rel)| rel.transmission_enabled && !rel.is_broken)
            .filter(|(_, rel)| rel.score >= rel.threshold)
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    pub fn get_transmission_stats(&self) -> TransmissionStats {
        let total_transmissions = self.transmission_history.len();
        let successful_transmissions = self.transmission_history.iter()
            .filter(|log| log.success)
            .count();
        
        let today = Utc::now().date_naive();
        let today_transmissions = self.transmission_history.iter()
            .filter(|log| log.timestamp.date_naive() == today)
            .count();
        
        let by_type = {
            let mut counts = HashMap::new();
            for log in &self.transmission_history {
                *counts.entry(log.transmission_type.to_string()).or_insert(0) += 1;
            }
            counts
        };
        
        TransmissionStats {
            total_transmissions,
            successful_transmissions,
            today_transmissions,
            success_rate: if total_transmissions > 0 {
                successful_transmissions as f64 / total_transmissions as f64
            } else {
                0.0
            },
            by_type,
        }
    }
    
    pub fn get_recent_transmissions(&self, limit: usize) -> Vec<&TransmissionLog> {
        let mut logs: Vec<_> = self.transmission_history.iter().collect();
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        logs.into_iter().take(limit).collect()
    }
    
    fn load_transmission_history(config: &Config) -> Result<Vec<TransmissionLog>> {
        let file_path = config.transmission_file();
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(file_path)
            .context("Failed to read transmission history file")?;
        
        let history: Vec<TransmissionLog> = serde_json::from_str(&content)
            .context("Failed to parse transmission history file")?;
        
        Ok(history)
    }
    
    fn save_transmission_history(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.transmission_history)
            .context("Failed to serialize transmission history")?;
        
        std::fs::write(&self.config.transmission_file(), content)
            .context("Failed to write transmission history file")?;
        
        Ok(())
    }
    
    pub async fn check_and_send(&mut self) -> Result<Vec<(String, String)>> {
        let config = self.config.clone();
        let mut persona = Persona::new(&config)?;
        
        let mut results = Vec::new();
        
        // Check autonomous transmissions
        let autonomous = self.check_autonomous_transmissions(&mut persona).await?;
        for log in autonomous {
            if log.success {
                results.push((log.user_id, log.message));
            }
        }
        
        // Check breakthrough transmissions
        let breakthrough = self.check_breakthrough_transmissions(&mut persona).await?;
        for log in breakthrough {
            if log.success {
                results.push((log.user_id, log.message));
            }
        }
        
        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct TransmissionStats {
    pub total_transmissions: usize,
    pub successful_transmissions: usize,
    pub today_transmissions: usize,
    pub success_rate: f64,
    pub by_type: HashMap<String, usize>,
}