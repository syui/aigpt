use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{config, reader};

fn generate_tid() -> String {
    const CHARSET: &[u8] = b"234567abcdefghijklmnopqrstuvwxyz";
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;
    let mut tid = [0u8; 13];
    let mut v = micros;
    for i in (0..13).rev() {
        tid[i] = CHARSET[(v & 0x1f) as usize];
        v >>= 5;
    }
    String::from_utf8(tid.to_vec()).unwrap()
}

fn next_version() -> u64 {
    match reader::read_memory() {
        Ok(Some(record)) => record["value"]["version"].as_u64().unwrap_or(0) + 1,
        _ => 1,
    }
}

pub fn save_memory(content: &str) -> Result<()> {
    let cfg = config::load();
    let did = cfg.did.clone().unwrap_or_else(|| "self".to_string());
    let tid = generate_tid();
    let version = next_version();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let record = json!({
        "uri": format!("at://{}/ai.syui.gpt.memory/{}", did, tid),
        "value": {
            "$type": "ai.syui.gpt.memory",
            "did": did,
            "content": {
                "$type": "ai.syui.gpt.memory#markdown",
                "text": content
            },
            "version": version,
            "createdAt": now
        }
    });

    let dir = config::collection_dir(&cfg, "ai.syui.gpt.memory");
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create {}", dir.display()))?;
    let path = dir.join(format!("{}.json", tid));
    let json_str = serde_json::to_string_pretty(&record)?;
    fs::write(&path, json_str)
        .with_context(|| format!("Failed to write {}", path.display()))
}
