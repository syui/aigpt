use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use super::analysis::UserAnalysis;
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
                ai_interpretation TEXT,
                priority_score REAL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Migrate existing tables (add columns if they don't exist)
        // SQLite doesn't have "IF NOT EXISTS" for columns, so we check first
        let has_ai_interpretation: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('memories') WHERE name='ai_interpretation'")?
            .query_row([], |row| row.get(0))
            .map(|count: i32| count > 0)?;

        if !has_ai_interpretation {
            conn.execute("ALTER TABLE memories ADD COLUMN ai_interpretation TEXT", [])?;
            conn.execute("ALTER TABLE memories ADD COLUMN priority_score REAL", [])?;
        }

        // Migrate for Layer 4: related_entities
        let has_related_entities: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('memories') WHERE name='related_entities'")?
            .query_row([], |row| row.get(0))
            .map(|count: i32| count > 0)?;

        if !has_related_entities {
            conn.execute("ALTER TABLE memories ADD COLUMN related_entities TEXT", [])?;
        }

        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON memories(created_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_updated_at ON memories(updated_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_priority_score ON memories(priority_score)",
            [],
        )?;

        // Create user_analyses table (Layer 3)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_analyses (
                id TEXT PRIMARY KEY,
                openness REAL NOT NULL,
                conscientiousness REAL NOT NULL,
                extraversion REAL NOT NULL,
                agreeableness REAL NOT NULL,
                neuroticism REAL NOT NULL,
                summary TEXT NOT NULL,
                analyzed_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_analyzed_at ON user_analyses(analyzed_at)",
            [],
        )?;

        // Create user_profiles table (Layer 3.5 - integrated profile cache)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_profiles (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                data TEXT NOT NULL,
                last_updated TEXT NOT NULL
            )",
            [],
        )?;

        // Create relationship_cache table (Layer 4 - relationship inference cache)
        // entity_id = "" for all_relationships cache
        conn.execute(
            "CREATE TABLE IF NOT EXISTS relationship_cache (
                entity_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                cached_at TEXT NOT NULL
            )",
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
        let related_entities_json = memory.related_entities
            .as_ref()
            .map(|entities| serde_json::to_string(entities).ok())
            .flatten();

        self.conn.execute(
            "INSERT INTO memories (id, content, ai_interpretation, priority_score, related_entities, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &memory.id,
                &memory.content,
                &memory.ai_interpretation,
                &memory.priority_score,
                related_entities_json,
                memory.created_at.to_rfc3339(),
                memory.updated_at.to_rfc3339(),
            ],
        )?;

        // Clear relationship cache since memory data changed
        self.clear_relationship_cache()?;

        Ok(())
    }

    /// Get a memory by ID
    pub fn get(&self, id: &str) -> Result<Memory> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, content, ai_interpretation, priority_score, related_entities, created_at, updated_at
                      FROM memories WHERE id = ?1")?;

        let memory = stmt.query_row(params![id], |row| {
            let created_at: String = row.get(5)?;
            let updated_at: String = row.get(6)?;
            let related_entities_json: Option<String> = row.get(4)?;
            let related_entities = related_entities_json
                .and_then(|json| serde_json::from_str(&json).ok());

            Ok(Memory {
                id: row.get(0)?,
                content: row.get(1)?,
                ai_interpretation: row.get(2)?,
                priority_score: row.get(3)?,
                related_entities,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?,
                updated_at: DateTime::parse_from_rfc3339(&updated_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?,
            })
        })?;

        Ok(memory)
    }

    /// Update an existing memory
    pub fn update(&self, memory: &Memory) -> Result<()> {
        let related_entities_json = memory.related_entities
            .as_ref()
            .map(|entities| serde_json::to_string(entities).ok())
            .flatten();

        let rows_affected = self.conn.execute(
            "UPDATE memories SET content = ?1, ai_interpretation = ?2, priority_score = ?3, related_entities = ?4, updated_at = ?5
             WHERE id = ?6",
            params![
                &memory.content,
                &memory.ai_interpretation,
                &memory.priority_score,
                related_entities_json,
                memory.updated_at.to_rfc3339(),
                &memory.id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(memory.id.clone()));
        }

        // Clear relationship cache since memory data changed
        self.clear_relationship_cache()?;

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

        // Clear relationship cache since memory data changed
        self.clear_relationship_cache()?;

        Ok(())
    }

    /// List all memories, ordered by creation time (newest first)
    pub fn list(&self) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, ai_interpretation, priority_score, related_entities, created_at, updated_at
             FROM memories ORDER BY created_at DESC",
        )?;

        let memories = stmt
            .query_map([], |row| {
                let created_at: String = row.get(5)?;
                let updated_at: String = row.get(6)?;
                let related_entities_json: Option<String> = row.get(4)?;
                let related_entities = related_entities_json
                    .and_then(|json| serde_json::from_str(&json).ok());

                Ok(Memory {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    ai_interpretation: row.get(2)?,
                    priority_score: row.get(3)?,
                    related_entities,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    /// Search memories by content or AI interpretation (case-insensitive)
    pub fn search(&self, query: &str) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, ai_interpretation, priority_score, related_entities, created_at, updated_at
             FROM memories
             WHERE content LIKE ?1 OR ai_interpretation LIKE ?1
             ORDER BY created_at DESC",
        )?;

        let search_pattern = format!("%{}%", query);
        let memories = stmt
            .query_map(params![search_pattern], |row| {
                let created_at: String = row.get(5)?;
                let updated_at: String = row.get(6)?;
                let related_entities_json: Option<String> = row.get(4)?;
                let related_entities = related_entities_json
                    .and_then(|json| serde_json::from_str(&json).ok());

                Ok(Memory {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    ai_interpretation: row.get(2)?,
                    priority_score: row.get(3)?,
                    related_entities,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?,
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            6,
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

    // ========== Layer 3: User Analysis Methods ==========

    /// Save a new user personality analysis
    pub fn save_analysis(&self, analysis: &UserAnalysis) -> Result<()> {
        self.conn.execute(
            "INSERT INTO user_analyses (id, openness, conscientiousness, extraversion, agreeableness, neuroticism, summary, analyzed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &analysis.id,
                &analysis.openness,
                &analysis.conscientiousness,
                &analysis.extraversion,
                &analysis.agreeableness,
                &analysis.neuroticism,
                &analysis.summary,
                analysis.analyzed_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get the most recent user analysis
    pub fn get_latest_analysis(&self) -> Result<Option<UserAnalysis>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, openness, conscientiousness, extraversion, agreeableness, neuroticism, summary, analyzed_at
             FROM user_analyses
             ORDER BY analyzed_at DESC
             LIMIT 1",
        )?;

        let result = stmt.query_row([], |row| {
            let analyzed_at: String = row.get(7)?;

            Ok(UserAnalysis {
                id: row.get(0)?,
                openness: row.get(1)?,
                conscientiousness: row.get(2)?,
                extraversion: row.get(3)?,
                agreeableness: row.get(4)?,
                neuroticism: row.get(5)?,
                summary: row.get(6)?,
                analyzed_at: DateTime::parse_from_rfc3339(&analyzed_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
            })
        });

        match result {
            Ok(analysis) => Ok(Some(analysis)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all user analyses, ordered by date (newest first)
    pub fn list_analyses(&self) -> Result<Vec<UserAnalysis>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, openness, conscientiousness, extraversion, agreeableness, neuroticism, summary, analyzed_at
             FROM user_analyses
             ORDER BY analyzed_at DESC",
        )?;

        let analyses = stmt
            .query_map([], |row| {
                let analyzed_at: String = row.get(7)?;

                Ok(UserAnalysis {
                    id: row.get(0)?,
                    openness: row.get(1)?,
                    conscientiousness: row.get(2)?,
                    extraversion: row.get(3)?,
                    agreeableness: row.get(4)?,
                    neuroticism: row.get(5)?,
                    summary: row.get(6)?,
                    analyzed_at: DateTime::parse_from_rfc3339(&analyzed_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                7,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(analyses)
    }

    // === Layer 3.5: Integrated Profile ===

    /// Save integrated profile to cache
    pub fn save_profile(&self, profile: &super::profile::UserProfile) -> Result<()> {
        let profile_json = serde_json::to_string(profile)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO user_profiles (id, data, last_updated) VALUES (1, ?1, ?2)",
            params![profile_json, profile.last_updated.to_rfc3339()],
        )?;

        Ok(())
    }

    /// Get cached profile if exists
    pub fn get_cached_profile(&self) -> Result<Option<super::profile::UserProfile>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM user_profiles WHERE id = 1")?;

        let result = stmt.query_row([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        });

        match result {
            Ok(json) => {
                let profile: super::profile::UserProfile = serde_json::from_str(&json)?;
                Ok(Some(profile))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get or generate profile (with automatic caching)
    pub fn get_profile(&self) -> Result<super::profile::UserProfile> {
        // Check cache first
        if let Some(cached) = self.get_cached_profile()? {
            // Check if needs update
            if !cached.needs_update(self)? {
                return Ok(cached);
            }
        }

        // Generate new profile
        let profile = super::profile::UserProfile::generate(self)?;

        // Cache it
        self.save_profile(&profile)?;

        Ok(profile)
    }

    // ========== Layer 4: Relationship Cache Methods ==========

    /// Cache duration in minutes
    const RELATIONSHIP_CACHE_DURATION_MINUTES: i64 = 5;

    /// Save relationship inference to cache
    pub fn save_relationship_cache(
        &self,
        entity_id: &str,
        relationship: &super::relationship::RelationshipInference,
    ) -> Result<()> {
        let data = serde_json::to_string(relationship)?;
        let cached_at = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT OR REPLACE INTO relationship_cache (entity_id, data, cached_at) VALUES (?1, ?2, ?3)",
            params![entity_id, data, cached_at],
        )?;

        Ok(())
    }

    /// Get cached relationship inference
    pub fn get_cached_relationship(
        &self,
        entity_id: &str,
    ) -> Result<Option<super::relationship::RelationshipInference>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data, cached_at FROM relationship_cache WHERE entity_id = ?1")?;

        let result = stmt.query_row([entity_id], |row| {
            let data: String = row.get(0)?;
            let cached_at: String = row.get(1)?;
            Ok((data, cached_at))
        });

        match result {
            Ok((data, cached_at_str)) => {
                // Check if cache is still valid (within 5 minutes)
                let cached_at = DateTime::parse_from_rfc3339(&cached_at_str)
                    .map_err(|e| MemoryError::Parse(e.to_string()))?
                    .with_timezone(&Utc);

                let age_minutes = (Utc::now() - cached_at).num_minutes();

                if age_minutes < Self::RELATIONSHIP_CACHE_DURATION_MINUTES {
                    let relationship: super::relationship::RelationshipInference =
                        serde_json::from_str(&data)?;
                    Ok(Some(relationship))
                } else {
                    // Cache expired
                    Ok(None)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Save all relationships list to cache (use empty string as entity_id)
    pub fn save_all_relationships_cache(
        &self,
        relationships: &[super::relationship::RelationshipInference],
    ) -> Result<()> {
        let data = serde_json::to_string(relationships)?;
        let cached_at = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT OR REPLACE INTO relationship_cache (entity_id, data, cached_at) VALUES ('', ?1, ?2)",
            params![data, cached_at],
        )?;

        Ok(())
    }

    /// Get cached all relationships list
    pub fn get_cached_all_relationships(
        &self,
    ) -> Result<Option<Vec<super::relationship::RelationshipInference>>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data, cached_at FROM relationship_cache WHERE entity_id = ''")?;

        let result = stmt.query_row([], |row| {
            let data: String = row.get(0)?;
            let cached_at: String = row.get(1)?;
            Ok((data, cached_at))
        });

        match result {
            Ok((data, cached_at_str)) => {
                let cached_at = DateTime::parse_from_rfc3339(&cached_at_str)
                    .map_err(|e| MemoryError::Parse(e.to_string()))?
                    .with_timezone(&Utc);

                let age_minutes = (Utc::now() - cached_at).num_minutes();

                if age_minutes < Self::RELATIONSHIP_CACHE_DURATION_MINUTES {
                    let relationships: Vec<super::relationship::RelationshipInference> =
                        serde_json::from_str(&data)?;
                    Ok(Some(relationships))
                } else {
                    Ok(None)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Clear all relationship caches (call when memories are modified)
    pub fn clear_relationship_cache(&self) -> Result<()> {
        self.conn.execute("DELETE FROM relationship_cache", [])?;
        Ok(())
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
