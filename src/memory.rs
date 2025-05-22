// src/memory.rs
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self};
//use std::fs::{self, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::{fs::File};
//use std::{env, fs::File};

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub message: String,
}

pub fn log_message(base_dir: &PathBuf, sender: &str, message: &str) {
    let now_utc = Utc::now();
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let mut file_path = base_dir.clone();
    file_path.push("memory");
    let _ = fs::create_dir_all(&file_path);
    file_path.push(format!("{}.json", date_str));

    let new_entry = MemoryEntry {
        timestamp: now_utc,
        sender: sender.to_string(),
        message: message.to_string(),
    };

    let mut entries = if file_path.exists() {
        let file = File::open(&file_path).expect("💥 メモリファイルの読み込み失敗");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    entries.push(new_entry);

    let file = File::create(&file_path).expect("💥 メモリファイルの書き込み失敗");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &entries).expect("💥 JSONの書き込み失敗");
}

// 利用例（ask_chatの中）
// log_message(&config.base_dir, "user", question);
// log_message(&config.base_dir, "ai", &response);
