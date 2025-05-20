//src/logic.rs
use crate::model::AiSystem;

#[allow(dead_code)]
pub fn should_send(ai: &AiSystem) -> bool {
    let r = &ai.relationship;
    let env = &ai.environment;
    let score = r.trust + r.intimacy + r.curiosity;
    let relationship_ok = score >= r.threshold;
    let luck_ok = env.luck_today > 0.5;

    ai.messaging.enabled && relationship_ok && luck_ok
}
