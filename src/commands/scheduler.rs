// src/commands/scheduler.rs
use seahorse::{Command, Context};
use std::thread;
use std::time::Duration;
use chrono::{Local, Utc, Timelike};
use crate::metrics::{load_user_data, save_user_data};
use crate::config::ConfigPaths;
use crate::chat::ask_chat;
use rand::prelude::*;
use rand::rng;

fn send_scheduled_message() {
    let config = ConfigPaths::new();
    let user_path = config.data_file("json");
    let mut user = load_user_data(&user_path);

    if !user.metrics.can_send {
        println!("🚫 送信条件を満たしていないため、スケジュール送信スキップ");
        return;
    }

    // 日付の比較（1日1回制限）
    let today = Local::now().format("%Y-%m-%d").to_string();
    if let Some(last_date) = &user.messaging.last_sent_date {
        if last_date != &today {
            user.messaging.sent_today = false;
        }
    } else {
        user.messaging.sent_today = false;
    }

    if user.messaging.sent_today {
        println!("🔁 本日はすでに送信済みです: {}", today);
        return;
    }

    if let Some(schedule_str) = &user.messaging.schedule_time {
        let now = Local::now();
        let target: Vec<&str> = schedule_str.split(':').collect();

        if target.len() != 2 {
            println!("⚠️ schedule_time形式が無効です: {}", schedule_str);
            return;
        }

        let (sh, sm) = (target[0].parse::<u32>(), target[1].parse::<u32>());
        if let (Ok(sh), Ok(sm)) = (sh, sm) {
            if now.hour() == sh && now.minute() == sm {
                if let Some(msg) = user.messaging.templates.choose(&mut rng()) {
                    println!("💬 自動送信メッセージ: {}", msg);
                    let dummy_context = Context::new(vec![], None, "".to_string());
                    ask_chat(&dummy_context, msg);
                    user.metrics.intimacy += 0.03;

                    // 送信済みのフラグ更新
                    user.messaging.sent_today = true;
                    user.messaging.last_sent_date = Some(today);

                    save_user_data(&user_path, &user);
                }
            }
        }
    }
}


pub fn scheduler_cmd() -> Command {
    Command::new("scheduler")
        .usage("scheduler [interval_sec]")
        .alias("s")
        .description("定期的に送信条件をチェックし、自発的なメッセージ送信を試みる")
        .action(|c: &Context| {
            let interval = c.args.get(0)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(3600); // デフォルト: 1時間（テストしやすく）

            println!("⏳ スケジューラー開始（{}秒ごと）...", interval);

            loop {
                let config = ConfigPaths::new();
                let user_path = config.data_file("json");
                let mut user = load_user_data(&user_path);

                let now = Utc::now();
                let elapsed = now.signed_duration_since(user.metrics.last_updated);
                let hours = elapsed.num_minutes() as f32 / 60.0;

                let speed_factor = if hours > 48.0 {
                    2.0
                } else if hours > 24.0 {
                    1.5
                } else {
                    1.0
                };

                user.metrics.trust = (user.metrics.trust - 0.01 * speed_factor).clamp(0.0, 1.0);
                user.metrics.intimacy = (user.metrics.intimacy - 0.01 * speed_factor).clamp(0.0, 1.0);
                user.metrics.energy = (user.metrics.energy - 0.01 * speed_factor).clamp(0.0, 1.0);

                user.metrics.can_send =
                    user.metrics.trust >= 0.5 &&
                    user.metrics.intimacy >= 0.5 &&
                    user.metrics.energy >= 0.5;

                user.metrics.last_updated = now;

                if user.metrics.can_send {
                    println!("💡 AIメッセージ送信条件を満たしています（信頼:{:.2}, 親密:{:.2}, エネルギー:{:.2}）",
                        user.metrics.trust,
                        user.metrics.intimacy,
                        user.metrics.energy
                    );
                    send_scheduled_message();
                } else {
                    println!("🤫 条件未達成のため送信スキップ: trust={:.2}, intimacy={:.2}, energy={:.2}",
                        user.metrics.trust,
                        user.metrics.intimacy,
                        user.metrics.energy
                    );
                }
                
                save_user_data(&user_path, &user);
                thread::sleep(Duration::from_secs(interval));
            }
        })
}

