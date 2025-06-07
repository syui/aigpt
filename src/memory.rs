use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub summary: Option<String>,
    pub importance: f64,
    pub memory_type: MemoryType,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Interaction,
    Summary,
    Core,
    Forgotten,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManager {
    memories: HashMap<String, Memory>,
    config: Config,
}

impl MemoryManager {
    pub fn new(config: &Config) -> Result<Self> {
        let memories = Self::load_memories(config)?;
        
        Ok(MemoryManager {
            memories,
            config: config.clone(),
        })
    }
    
    pub fn add_memory(&mut self, user_id: &str, content: &str, importance: f64) -> Result<String> {
        let memory_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let memory = Memory {
            id: memory_id.clone(),
            user_id: user_id.to_string(),
            content: content.to_string(),
            summary: None,
            importance,
            memory_type: MemoryType::Interaction,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        };
        
        self.memories.insert(memory_id.clone(), memory);
        self.save_memories()?;
        
        Ok(memory_id)
    }
    
    pub fn get_memories(&mut self, user_id: &str, limit: usize) -> Vec<&Memory> {
        // Get immutable references for sorting
        let mut user_memory_ids: Vec<_> = self.memories
            .iter()
            .filter(|(_, m)| m.user_id == user_id)
            .map(|(id, memory)| {
                let score = memory.importance * 0.7 + (1.0 / ((Utc::now() - memory.created_at).num_hours() as f64 + 1.0)) * 0.3;
                (id.clone(), score)
            })
            .collect();
        
        // Sort by score
        user_memory_ids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Update access information and collect references
        let now = Utc::now();
        let mut result: Vec<&Memory> = Vec::new();
        
        for (memory_id, _) in user_memory_ids.into_iter().take(limit) {
            if let Some(memory) = self.memories.get_mut(&memory_id) {
                memory.last_accessed = now;
                memory.access_count += 1;
                // We can't return mutable references here, so we'll need to adjust the return type
            }
        }
        
        // Return immutable references
        self.memories
            .values()
            .filter(|m| m.user_id == user_id)
            .take(limit)
            .collect()
    }
    
    pub fn search_memories(&self, user_id: &str, keywords: &[String]) -> Vec<&Memory> {
        self.memories
            .values()
            .filter(|m| {
                m.user_id == user_id && 
                keywords.iter().any(|keyword| {
                    m.content.to_lowercase().contains(&keyword.to_lowercase()) ||
                    m.summary.as_ref().map_or(false, |s| s.to_lowercase().contains(&keyword.to_lowercase()))
                })
            })
            .collect()
    }
    
