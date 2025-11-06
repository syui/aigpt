use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::{Memory, MemoryStore, UserProfile};
use crate::core::error::Result;

/// Inferred relationship with an entity (Layer 4)
///
/// This is not stored permanently but generated on-demand from
/// Layer 1 memories and Layer 3.5 user profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipInference {
    /// Entity identifier
    pub entity_id: String,

    /// Total interaction count with this entity
    pub interaction_count: u32,

    /// Average priority score of memories with this entity
    pub avg_priority: f32,

    /// Days since last interaction
    pub days_since_last: i64,

    /// Inferred bond strength (0.0-1.0)
    pub bond_strength: f32,

    /// Inferred relationship type
    pub relationship_type: String,

    /// Confidence in this inference (0.0-1.0, based on data volume)
    pub confidence: f32,

    /// When this inference was generated
    pub inferred_at: DateTime<Utc>,
}

impl RelationshipInference {
    /// Infer relationship from memories and user profile
    pub fn infer(
        entity_id: String,
        memories: &[Memory],
        user_profile: &UserProfile,
    ) -> Self {
        // Filter memories related to this entity
        let entity_memories: Vec<_> = memories
            .iter()
            .filter(|m| m.has_entity(&entity_id))
            .collect();

        let interaction_count = entity_memories.len() as u32;

        // Calculate average priority
        let total_priority: f32 = entity_memories
            .iter()
            .filter_map(|m| m.priority_score)
            .sum();
        let priority_count = entity_memories
            .iter()
            .filter(|m| m.priority_score.is_some())
            .count() as f32;
        let avg_priority = if priority_count > 0.0 {
            total_priority / priority_count
        } else {
            0.5 // Default to neutral if no scores
        };

        // Calculate days since last interaction
        let days_since_last = entity_memories
            .iter()
            .map(|m| (Utc::now() - m.created_at).num_days())
            .min()
            .unwrap_or(999);

        // Infer bond strength based on user personality
        let bond_strength = Self::calculate_bond_strength(
            interaction_count,
            avg_priority,
            user_profile,
        );

        // Infer relationship type
        let relationship_type = Self::infer_relationship_type(
            interaction_count,
            avg_priority,
            bond_strength,
        );

        // Calculate confidence
        let confidence = Self::calculate_confidence(interaction_count);

        RelationshipInference {
            entity_id,
            interaction_count,
            avg_priority,
            days_since_last,
            bond_strength,
            relationship_type,
            confidence,
            inferred_at: Utc::now(),
        }
    }

    /// Calculate bond strength from interaction data and user personality
    fn calculate_bond_strength(
        interaction_count: u32,
        avg_priority: f32,
        user_profile: &UserProfile,
    ) -> f32 {
        // Extract extraversion score (if available)
        let extraversion = user_profile
            .dominant_traits
            .iter()
            .find(|t| t.name == "extraversion")
            .map(|t| t.score)
            .unwrap_or(0.5);

        let bond_strength = if extraversion < 0.5 {
            // Introverted: fewer but deeper relationships
            // Interaction count matters more
            let count_factor = (interaction_count as f32 / 20.0).min(1.0);
            let priority_factor = avg_priority;

            // Weight: 60% count, 40% priority
            count_factor * 0.6 + priority_factor * 0.4
        } else {
            // Extroverted: many relationships, quality varies
            // Priority matters more
            let count_factor = (interaction_count as f32 / 50.0).min(1.0);
            let priority_factor = avg_priority;

            // Weight: 40% count, 60% priority
            count_factor * 0.4 + priority_factor * 0.6
        };

        bond_strength.clamp(0.0, 1.0)
    }

    /// Infer relationship type from metrics
    fn infer_relationship_type(
        interaction_count: u32,
        avg_priority: f32,
        bond_strength: f32,
    ) -> String {
        if bond_strength >= 0.8 {
            "close_friend".to_string()
        } else if bond_strength >= 0.6 {
            "friend".to_string()
        } else if bond_strength >= 0.4 {
            if avg_priority >= 0.6 {
                "valued_acquaintance".to_string()
            } else {
                "acquaintance".to_string()
            }
        } else if interaction_count >= 5 {
            "regular_contact".to_string()
        } else {
            "distant".to_string()
        }
    }

