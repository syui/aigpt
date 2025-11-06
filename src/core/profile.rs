use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::{MemoryStore, UserAnalysis};
use crate::core::error::Result;

/// Integrated user profile - the essence of Layer 1-3 data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Dominant personality traits (top 2-3 from Big Five)
    pub dominant_traits: Vec<TraitScore>,

    /// Core interests (most frequent topics from memories)
    pub core_interests: Vec<String>,

    /// Core values (extracted from high-priority memories)
    pub core_values: Vec<String>,

    /// Key memory IDs (top priority memories as evidence)
    pub key_memory_ids: Vec<String>,

    /// Data quality score (0.0-1.0 based on data volume)
    pub data_quality: f32,

    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitScore {
    pub name: String,
    pub score: f32,
}

impl UserProfile {
    /// Generate integrated profile from Layer 1-3 data
    pub fn generate(store: &MemoryStore) -> Result<Self> {
        // Get latest personality analysis (Layer 3)
        let personality = store.get_latest_analysis()?;

        // Get all memories (Layer 1-2)
        let memories = store.list()?;

        // Extract dominant traits from Big Five
        let dominant_traits = extract_dominant_traits(&personality);

        // Extract core interests from memory content
        let core_interests = extract_core_interests(&memories);

        // Extract core values from high-priority memories
        let core_values = extract_core_values(&memories);

        // Get top priority memory IDs
        let key_memory_ids = extract_key_memories(&memories);

        // Calculate data quality
        let data_quality = calculate_data_quality(&memories, &personality);

        Ok(UserProfile {
            dominant_traits,
            core_interests,
            core_values,
            key_memory_ids,
            data_quality,
            last_updated: Utc::now(),
        })
    }

