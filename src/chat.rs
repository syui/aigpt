// src/chat.rs
use std::fs;
use std::process::Command;
use serde::Deserialize;
use seahorse::Context;
use crate::config::ConfigPaths;
use crate::metrics::{load_user_data, save_user_data, update_metrics_decay};
//use std::process::Stdio;
//use std::io::Write;
//use std::time::Duration;
//use std::net::TcpStream;

#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    OpenAI,
    Ollama,
    MCP,
}

impl Provider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(Provider::OpenAI),
            "ollama" => Some(Provider::Ollama),
            "mcp" => Some(Provider::MCP),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Ollama => "ollama",
            Provider::MCP => "mcp",
        }
    }
}

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

pub fn ask_chat(c: &Context, question: &str) -> Option<String> {
    let config = ConfigPaths::new();
    let base_dir = config.base_dir.join("mcp");
    let user_path = config.base_dir.join("user.json");

    let mut user = load_user_data(&user_path);
    user.metrics = update_metrics_decay();

    // 各種オプション
    let ollama_host = c.string_flag("host").ok();
    let ollama_model = c.string_flag("model").ok();
    let provider_str = c.string_flag("provider").unwrap_or_else(|_| "ollama".to_string());
    let provider = Provider::from_str(&provider_str).unwrap_or(Provider::Ollama);
    let api_key = c.string_flag("api-key").ok().or_else(load_openai_api_key);

    println!("🔍 使用プロバイダー: {}", provider.as_str());

    match provider {
        Provider::MCP => {
            let client = reqwest::blocking::Client::new();
            let url = std::env::var("MCP_URL").unwrap_or("http://127.0.0.1:5000/chat".to_string());
            let res = client.post(url)
                .json(&serde_json::json!({"message": question}))
                .send();

            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let json: serde_json::Value = resp.json().ok()?;
                        let text = json.get("response")?.as_str()?.to_string();
                        user.metrics.intimacy += 0.01;
                        user.metrics.last_updated = chrono::Utc::now();
                        save_user_data(&user_path, &user);
                        Some(text)
                    } else {
                        eprintln!("❌ MCPエラー: HTTP {}", resp.status());
                        None
                    }
                }
                Err(e) => {
                    eprintln!("❌ MCP接続失敗: {}", e);
                    None
                }
            }
        }
        _ => {
            // Python 実行パス
            let python_path = if cfg!(target_os = "windows") {
                base_dir.join(".venv/Scripts/mcp.exe")
            } else {
                base_dir.join(".venv/bin/mcp")
            };

            let mut command = Command::new(python_path);
            command.arg("ask").arg(question);

            if let Some(host) = ollama_host {
                command.env("OLLAMA_HOST", host);
            }
            if let Some(model) = ollama_model {
                command.env("OLLAMA_MODEL", model.clone());
                command.env("OPENAI_MODEL", model);
            }
            command.env("PROVIDER", provider.as_str());

            if let Some(key) = api_key {
                command.env("OPENAI_API_KEY", key);
            }

            let output = command.output().expect("❌ MCPチャットスクリプトの実行に失敗しました");

            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout).to_string();
                user.metrics.intimacy += 0.01;
                user.metrics.last_updated = chrono::Utc::now();
                save_user_data(&user_path, &user);

                Some(response)
            } else {
                eprintln!(
                    "❌ 実行エラー: {}\n{}",
                    String::from_utf8_lossy(&output.stderr),
                    String::from_utf8_lossy(&output.stdout),
                );
                None
            }
        }
    }
}
