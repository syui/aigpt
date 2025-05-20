// src/commands/db.rs
use seahorse::{Command, Context};
use crate::utils::load_config;
use crate::model::AiSystem;
use rusqlite::Connection;
use std::fs;

pub fn save_cmd() -> Command {
    Command::new("save")
        .usage("save")
        .action(|_c: &Context| {
            let ai = load_config("config/config.json");
            let conn = Connection::open("config/ai_state.db").expect("DB接続失敗");
            ai.save_to_db(&conn).expect("DB保存失敗");
            println!("💾 DBに保存完了");
        })
}

pub fn export_cmd() -> Command {
    Command::new("export")
        .usage("export [output.json]")
        .action(|c: &Context| {
            let path = c.args.get(0).map(|s| s.as_str()).unwrap_or("output.json");
            let conn = Connection::open("config/ai_state.db").expect("DB接続失敗");
            let ai = AiSystem::load_from_db(&conn).expect("DB読み込み失敗");

            let json = serde_json::to_string_pretty(&ai).expect("JSON変換失敗");
            fs::write(path, json).expect("ファイル書き込み失敗");

            println!("📤 JSONにエクスポート完了: {path}");
        })
}
