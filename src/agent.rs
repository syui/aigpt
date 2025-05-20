use chrono::{NaiveDateTime};

#[allow(dead_code)]
#[derive(Debug)]
pub struct AIState {
    pub relation_score: f32,
    pub previous_score: f32,
    pub decay_rate: f32,
    pub sensitivity: f32,
    pub message_threshold: f32,
    pub last_message_time: NaiveDateTime,
}

#[allow(dead_code)]
impl AIState {
    pub fn update(&mut self, now: NaiveDateTime) {
        let days_passed = (now - self.last_message_time).num_days() as f32;
        let decay = self.decay_rate * days_passed;
        self.previous_score = self.relation_score;
        self.relation_score -= decay;
        self.relation_score = self.relation_score.clamp(0.0, 100.0);
    }

    pub fn should_talk(&self) -> bool {
        let delta = self.previous_score - self.relation_score;
        delta > self.message_threshold && self.sensitivity > 0.5
    }

    pub fn generate_message(&self) -> String {
        match self.relation_score as i32 {
            80..=100 => "ふふっ、最近どうしてる？会いたくなっちゃった！".to_string(),
            60..=79 => "ちょっとだけ、さみしかったんだよ？".to_string(),
            40..=59 => "えっと……話せる時間ある？".to_string(),
            _ => "ううん、もしかして私のこと、忘れちゃったのかな……".to_string(),
        }
    }
}
