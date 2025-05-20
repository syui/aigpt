// src/commands/scheduler.rs

use seahorse::{Command, Context};
use std::thread;
use std::time::Duration;
use chrono::Local;

pub fn scheduler_cmd() -> Command {
    Command::new("scheduler")
        .usage("scheduler [interval_sec]")
        .alias("s")
        .action(|c: &Context| {
            let interval = c.args.get(0)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60); // デフォルト: 60秒ごと

            println!("⏳ スケジューラー開始（{interval}秒ごと）...");

            loop {
                let now = Local::now();
                println!("🔁 タスク実行中: {}", now.format("%Y-%m-%d %H:%M:%S"));
                
                // ここで talk_cmd や save_cmd の内部処理を呼ぶ感じ
                // たとえば load_config → AI更新 → print とか

                thread::sleep(Duration::from_secs(interval));
            }
        })
}
