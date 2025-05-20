// src/commands/db.rs
use seahorse::{Command, Context};
use crate::utils::{load_config};
use crate::model::AiSystem;
use crate::config::ConfigPaths;

use rusqlite::Connection;
use std::fs;

pub fn save_cmd() -> Command {
    Command::new("save")
        .usage("save")
        .action(|_c: &Context| {
            let paths = ConfigPaths::new();

            let json_path = paths.data_file("json");
            let db_path = paths.data_file("db");

            let ai = load_config(json_path.to_str().unwrap());
            let conn = Connection::open(db_path).expect("DB接続失敗");

            ai.save_to_db(&conn).expect("DB保存失敗");
            println!("💾 DBに保存完了");
        })
}

pub fn export_cmd() -> Command {
    Command::new("export")
        .usage("export [output.json]")
        .action(|c: &Context| {
            let output_path = c.args.get(0).map(|s| s.as_str()).unwrap_or("output.json");

            let paths = ConfigPaths::new();
            let db_path = paths.data_file("db");

            let conn = Connection::open(db_path).expect("DB接続失敗");
            let ai = AiSystem::load_from_db(&conn).expect("DB読み込み失敗");

            let json = serde_json::to_string_pretty(&ai).expect("JSON変換失敗");
            fs::write(output_path, json).expect("ファイル書き込み失敗");

            println!("📤 JSONにエクスポート完了: {output_path}");
        })
}
