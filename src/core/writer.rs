use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{config, reader};

fn generate_tid() -> String {
    // ATProto TID: 64-bit integer as 13 base32-sortable chars
    // bit 63: always 0 (sign), bits 62..10: timestamp (microseconds), bits 9..0: clock_id
    const CHARSET: &[u8] = b"234567abcdefghijklmnopqrstuvwxyz";
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;
    let v = (micros << 10) & 0x7FFFFFFFFFFFFFFF;
    let mut tid = String::with_capacity(13);
    for i in (0..13).rev() {
        let idx = ((v >> (i * 5)) & 0x1f) as usize;
        tid.push(CHARSET[idx] as char);
    }
    tid
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
