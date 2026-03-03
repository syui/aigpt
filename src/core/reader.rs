use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

use crate::core::config::{self, COLLECTION_CORE, COLLECTION_MEMORY};

pub fn read_core() -> Result<Value> {
    let cfg = config::load();
    let path = config::record_path(&cfg, COLLECTION_CORE, "self");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let record: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(record)
}

pub fn read_memory_all() -> Result<Vec<Value>> {
    let cfg = config::load();
    let dir = config::collection_dir(&cfg, COLLECTION_MEMORY);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("Failed to read {}", dir.display())),
    };
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort_by_key(|e| e.file_name());

    let mut records = Vec::with_capacity(files.len());
    for entry in &files {
        let path = entry.path();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let record: Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        records.push(record);
    }
    Ok(records)
}

pub fn memory_count() -> usize {
    let cfg = config::load();
    let dir = config::collection_dir(&cfg, COLLECTION_MEMORY);
    fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .count()
        })
        .unwrap_or(0)
}
