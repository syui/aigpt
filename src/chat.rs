// src/chat.rs

use seahorse::Context;
use std::process::Command;
//use std::env;
use crate::config::ConfigPaths;

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

    let mut command = Command::new(python_path);
    command.arg(script_path).arg(question);

    if let Some(host) = ollama_host {
        command.env("OLLAMA_HOST", host);
    }
    if let Some(model) = ollama_model {
        command.env("OLLAMA_MODEL", model);
    }

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