    /// Check if profile needs update
    pub fn needs_update(&self, store: &MemoryStore) -> Result<bool> {
        // Update if 7+ days old
        let days_old = (Utc::now() - self.last_updated).num_days();
        if days_old >= 7 {
            return Ok(true);
        }

        // Update if 10+ new memories since last update
        let memory_count = store.count()?;
        let expected_count = self.key_memory_ids.len() * 2; // Rough estimate
        if memory_count > expected_count + 10 {
            return Ok(true);
        }

        // Update if new personality analysis exists
        if let Some(latest) = store.get_latest_analysis()? {
            if latest.analyzed_at > self.last_updated {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Extract top 2-3 personality traits from Big Five
fn extract_dominant_traits(analysis: &Option<UserAnalysis>) -> Vec<TraitScore> {
    if analysis.is_none() {
        return vec![];
    }

    let analysis = analysis.as_ref().unwrap();

    let mut traits = vec![
        TraitScore { name: "openness".to_string(), score: analysis.openness },
        TraitScore { name: "conscientiousness".to_string(), score: analysis.conscientiousness },
        TraitScore { name: "extraversion".to_string(), score: analysis.extraversion },
        TraitScore { name: "agreeableness".to_string(), score: analysis.agreeableness },
        TraitScore { name: "neuroticism".to_string(), score: analysis.neuroticism },
    ];

    // Sort by score descending
    traits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Return top 3
    traits.into_iter().take(3).collect()
}

/// Extract core interests from memory content (frequency analysis)
fn extract_core_interests(memories: &[crate::core::Memory]) -> Vec<String> {
    let mut word_freq: HashMap<String, usize> = HashMap::new();

    for memory in memories {
        // Extract keywords from content
        let words = extract_keywords(&memory.content);
        for word in words {
            *word_freq.entry(word).or_insert(0) += 1;
        }

        // Also consider AI interpretation if available
        if let Some(ref interpretation) = memory.ai_interpretation {
            let words = extract_keywords(interpretation);
            for word in words {
                *word_freq.entry(word).or_insert(0) += 2; // Weight interpretation higher
            }
        }
    }

    // Sort by frequency and take top 5
    let mut freq_vec: Vec<_> = word_freq.into_iter().collect();
    freq_vec.sort_by(|a, b| b.1.cmp(&a.1));

    freq_vec.into_iter()
        .take(5)
        .map(|(word, _)| word)
        .collect()
}

/// Extract core values from high-priority memories
fn extract_core_values(memories: &[crate::core::Memory]) -> Vec<String> {
    // Filter high-priority memories (>= 0.7)
    let high_priority: Vec<_> = memories.iter()
        .filter(|m| m.priority_score.map(|s| s >= 0.7).unwrap_or(false))
        .collect();

    if high_priority.is_empty() {
        return vec![];
    }

    let mut value_freq: HashMap<String, usize> = HashMap::new();

    for memory in high_priority {
        // Extract value keywords from interpretation
        if let Some(ref interpretation) = memory.ai_interpretation {
            let values = extract_value_keywords(interpretation);
            for value in values {
                *value_freq.entry(value).or_insert(0) += 1;
            }
        }
    }

    // Sort by frequency and take top 5
    let mut freq_vec: Vec<_> = value_freq.into_iter().collect();
    freq_vec.sort_by(|a, b| b.1.cmp(&a.1));

    freq_vec.into_iter()
        .take(5)
        .map(|(value, _)| value)
        .collect()
}

/// Extract key memory IDs (top priority)
fn extract_key_memories(memories: &[crate::core::Memory]) -> Vec<String> {
    let mut sorted_memories: Vec<_> = memories.iter()
        .filter(|m| m.priority_score.is_some())
        .collect();

    sorted_memories.sort_by(|a, b| {
        b.priority_score.unwrap()
            .partial_cmp(&a.priority_score.unwrap())
            .unwrap()
    });

    sorted_memories.into_iter()
        .take(10)
        .map(|m| m.id.clone())
        .collect()
}

/// Calculate data quality based on volume
fn calculate_data_quality(memories: &[crate::core::Memory], personality: &Option<UserAnalysis>) -> f32 {
    let memory_count = memories.len() as f32;
    let has_personality = if personality.is_some() { 1.0 } else { 0.0 };

    // Quality increases with data volume
    let memory_quality = (memory_count / 50.0).min(1.0); // Max quality at 50+ memories
    let personality_quality = has_personality * 0.5;

    // Weighted average
    (memory_quality * 0.5 + personality_quality).min(1.0)
}

/// Extract keywords from text (simple word frequency)
fn extract_keywords(text: &str) -> Vec<String> {
    // Simple keyword extraction: words longer than 3 chars
    text.split_whitespace()
        .filter(|w| w.len() > 3)
        .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !is_stopword(w))
        .collect()
}

/// Extract value-related keywords from interpretation
fn extract_value_keywords(text: &str) -> Vec<String> {
    let value_indicators = [
        "重視", "大切", "価値", "重要", "優先", "好む", "志向",
        "シンプル", "効率", "品質", "安定", "革新", "創造",
        "value", "important", "priority", "prefer", "focus",
        "simple", "efficient", "quality", "stable", "creative",
    ];

    let words = extract_keywords(text);
    words.into_iter()
        .filter(|w| {
            value_indicators.iter().any(|indicator| w.contains(indicator))
        })
        .collect()
}

/// Check if word is a stopword
fn is_stopword(word: &str) -> bool {
    let stopwords = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
        "be", "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "can", "this", "that", "these", "those",
        "です", "ます", "ました", "である", "ある", "いる", "する", "した",
        "という", "として", "ために", "によって", "について",
    ];

    stopwords.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let text = "Rust architecture design is important for scalability";
        let keywords = extract_keywords(text);

        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"architecture".to_string()));
        assert!(keywords.contains(&"design".to_string()));
        assert!(!keywords.contains(&"is".to_string())); // stopword
    }

    #[test]
    fn test_stopword() {
        assert!(is_stopword("the"));
        assert!(is_stopword("です"));
        assert!(!is_stopword("rust"));
    }
}
