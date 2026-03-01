use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::config;

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

/// Save a single memory element as a new TID file
pub fn save_memory(content: &str) -> Result<()> {
    let cfg = config::load();
    let did = cfg.did.clone().unwrap_or_else(|| "self".to_string());
    let tid = generate_tid();
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

/// Delete all memory files, then write new ones from the given items
pub fn compress_memory(items: &[String]) -> Result<()> {
    let cfg = config::load();
    let did = cfg.did.clone().unwrap_or_else(|| "self".to_string());
    let dir = config::collection_dir(&cfg, "ai.syui.gpt.memory");

    // delete all existing memory files
    if dir.exists() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|ext| ext == "json") {
                fs::remove_file(entry.path())?;
            }
        }
    }

    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create {}", dir.display()))?;

    // write each item as a new TID file
    for item in items {
        let micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        // small delay to ensure unique TIDs
        std::thread::sleep(std::time::Duration::from_micros(1));

        let v = (micros << 10) & 0x7FFFFFFFFFFFFFFF;
        const CHARSET: &[u8] = b"234567abcdefghijklmnopqrstuvwxyz";
        let mut tid = String::with_capacity(13);
        for i in (0..13).rev() {
            let idx = ((v >> (i * 5)) & 0x1f) as usize;
            tid.push(CHARSET[idx] as char);
        }

        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let record = json!({
            "uri": format!("at://{}/ai.syui.gpt.memory/{}", did, tid),
            "value": {
                "$type": "ai.syui.gpt.memory",
                "did": did,
                "content": {
                    "$type": "ai.syui.gpt.memory#markdown",
                    "text": item
                },
                "createdAt": now
            }
        });

        let path = dir.join(format!("{}.json", tid));
        let json_str = serde_json::to_string_pretty(&record)?;
        fs::write(&path, json_str)
            .with_context(|| format!("Failed to write {}", path.display()))?;
    }

    Ok(())
}
