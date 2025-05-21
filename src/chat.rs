// src/chat.rs

use seahorse::Context;
use std::process::Command;
use crate::config::ConfigPaths;

#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    OpenAI,
    Ollama,
}

impl Provider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(Provider::OpenAI),
            "ollama" => Some(Provider::Ollama),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Ollama => "ollama",
        }
    }
}

use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct OpenAIKey {
    token: String,
}

fn load_openai_api_key() -> Option<String> {
    let config = ConfigPaths::new();
    let path = config.base_dir.join("openai.json");
    let data = fs::read_to_string(path).ok()?;
    let parsed: OpenAIKey = serde_json::from_str(&data).ok()?;
    Some(parsed.token)
}

pub fn ask_chat(c: &Context, question: &str) {
    let config = ConfigPaths::new();
    let base_dir = config.base_dir.join("mcp");
    let script_path = base_dir.join("scripts/ask.py");

    let python_path = if cfg!(target_os = "windows") {
        base_dir.join(".venv/Scripts/python.exe")
    } else {
        base_dir.join(".venv/bin/python")
    };

    let ollama_host = c.string_flag("host").ok();
    let ollama_model = c.string_flag("model").ok();
    let api_key = c.string_flag("api-key").ok()
        .or_else(|| load_openai_api_key());

    use crate::chat::Provider;

    let provider_str = c.string_flag("provider").unwrap_or_else(|_| "ollama".to_string());
    let provider = Provider::from_str(&provider_str).unwrap_or(Provider::Ollama);

    println!("🔍 使用プロバイダー: {}", provider.as_str());

    // 🛠️ command の定義をここで行う
    let mut command = Command::new(python_path);
    command.arg(script_path).arg(question);

    // ✨ 環境変数をセット
    command.env("PROVIDER", provider.as_str());

    if let Some(host) = ollama_host {
        command.env("OLLAMA_HOST", host);
    }
    if let Some(model) = ollama_model {
        command.env("OLLAMA_MODEL", model);
    }
    if let Some(api_key) = api_key {
        command.env("OPENAI_API_KEY", api_key);
    }

    // 🔁 実行
    let output = command
        .output()
        .expect("❌ MCPチャットスクリプトの実行に失敗しました");

    if output.status.success() {
        println!("💬 {}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!(
            "❌ 実行エラー: {}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout),
        );
    }
}
