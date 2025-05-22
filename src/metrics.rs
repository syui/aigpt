// src/metrics.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::config::ConfigPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub trust: f32,
    pub intimacy: f32,
    pub energy: f32,
    pub can_send: bool,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub kind: String,
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub trust: f32,
    pub intimacy: f32,
    pub curiosity: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub luck_today: f32,
    pub luck_history: Vec<f32>,
    pub level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Messaging {
    pub enabled: bool,
    pub schedule_time: Option<String>,
    pub decay_rate: f32,
    pub templates: Vec<String>,
    pub sent_today: bool, // 追加
    pub last_sent_date: Option<String>, // 追加
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub recent_messages: Vec<String>,
    pub long_term_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub personality: Personality,
    pub relationship: Relationship,
    pub environment: Environment,
    pub messaging: Messaging,
    pub last_interaction: DateTime<Utc>,
    pub memory: Memory,
    pub metrics: Metrics,
}

impl Metrics {
    pub fn decay(&mut self) {
        let now = Utc::now();
        let hours = (now - self.last_updated).num_minutes() as f32 / 60.0;
        self.trust = decay_param(self.trust, hours);
        self.intimacy = decay_param(self.intimacy, hours);
        self.energy = decay_param(self.energy, hours);
        self.can_send = self.trust >= 0.5 && self.intimacy >= 0.5 && self.energy >= 0.5;
        self.last_updated = now;
    }
}

pub fn load_user_data(path: &Path) -> UserData {
    let config = ConfigPaths::new();
    let example_path = Path::new("example.json");
    config.ensure_file_exists("json", example_path);

    if !path.exists() {
        return UserData {
            personality: Personality {
                kind: "positive".into(),
                strength: 0.8,
            },
            relationship: Relationship {
                trust: 0.2,
                intimacy: 0.6,
                curiosity: 0.5,
                threshold: 1.5,
            },
            environment: Environment {
                luck_today: 0.9,
                luck_history: vec![0.9, 0.9, 0.9],
                level: 1,
            },
            messaging: Messaging {
                enabled: true,
                schedule_time: Some("08:00".to_string()),
                decay_rate: 0.1,
                templates: vec![
                    "おはよう！今日もがんばろう！".to_string(),
                    "ねえ、話したいことがあるの。".to_string(),
                ],
                sent_today: false,
                last_sent_date: None,
            },
            last_interaction: Utc::now(),
            memory: Memory {
                recent_messages: vec![],
                long_term_notes: vec![],
            },
            metrics: Metrics {
                trust: 0.5,
                intimacy: 0.5,
                energy: 0.5,
                can_send: true,
                last_updated: Utc::now(),
            },
        };
    }

    let content = fs::read_to_string(path).expect("user.json の読み込みに失敗しました");
    serde_json::from_str(&content).expect("user.json のパースに失敗しました")
}

pub fn save_user_data(path: &Path, data: &UserData) {
    let content = serde_json::to_string_pretty(data).expect("user.json のシリアライズ失敗");
    fs::write(path, content).expect("user.json の書き込みに失敗しました");
}

pub fn update_metrics_decay() -> Metrics {
    let config = ConfigPaths::new();
    let path = config.base_dir.join("user.json");
    let mut data = load_user_data(&path);
    data.metrics.decay();
    save_user_data(&path, &data);
    data.metrics
}

fn decay_param(value: f32, hours: f32) -> f32 {
    let decay_rate = 0.05;
    (value * (1.0f32 - decay_rate).powf(hours)).clamp(0.0, 1.0)
}
