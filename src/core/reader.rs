use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aigpt")
}

pub fn read_core() -> Result<String> {
    let path = config_dir().join("core.md");
    fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))
}

pub fn read_memory() -> Result<String> {
    let path = config_dir().join("memory.md");
    match fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e).with_context(|| format!("Failed to read {}", path.display())),
    }
}
