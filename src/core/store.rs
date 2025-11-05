use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use super::error::{MemoryError, Result};
use super::memory::Memory;

/// SQLite-based memory storage
pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    /// Create a new MemoryStore with the given database path
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;

        // Initialize database schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON memories(created_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_updated_at ON memories(updated_at)",
            [],
        )?;

        Ok(Self { conn })
    }

    /// Create a new MemoryStore using default config directory
    pub fn default() -> Result<Self> {
        let data_dir = dirs::config_dir()
            .ok_or_else(|| MemoryError::Config("Could not find config directory".to_string()))?
            .join("syui")
            .join("ai")
            .join("gpt");

        let db_path = data_dir.join("memory.db");
        Self::new(db_path)
    }

    /// Insert a new memory
    pub fn create(&self, memory: &Memory) -> Result<()> {
        self.conn.execute(
            "INSERT INTO memories (id, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                &memory.id,
                &memory.content,
                memory.created_at.to_rfc3339(),
                memory.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get a memory by ID
    pub fn get(&self, id: &str) -> Result<Memory> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, content, created_at, updated_at FROM memories WHERE id = ?1")?;

        let memory = stmt.query_row(params![id], |row| {
            let created_at: String = row.get(2)?;
            let updated_at: String = row.get(3)?;

            Ok(Memory {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?,
                updated_at: DateTime::parse_from_rfc3339(&updated_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?,
            })
        })?;

        Ok(memory)
    }

    /// Update an existing memory
    pub fn update(&self, memory: &Memory) -> Result<()> {
        let rows_affected = self.conn.execute(
            "UPDATE memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
            params![
                &memory.content,
                memory.updated_at.to_rfc3339(),
                &memory.id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(memory.id.clone()));
        }

        Ok(())
    }

    /// Delete a memory by ID
    pub fn delete(&self, id: &str) -> Result<()> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM memories WHERE id = ?1", params![id])?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(id.to_string()));
        }

        Ok(())
    }

    /// List all memories, ordered by creation time (newest first)
    pub fn list(&self) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, created_at, updated_at FROM memories ORDER BY created_at DESC",
        )?;

        let memories = stmt
            .query_map([], |row| {
                let created_at: String = row.get(2)?;
                let updated_at: String = row.get(3)?;

                Ok(Memory {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    /// Search memories by content (case-insensitive)
    pub fn search(&self, query: &str) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, created_at, updated_at FROM memories
             WHERE content LIKE ?1
             ORDER BY created_at DESC",
        )?;

        let search_pattern = format!("%{}%", query);
        let memories = stmt
            .query_map(params![search_pattern], |row| {
                let created_at: String = row.get(2)?;
                let updated_at: String = row.get(3)?;

                Ok(Memory {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    /// Count total memories
    pub fn count(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> MemoryStore {
        MemoryStore::new(":memory:".into()).unwrap()
    }

    #[test]
    fn test_create_and_get() {
        let store = create_test_store();
        let memory = Memory::new("Test content".to_string());

        store.create(&memory).unwrap();
        let retrieved = store.get(&memory.id).unwrap();

        assert_eq!(retrieved.id, memory.id);
        assert_eq!(retrieved.content, memory.content);
    }

    #[test]
    fn test_update() {
        let store = create_test_store();
        let mut memory = Memory::new("Original".to_string());

        store.create(&memory).unwrap();

        memory.update_content("Updated".to_string());
        store.update(&memory).unwrap();

        let retrieved = store.get(&memory.id).unwrap();
        assert_eq!(retrieved.content, "Updated");
    }

    #[test]
    fn test_delete() {
        let store = create_test_store();
        let memory = Memory::new("To delete".to_string());

        store.create(&memory).unwrap();
        store.delete(&memory.id).unwrap();

        assert!(store.get(&memory.id).is_err());
    }

    #[test]
    fn test_list() {
        let store = create_test_store();

        let mem1 = Memory::new("First".to_string());
        let mem2 = Memory::new("Second".to_string());

        store.create(&mem1).unwrap();
        store.create(&mem2).unwrap();

        let memories = store.list().unwrap();
        assert_eq!(memories.len(), 2);
    }

    #[test]
    fn test_search() {
        let store = create_test_store();

        store
            .create(&Memory::new("Hello world".to_string()))
            .unwrap();
        store
            .create(&Memory::new("Goodbye world".to_string()))
            .unwrap();
        store.create(&Memory::new("Testing".to_string())).unwrap();

        let results = store.search("world").unwrap();
        assert_eq!(results.len(), 2);

        let results = store.search("Hello").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_count() {
        let store = create_test_store();
        assert_eq!(store.count().unwrap(), 0);

        store.create(&Memory::new("Test".to_string())).unwrap();
        assert_eq!(store.count().unwrap(), 1);
    }
}
