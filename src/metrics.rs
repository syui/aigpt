// src/metrics.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    pub trust: f32,
    pub intimacy: f32,
    pub energy: f32,
    pub can_send: bool,
    pub last_updated: DateTime<Utc>,
}

impl Metrics {
    fn default() -> Self {
        Self {
            trust: 0.5,
            intimacy: 0.5,
            energy: 0.5,
            last_updated: chrono::Utc::now(),
            can_send: true,
        }
    }
    /// パラメータの減衰処理を行い、can_sendを更新する
    pub fn decay(&mut self) {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_updated);
        let hours = elapsed.num_minutes() as f32 / 60.0;

        self.trust = decay_param(self.trust, hours);
        self.intimacy = decay_param(self.intimacy, hours);
        self.energy = decay_param(self.energy, hours);

        self.last_updated = now;
        self.can_send = self.trust >= 0.5 && self.intimacy >= 0.5 && self.energy >= 0.5;
    }

    /// JSONからMetricsを読み込み、減衰し、保存して返す
    pub fn load_and_decay(path: &Path) -> Self {
        let mut metrics = if path.exists() {
            let content = fs::read_to_string(path).expect("metrics.jsonの読み込みに失敗しました");
            serde_json::from_str(&content).expect("JSONパース失敗")
        } else {
            println!("⚠️ metrics.json が存在しないため、新しく作成します。");
            Metrics::default()
        };

        metrics.decay();
        metrics.save(path);
        metrics
    }

    /// Metricsを保存する
    pub fn save(&self, path: &Path) {
        let data = serde_json::to_string_pretty(self).expect("JSON変換失敗");
        fs::write(path, data).expect("metrics.jsonの書き込みに失敗しました");
    }
}

/// 単一のパラメータを減衰させる
fn decay_param(value: f32, hours: f32) -> f32 {
    let decay_rate = 0.01; // 時間ごとの減衰率
    (value * (1.0f32 - decay_rate).powf(hours)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_decay_behavior() {
        let mut metrics = Metrics {
            trust: 1.0,
            intimacy: 1.0,
            energy: 1.0,
            can_send: true,
            last_updated: Utc::now() - Duration::hours(12),
        };
        metrics.decay();
        assert!(metrics.trust < 1.0);
        assert!(metrics.can_send); // 減衰後でも0.5以上あるならtrue
    }
} 

pub fn load_metrics(path: &Path) -> Metrics {
    Metrics::load_and_decay(path)
}

pub fn save_metrics(metrics: &Metrics, path: &Path) {
    metrics.save(path)
}

pub fn update_metrics_decay(metrics: &mut Metrics) {
    metrics.decay()
}
