use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub user_id: String,
    pub score: f64,
    pub threshold: f64,
    pub status: RelationshipStatus,
    pub total_interactions: u32,
    pub positive_interactions: u32,
    pub negative_interactions: u32,
    pub transmission_enabled: bool,
    pub is_broken: bool,
    pub last_interaction: Option<DateTime<Utc>>,
    pub last_transmission: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub daily_interaction_count: u32,
    pub last_daily_reset: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipStatus {
    New,
    Acquaintance,
    Friend,
    CloseFriend,
    Broken,
}

impl std::fmt::Display for RelationshipStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationshipStatus::New => write!(f, "new"),
            RelationshipStatus::Acquaintance => write!(f, "acquaintance"),
            RelationshipStatus::Friend => write!(f, "friend"),
            RelationshipStatus::CloseFriend => write!(f, "close_friend"),
            RelationshipStatus::Broken => write!(f, "broken"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipTracker {
    relationships: HashMap<String, Relationship>,
    config: Config,
}

impl RelationshipTracker {
    pub fn new(config: &Config) -> Result<Self> {
        let relationships = Self::load_relationships(config)?;
        
        Ok(RelationshipTracker {
            relationships,
            config: config.clone(),
        })
    }
    
    pub fn get_or_create_relationship(&mut self, user_id: &str) -> &mut Relationship {
        let now = Utc::now();
        
        self.relationships.entry(user_id.to_string()).or_insert_with(|| {
            Relationship {
                user_id: user_id.to_string(),
                score: 0.0,
                threshold: 10.0, // Default threshold for transmission
                status: RelationshipStatus::New,
                total_interactions: 0,
                positive_interactions: 0,
                negative_interactions: 0,
                transmission_enabled: false,
                is_broken: false,
                last_interaction: None,
                last_transmission: None,
                created_at: now,
                daily_interaction_count: 0,
                last_daily_reset: now,
            }
        })
    }
    
    pub fn process_interaction(&mut self, user_id: &str, sentiment: f64) -> Result<f64> {
        let now = Utc::now();
        let score_change;
        
        // Create relationship if it doesn't exist
        {
            let relationship = self.get_or_create_relationship(user_id);
            
            // Reset daily count if needed
            if (now - relationship.last_daily_reset).num_days() >= 1 {
                relationship.daily_interaction_count = 0;
                relationship.last_daily_reset = now;
            }
            
            // Apply daily interaction limit
            if relationship.daily_interaction_count >= 10 {
                return Ok(0.0); // No score change due to daily limit
            }
            
            // Store previous score for potential future logging
            
            // Calculate score change based on sentiment
            let mut base_score_change = sentiment * 0.5; // Base change
            
            // Apply diminishing returns for high interaction counts
            let interaction_factor = 1.0 / (1.0 + relationship.total_interactions as f64 * 0.01);
            base_score_change *= interaction_factor;
            score_change = base_score_change;
            
            // Update relationship data
            relationship.score += score_change;
            relationship.score = relationship.score.max(-50.0).min(100.0); // Clamp score
            relationship.total_interactions += 1;
            relationship.daily_interaction_count += 1;
            relationship.last_interaction = Some(now);
            
            if sentiment > 0.0 {
                relationship.positive_interactions += 1;
            } else if sentiment < 0.0 {
                relationship.negative_interactions += 1;
            }
            
            // Check for relationship breaking
            if relationship.score <= -20.0 && !relationship.is_broken {
                relationship.is_broken = true;
                relationship.transmission_enabled = false;
                relationship.status = RelationshipStatus::Broken;
            }
            
            // Enable transmission if threshold is reached
            if relationship.score >= relationship.threshold && !relationship.is_broken {
                relationship.transmission_enabled = true;
            }
        }
        
        // Update status based on score (separate borrow)
        self.update_relationship_status(user_id);
        
        self.save_relationships()?;
        
        Ok(score_change)
    }
    
    fn update_relationship_status(&mut self, user_id: &str) {
        if let Some(relationship) = self.relationships.get_mut(user_id) {
            if relationship.is_broken {
                return; // Broken relationships cannot change status
            }
            
            relationship.status = match relationship.score {
                score if score >= 50.0 => RelationshipStatus::CloseFriend,
                score if score >= 20.0 => RelationshipStatus::Friend,
                score if score >= 5.0 => RelationshipStatus::Acquaintance,
                _ => RelationshipStatus::New,
            };
        }
    }
    
    pub fn apply_time_decay(&mut self) -> Result<()> {
        let now = Utc::now();
        let decay_rate = 0.1; // 10% decay per day
        
        for relationship in self.relationships.values_mut() {
            if let Some(last_interaction) = relationship.last_interaction {
                let days_since_interaction = (now - last_interaction).num_days() as f64;
                
                if days_since_interaction > 0.0 {
                    let decay_factor = (1.0_f64 - decay_rate).powf(days_since_interaction);
                    relationship.score *= decay_factor;
                    
                    // Update status after decay
                    if relationship.score < relationship.threshold {
                        relationship.transmission_enabled = false;
                    }
                }
            }
        }
        
        // Update statuses for all relationships
        let user_ids: Vec<String> = self.relationships.keys().cloned().collect();
        for user_id in user_ids {
            self.update_relationship_status(&user_id);
        }
        
        self.save_relationships()?;
        Ok(())
    }
    
    pub fn get_relationship(&self, user_id: &str) -> Option<&Relationship> {
        self.relationships.get(user_id)
    }
    
    pub fn list_all_relationships(&self) -> &HashMap<String, Relationship> {
        &self.relationships
    }
    
    pub fn get_transmission_eligible(&self) -> HashMap<String, &Relationship> {
        self.relationships
            .iter()
            .filter(|(_, rel)| rel.transmission_enabled && !rel.is_broken)
            .map(|(id, rel)| (id.clone(), rel))
            .collect()
    }
    
    pub fn record_transmission(&mut self, user_id: &str) -> Result<()> {
        if let Some(relationship) = self.relationships.get_mut(user_id) {
            relationship.last_transmission = Some(Utc::now());
            self.save_relationships()?;
        }
        Ok(())
    }
    
    pub fn get_relationship_stats(&self) -> RelationshipStats {
        let total_relationships = self.relationships.len();
        let active_relationships = self.relationships
            .values()
            .filter(|r| r.total_interactions > 0)
            .count();
        let transmission_enabled = self.relationships
            .values()
            .filter(|r| r.transmission_enabled)
            .count();
        let broken_relationships = self.relationships
            .values()
            .filter(|r| r.is_broken)
            .count();
        
        let avg_score = if total_relationships > 0 {
            self.relationships.values().map(|r| r.score).sum::<f64>() / total_relationships as f64
        } else {
            0.0
        };
        
        RelationshipStats {
            total_relationships,
            active_relationships,
            transmission_enabled,
            broken_relationships,
            avg_score,
        }
    }
    
    fn load_relationships(config: &Config) -> Result<HashMap<String, Relationship>> {
        let file_path = config.relationships_file();
        if !file_path.exists() {
            return Ok(HashMap::new());
        }
        
        let content = std::fs::read_to_string(file_path)
            .context("Failed to read relationships file")?;
        
        let relationships: HashMap<String, Relationship> = serde_json::from_str(&content)
            .context("Failed to parse relationships file")?;
        
        Ok(relationships)
    }
    
    fn save_relationships(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.relationships)
            .context("Failed to serialize relationships")?;
        
        std::fs::write(&self.config.relationships_file(), content)
            .context("Failed to write relationships file")?;
        
        Ok(())
    }
    
    pub fn get_all_relationships(&self) -> Result<HashMap<String, RelationshipCompact>> {
        let mut result = HashMap::new();
        
        for (user_id, relationship) in &self.relationships {
            result.insert(user_id.clone(), RelationshipCompact {
                score: relationship.score,
                trust_level: relationship.score / 10.0, // Simplified trust calculation
                interaction_count: relationship.total_interactions,
                last_interaction: relationship.last_interaction.unwrap_or(relationship.created_at),
                status: relationship.status.clone(),
            });
        }
        
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RelationshipStats {
    pub total_relationships: usize,
    pub active_relationships: usize,
    pub transmission_enabled: usize,
    pub broken_relationships: usize,
    pub avg_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelationshipCompact {
    pub score: f64,
    pub trust_level: f64,
    pub interaction_count: u32,
    pub last_interaction: DateTime<Utc>,
    pub status: RelationshipStatus,
}