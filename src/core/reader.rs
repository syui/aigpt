use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

use crate::core::config;

pub fn read_core() -> Result<Value> {
    let cfg = config::load();
    let path = config::record_path(&cfg, "ai.syui.gpt.core", "self");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let record: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(record)
}

pub fn read_memory() -> Result<Option<Value>> {
    let cfg = config::load();
    let dir = config::collection_dir(&cfg, "ai.syui.gpt.memory");
    if !dir.exists() {
        return Ok(None);
    }
    let mut files: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    if files.is_empty() {
        return Ok(None);
    }
    files.sort_by_key(|e| e.file_name());
    let latest = files.last().unwrap().path();
    let content = fs::read_to_string(&latest)
        .with_context(|| format!("Failed to read {}", latest.display()))?;
    let record: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", latest.display()))?;
    Ok(Some(record))
}
