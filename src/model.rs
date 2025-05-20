//src/model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AiSystem {
    pub personality: Personality,
    pub relationship: Relationship,
    pub environment: Environment,
    pub messaging: Messaging,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Personality {
    pub kind: String, // e.g., "positive", "negative", "neutral"
    pub strength: f32, // 0.0 - 1.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub trust: f32,       // 0.0 - 1.0
    pub intimacy: f32,    // 0.0 - 1.0
    pub curiosity: f32,   // 0.0 - 1.0
    pub threshold: f32,   // if sum > threshold, allow messaging
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    pub luck_today: f32,        // 0.1 - 1.0
    pub luck_history: Vec<f32>, // last 3 values
    pub level: i32,             // current mental strength level
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Messaging {
    pub enabled: bool,
    pub schedule_time: Option<String>, // e.g., "08:00"
    pub decay_rate: f32,               // how quickly emotion fades (0.0 - 1.0)
    pub templates: Vec<String>,       // message template variations
}


