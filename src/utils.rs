// src/utils.rs
use std::fs;
use crate::model::AiSystem;

pub fn load_config(path: &str) -> AiSystem {
    let data = fs::read_to_string(path).expect("JSON読み込み失敗");
    serde_json::from_str(&data).expect("JSONパース失敗")
}

pub fn save_config(path: &str, ai: &AiSystem) {
    let json = serde_json::to_string_pretty(&ai).expect("JSONシリアライズ失敗");
    fs::write(path, json).expect("JSON保存失敗");
}
