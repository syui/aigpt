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

    /// When this memory was created
    pub created_at: DateTime<Utc>,

    /// When this memory was last updated
    pub updated_at: DateTime<Utc>,
}

impl Memory {
    /// Create a new memory with generated ULID
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        let id = Ulid::new().to_string();

        Self {
            id,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the content of this memory
    pub fn update_content(&mut self, content: String) {
        self.content = content;
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
}