    pub fn get_contextual_memories(&self, user_id: &str, query: &str, limit: usize) -> Vec<&Memory> {
        let query_lower = query.to_lowercase();
        let mut relevant_memories: Vec<_> = self.memories
            .values()
            .filter(|m| {
                m.user_id == user_id && (
                    m.content.to_lowercase().contains(&query_lower) ||
                    m.summary.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
                )
            })
            .collect();
        
        // Sort by relevance (simple keyword matching for now)
        relevant_memories.sort_by(|a, b| {
            let score_a = Self::calculate_relevance_score(a, &query_lower);
            let score_b = Self::calculate_relevance_score(b, &query_lower);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        relevant_memories.into_iter().take(limit).collect()
    }
    
    fn calculate_relevance_score(memory: &Memory, query: &str) -> f64 {
        let content_matches = memory.content.to_lowercase().matches(query).count() as f64;
        let summary_matches = memory.summary.as_ref()
            .map_or(0.0, |s| s.to_lowercase().matches(query).count() as f64);
        
        let relevance = (content_matches + summary_matches) * memory.importance;
        let recency_bonus = 1.0 / ((Utc::now() - memory.created_at).num_days() as f64).max(1.0);
        
        relevance + recency_bonus * 0.1
    }
    
    pub fn create_summary(&mut self, user_id: &str, content: &str) -> Result<String> {
        // Simple summary creation (in real implementation, this would use AI)
        let summary = if content.len() > 100 {
            format!("{}...", &content[..97])
        } else {
            content.to_string()
        };
        
        self.add_memory(user_id, &summary, 0.8)
    }
    
    pub fn create_core_memory(&mut self, user_id: &str, content: &str) -> Result<String> {
        let memory_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let memory = Memory {
            id: memory_id.clone(),
            user_id: user_id.to_string(),
            content: content.to_string(),
            summary: None,
            importance: 1.0, // Core memories have maximum importance
            memory_type: MemoryType::Core,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        };
        
        self.memories.insert(memory_id.clone(), memory);
        self.save_memories()?;
        
        Ok(memory_id)
    }
    
    pub fn get_memory_stats(&self, user_id: &str) -> MemoryStats {
        let user_memories: Vec<_> = self.memories
            .values()
            .filter(|m| m.user_id == user_id)
            .collect();
        
        let total_memories = user_memories.len();
        let core_memories = user_memories.iter()
            .filter(|m| matches!(m.memory_type, MemoryType::Core))
            .count();
        let summary_memories = user_memories.iter()
            .filter(|m| matches!(m.memory_type, MemoryType::Summary))
            .count();
        let interaction_memories = user_memories.iter()
            .filter(|m| matches!(m.memory_type, MemoryType::Interaction))
            .count();
        
        let avg_importance = if total_memories > 0 {
            user_memories.iter().map(|m| m.importance).sum::<f64>() / total_memories as f64
        } else {
            0.0
        };
        
        MemoryStats {
            total_memories,
            core_memories,
            summary_memories,
            interaction_memories,
            avg_importance,
        }
    }
    
    fn load_memories(config: &Config) -> Result<HashMap<String, Memory>> {
        let file_path = config.memory_file();
        if !file_path.exists() {
            return Ok(HashMap::new());
        }
        
        let content = std::fs::read_to_string(file_path)
            .context("Failed to read memories file")?;
        
        let memories: HashMap<String, Memory> = serde_json::from_str(&content)
            .context("Failed to parse memories file")?;
        
        Ok(memories)
    }
    
    fn save_memories(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.memories)
            .context("Failed to serialize memories")?;
        
        std::fs::write(&self.config.memory_file(), content)
            .context("Failed to write memories file")?;
        
        Ok(())
    }
    
    pub fn get_stats(&self) -> Result<MemoryStats> {
        let total_memories = self.memories.len();
        let core_memories = self.memories.values()
            .filter(|m| matches!(m.memory_type, MemoryType::Core))
            .count();
        let summary_memories = self.memories.values()
            .filter(|m| matches!(m.memory_type, MemoryType::Summary))
            .count();
        let interaction_memories = self.memories.values()
            .filter(|m| matches!(m.memory_type, MemoryType::Interaction))
            .count();
        
        let avg_importance = if total_memories > 0 {
            self.memories.values().map(|m| m.importance).sum::<f64>() / total_memories as f64
        } else {
            0.0
        };
        
        Ok(MemoryStats {
            total_memories,
            core_memories,
            summary_memories,
            interaction_memories,
            avg_importance,
        })
    }
    
    pub async fn run_maintenance(&mut self) -> Result<()> {
        // Cleanup old, low-importance memories
        let cutoff_date = Utc::now() - chrono::Duration::days(30);
        let memory_ids_to_remove: Vec<String> = self.memories
            .iter()
            .filter(|(_, m)| {
                m.importance < 0.3 
                && m.created_at < cutoff_date 
                && m.access_count <= 1
                && !matches!(m.memory_type, MemoryType::Core)
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in memory_ids_to_remove {
            self.memories.remove(&id);
        }
        
        // Mark old memories as forgotten instead of deleting
        let forgotten_cutoff = Utc::now() - chrono::Duration::days(90);
        for memory in self.memories.values_mut() {
            if memory.created_at < forgotten_cutoff 
                && memory.importance < 0.2 
                && !matches!(memory.memory_type, MemoryType::Core) {
                memory.memory_type = MemoryType::Forgotten;
            }
        }
        
        // Save changes
        self.save_memories()?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub core_memories: usize,
    pub summary_memories: usize,
    pub interaction_memories: usize,
    pub avg_importance: f64,
}