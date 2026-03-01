use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use chrono::Utc;

pub struct Config {
    pub path: Option<String>,
    pub did: Option<String>,
    pub handle: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    bot: Option<BotConfig>,
}

#[derive(Debug, Deserialize)]
struct BotConfig {
    did: Option<String>,
    handle: Option<String>,
    path: Option<String>,
}

pub fn config_file() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ai.syui.gpt")
        .join("config.json")
}

fn expand_path(p: &str) -> PathBuf {
    if p.starts_with("~/") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(&p[2..])
    } else {
        PathBuf::from(p)
    }
}

pub fn load() -> Config {
    let cfg_path = config_file();
    if let Ok(content) = fs::read_to_string(&cfg_path) {
        if let Ok(file) = serde_json::from_str::<ConfigFile>(&content) {
            if let Some(bot) = file.bot {
                return Config {
                    path: bot.path,
                    did: bot.did,
                    handle: bot.handle,
                };
            }
        }
    }

    Config {
        path: None,
        did: None,
        handle: None,
    }
}

pub fn init() {
    let cfg_path = config_file();
    if !cfg_path.exists() {
        if let Some(parent) = cfg_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let default_cfg = json!({
            "bot": {
                "did": null,
                "handle": null,
                "path": null
            }
        });
        let _ = fs::write(&cfg_path, serde_json::to_string_pretty(&default_cfg).unwrap());
    }

    let cfg = load();
    let core_path = record_path(&cfg, "ai.syui.gpt.core", "self");
    if !core_path.exists() {
        if let Some(parent) = core_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let did = cfg.did.clone().unwrap_or_else(|| "self".to_string());
        let handle = cfg.handle.clone().unwrap_or_else(|| "self".to_string());
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let core_record = json!({
            "uri": format!("at://{}/ai.syui.gpt.core/self", did),
            "value": {
                "$type": "ai.syui.gpt.core",
                "did": did,
                "handle": handle,
                "content": {
                    "$type": "ai.syui.gpt.core#markdown",
                    "text": ""
                },
                "createdAt": now
            }
        });
        let _ = fs::write(&core_path, serde_json::to_string_pretty(&core_record).unwrap());
    }

    let memory_dir = collection_dir(&cfg, "ai.syui.gpt.memory");
    let _ = fs::create_dir_all(&memory_dir);
}

pub fn base_dir(cfg: &Config) -> PathBuf {
    match &cfg.path {
        Some(p) => expand_path(p),
        None => dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai.syui.gpt"),
    }
}

pub fn identity(cfg: &Config) -> String {
    if cfg!(windows) {
        cfg.handle.clone().unwrap_or_else(|| "self".to_string())
    } else {
        cfg.did.clone().unwrap_or_else(|| "self".to_string())
    }
}

/// $cfg/{did|handle}/{collection}/{rkey}.json
pub fn record_path(cfg: &Config, collection: &str, rkey: &str) -> PathBuf {
    base_dir(cfg)
        .join(identity(cfg))
        .join(collection)
        .join(format!("{}.json", rkey))
}

/// $cfg/{did|handle}/{collection}/
pub fn collection_dir(cfg: &Config, collection: &str) -> PathBuf {
    base_dir(cfg).join(identity(cfg)).join(collection)
}
