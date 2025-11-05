use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Represents a single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier using ULID (time-sortable)
    pub id: String,

    /// The actual content of the memory
    pub content: String,

    /// AI's creative interpretation of the content (Layer 2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_interpretation: Option<String>,

    /// Priority score evaluated by AI: 0.0 (low) to 1.0 (high) (Layer 2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_score: Option<f32>,

    /// When this memory was created
    pub created_at: DateTime<Utc>,

    /// When this memory was last updated
    pub updated_at: DateTime<Utc>,
}

impl Memory {
    /// Create a new memory with generated ULID (Layer 1)
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        let id = Ulid::new().to_string();

        Self {
            id,
            content,
            ai_interpretation: None,
            priority_score: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new AI-interpreted memory (Layer 2)
    pub fn new_ai(
        content: String,
        ai_interpretation: Option<String>,
        priority_score: Option<f32>,
    ) -> Self {
        let now = Utc::now();
        let id = Ulid::new().to_string();

        Self {
            id,
            content,
            ai_interpretation,
            priority_score,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the content of this memory
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }

    /// Set or update AI interpretation
    pub fn set_ai_interpretation(&mut self, interpretation: String) {
        self.ai_interpretation = Some(interpretation);
        self.updated_at = Utc::now();
    }

    /// Set or update priority score
    pub fn set_priority_score(&mut self, score: f32) {
        self.priority_score = Some(score.clamp(0.0, 1.0));
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_memory() {
        let memory = Memory::new("Test content".to_string());
        assert_eq!(memory.content, "Test content");
        assert!(!memory.id.is_empty());
        assert!(memory.ai_interpretation.is_none());
        assert!(memory.priority_score.is_none());
    }

    #[test]
    fn test_new_ai_memory() {
        let memory = Memory::new_ai(
            "Test content".to_string(),
            Some("AI interpretation".to_string()),
            Some(0.75),
        );
        assert_eq!(memory.content, "Test content");
        assert_eq!(memory.ai_interpretation, Some("AI interpretation".to_string()));
        assert_eq!(memory.priority_score, Some(0.75));
    }

    #[test]
    fn test_update_memory() {
        let mut memory = Memory::new("Original".to_string());
        let original_time = memory.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        memory.update_content("Updated".to_string());

        assert_eq!(memory.content, "Updated");
        assert!(memory.updated_at > original_time);
    }

    #[test]
    fn test_set_ai_interpretation() {
        let mut memory = Memory::new("Test".to_string());
        memory.set_ai_interpretation("Interpretation".to_string());
        assert_eq!(memory.ai_interpretation, Some("Interpretation".to_string()));
    }

    #[test]
    fn test_set_priority_score() {
        let mut memory = Memory::new("Test".to_string());
        memory.set_priority_score(0.8);
        assert_eq!(memory.priority_score, Some(0.8));

        // Test clamping
        memory.set_priority_score(1.5);
        assert_eq!(memory.priority_score, Some(1.0));

        memory.set_priority_score(-0.5);
        assert_eq!(memory.priority_score, Some(0.0));
    }
}
