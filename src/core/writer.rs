use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ai.syui.gpt")
}

pub fn save_memory(content: &str) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create {}", dir.display()))?;
    let path = dir.join("memory.md");
    fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))
}