    /// Calculate confidence based on data volume
    fn calculate_confidence(interaction_count: u32) -> f32 {
        // Confidence increases with more data
        // 1-2 interactions: low confidence (0.2-0.3)
        // 5 interactions: medium confidence (0.5)
        // 10+ interactions: high confidence (0.8+)
        let confidence = match interaction_count {
            0 => 0.0,
            1 => 0.2,
            2 => 0.3,
            3 => 0.4,
            4 => 0.45,
            5..=9 => 0.5 + (interaction_count - 5) as f32 * 0.05,
            _ => 0.8 + ((interaction_count - 10) as f32 * 0.02).min(0.2),
        };

        confidence.clamp(0.0, 1.0)
    }
}

/// Generate relationship inferences for all entities in memories
pub fn infer_all_relationships(
    store: &MemoryStore,
) -> Result<Vec<RelationshipInference>> {
    // Check cache first
    if let Some(cached) = store.get_cached_all_relationships()? {
        return Ok(cached);
    }

    // Get all memories
    let memories = store.list()?;

    // Get user profile
    let user_profile = store.get_profile()?;

    // Extract all unique entities
    let mut entities: HashMap<String, ()> = HashMap::new();
    for memory in &memories {
        if let Some(ref entity_list) = memory.related_entities {
            for entity in entity_list {
                entities.insert(entity.clone(), ());
            }
        }
    }

    // Infer relationship for each entity
    let mut relationships: Vec<_> = entities
        .keys()
        .map(|entity_id| {
            RelationshipInference::infer(
                entity_id.clone(),
                &memories,
                &user_profile,
            )
        })
        .collect();

    // Sort by bond strength (descending)
    relationships.sort_by(|a, b| {
        b.bond_strength
            .partial_cmp(&a.bond_strength)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Cache the result
    store.save_all_relationships_cache(&relationships)?;

    Ok(relationships)
}

/// Get relationship inference for a specific entity (with caching)
pub fn get_relationship(
    store: &MemoryStore,
    entity_id: &str,
) -> Result<RelationshipInference> {
    // Check cache first
    if let Some(cached) = store.get_cached_relationship(entity_id)? {
        return Ok(cached);
    }

    // Get all memories
    let memories = store.list()?;

    // Get user profile
    let user_profile = store.get_profile()?;

    // Infer relationship
    let relationship = RelationshipInference::infer(
        entity_id.to_string(),
        &memories,
        &user_profile,
    );

    // Cache it
    store.save_relationship_cache(entity_id, &relationship)?;

    Ok(relationship)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::profile::TraitScore;

    #[test]
    fn test_confidence_calculation() {
        assert_eq!(RelationshipInference::calculate_confidence(0), 0.0);
        assert_eq!(RelationshipInference::calculate_confidence(1), 0.2);
        assert_eq!(RelationshipInference::calculate_confidence(5), 0.5);
        assert!(RelationshipInference::calculate_confidence(10) >= 0.8);
    }

    #[test]
    fn test_relationship_type() {
        assert_eq!(
            RelationshipInference::infer_relationship_type(20, 0.9, 0.85),
            "close_friend"
        );
        assert_eq!(
            RelationshipInference::infer_relationship_type(10, 0.7, 0.65),
            "friend"
        );
        assert_eq!(
            RelationshipInference::infer_relationship_type(5, 0.5, 0.45),
            "acquaintance"
        );
    }

    #[test]
    fn test_bond_strength_introverted() {
        let user_profile = UserProfile {
            dominant_traits: vec![
                TraitScore {
                    name: "extraversion".to_string(),
                    score: 0.3, // Introverted
                },
            ],
            core_interests: vec![],
            core_values: vec![],
            key_memory_ids: vec![],
            data_quality: 1.0,
            last_updated: Utc::now(),
        };

        // Introverted: count matters more
        let strength = RelationshipInference::calculate_bond_strength(
            20, // Many interactions
            0.5, // Medium priority
            &user_profile,
        );

        // Should be high due to high interaction count
        assert!(strength > 0.5);
    }
}
