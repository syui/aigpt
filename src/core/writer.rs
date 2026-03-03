use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::config::{self, COLLECTION_MEMORY};

static TID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_tid() -> String {
    const CHARSET: &[u8] = b"234567abcdefghijklmnopqrstuvwxyz";
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;
    let clock_id = TID_COUNTER.fetch_add(1, Ordering::Relaxed) & 0x3FF;
    let v = ((micros << 10) | clock_id) & 0x7FFFFFFFFFFFFFFF;
    let mut tid = String::with_capacity(13);
    for i in (0..13).rev() {
        let idx = ((v >> (i * 5)) & 0x1f) as usize;
        tid.push(CHARSET[idx] as char);
    }
    tid
}

fn build_memory_record(did: &str, tid: &str, text: &str) -> Value {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    json!({
        "uri": format!("at://{}/{}/{}", did, COLLECTION_MEMORY, tid),
        "value": {
            "$type": COLLECTION_MEMORY,
            "did": did,
            "content": {
                "$type": format!("{}#markdown", COLLECTION_MEMORY),
                "text": text
            },
            "createdAt": now
        }
    })
}

/// Save a single memory element as a new TID file
pub fn save_memory(content: &str) -> Result<()> {
    let cfg = config::load();
    let tid = generate_tid();
    let record = build_memory_record(cfg.did(), &tid, content);

    let dir = config::collection_dir(&cfg, COLLECTION_MEMORY);
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
    let dir = config::collection_dir(&cfg, COLLECTION_MEMORY);

    // delete all existing memory files
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|ext| ext == "json") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create {}", dir.display()))?;

    for item in items {
        let tid = generate_tid();
        let record = build_memory_record(cfg.did(), &tid, item);
        let path = dir.join(format!("{}.json", tid));
        let json_str = serde_json::to_string_pretty(&record)?;
        fs::write(&path, json_str)
            .with_context(|| format!("Failed to write {}", path.display()))?;
    }

    Ok(())
}
