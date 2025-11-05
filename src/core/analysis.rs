use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// User personality analysis based on Big Five model (OCEAN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnalysis {
    /// Unique identifier using ULID
    pub id: String,

    /// Openness to Experience (0.0-1.0)
    /// Curiosity, imagination, willingness to try new things
    pub openness: f32,

    /// Conscientiousness (0.0-1.0)
    /// Organization, responsibility, self-discipline
    pub conscientiousness: f32,

    /// Extraversion (0.0-1.0)
    /// Sociability, assertiveness, energy level
    pub extraversion: f32,

    /// Agreeableness (0.0-1.0)
    /// Compassion, cooperation, trust
    pub agreeableness: f32,

    /// Neuroticism (0.0-1.0)
    /// Emotional stability, anxiety, mood swings
    pub neuroticism: f32,

    /// AI-generated summary of the personality analysis
    pub summary: String,

    /// When this analysis was performed
    pub analyzed_at: DateTime<Utc>,
}

impl UserAnalysis {
    /// Create a new personality analysis
    pub fn new(
        openness: f32,
        conscientiousness: f32,
        extraversion: f32,
        agreeableness: f32,
        neuroticism: f32,
        summary: String,
    ) -> Self {
        let id = Ulid::new().to_string();
        let analyzed_at = Utc::now();

        Self {
            id,
            openness: openness.clamp(0.0, 1.0),
            conscientiousness: conscientiousness.clamp(0.0, 1.0),
            extraversion: extraversion.clamp(0.0, 1.0),
            agreeableness: agreeableness.clamp(0.0, 1.0),
            neuroticism: neuroticism.clamp(0.0, 1.0),
            summary,
            analyzed_at,
        }
    }

    /// Get the dominant trait (highest score)
    pub fn dominant_trait(&self) -> &str {
        let scores = [
            (self.openness, "Openness"),
            (self.conscientiousness, "Conscientiousness"),
            (self.extraversion, "Extraversion"),
            (self.agreeableness, "Agreeableness"),
            (self.neuroticism, "Neuroticism"),
        ];

        scores
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, name)| *name)
            .unwrap_or("Unknown")
    }

    /// Check if a trait is high (>= 0.6)
    pub fn is_high(&self, trait_name: &str) -> bool {
        let score = match trait_name.to_lowercase().as_str() {
            "openness" | "o" => self.openness,
            "conscientiousness" | "c" => self.conscientiousness,
            "extraversion" | "e" => self.extraversion,
            "agreeableness" | "a" => self.agreeableness,
            "neuroticism" | "n" => self.neuroticism,
            _ => return false,
        };
        score >= 0.6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_analysis() {
        let analysis = UserAnalysis::new(
            0.8,
            0.7,
            0.4,
            0.6,
            0.3,
            "Test summary".to_string(),
        );

        assert_eq!(analysis.openness, 0.8);
        assert_eq!(analysis.conscientiousness, 0.7);
        assert_eq!(analysis.extraversion, 0.4);
        assert_eq!(analysis.agreeableness, 0.6);
        assert_eq!(analysis.neuroticism, 0.3);
        assert!(!analysis.id.is_empty());
    }

    #[test]
    fn test_score_clamping() {
        let analysis = UserAnalysis::new(
            1.5,  // Should clamp to 1.0
            -0.2, // Should clamp to 0.0
            0.5,
            0.5,
            0.5,
            "Test".to_string(),
        );

        assert_eq!(analysis.openness, 1.0);
        assert_eq!(analysis.conscientiousness, 0.0);
    }

    #[test]
    fn test_dominant_trait() {
        let analysis = UserAnalysis::new(
            0.9, // Highest
            0.5,
            0.4,
            0.6,
            0.3,
            "Test".to_string(),
        );

        assert_eq!(analysis.dominant_trait(), "Openness");
    }

    #[test]
    fn test_is_high() {
        let analysis = UserAnalysis::new(
            0.8, // High
            0.4, // Low
            0.6, // Threshold
            0.5,
            0.3,
            "Test".to_string(),
        );

        assert!(analysis.is_high("openness"));
        assert!(!analysis.is_high("conscientiousness"));
        assert!(analysis.is_high("extraversion")); // 0.6 is high
    }
}
