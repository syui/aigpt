// src/cli.rs
use std::path::{Path};
use chrono::{Duration, Local};
use rusqlite::Connection;

use seahorse::{App, Command, Context};
use crate::utils::{load_config, save_config};
use crate::commands::db::{save_cmd, export_cmd};
use crate::config::ConfigPaths;
use crate::agent::AIState;

pub fn cli_app() -> App {
    let set_cmd = Command::new("set")
        .usage("set [trust|intimacy|curiosity] [value]")
        .action(|c: &Context| {
            if c.args.len() != 2 {
                eprintln!("Usage: set [trust|intimacy|curiosity] [value]");
                std::process::exit(1);
            }

            let field = &c.args[0];
            let value: f32 = c.args[1].parse().unwrap_or_else(|_| {
                eprintln!("数値で入力してください");
                std::process::exit(1);
            });

            // ConfigPathsを使って設定ファイルのパスを取得
            let config_paths = ConfigPaths::new();
            let json_path = config_paths.data_file("json");
            // まだ user.json がない場合、example.json をコピー
            config_paths.ensure_file_exists("json", Path::new("example.json"));
            let db_path = config_paths.data_file("db");
            let mut ai = load_config(json_path.to_str().unwrap());

            match field.as_str() {
                "trust" => ai.relationship.trust = value,
                "intimacy" => ai.relationship.intimacy = value,
                "curiosity" => ai.relationship.curiosity = value,
                _ => {
                    eprintln!("trust / intimacy / curiosity のいずれかを指定してください");
                    std::process::exit(1);
                }
            }
            save_config(json_path.to_str().unwrap(), &ai);

            let conn = Connection::open(db_path.to_str().unwrap()).expect("DB接続失敗");
            ai.save_to_db(&conn).expect("DB保存失敗");

            println!("✅ {field} を {value} に更新しました");
        });

    let show_cmd = Command::new("show")
        .usage("show")
        .action(|_c: &Context| {
            // ConfigPathsを使って設定ファイルのパスを取得
            let config_paths = ConfigPaths::new();
            let ai = load_config(config_paths.data_file("json").to_str().unwrap());
            println!("🧠 現在のAI状態:\n{:#?}", ai);
        });

    let talk_cmd = Command::new("talk")
        .usage("talk")
        .action(|_c: &Context| {
            let config_paths = ConfigPaths::new();
            let ai = load_config(config_paths.data_file("json").to_str().unwrap());

            let now = Local::now().naive_local();
            let mut state = AIState {
                relation_score: 80.0,
                previous_score: 80.0,
                decay_rate: ai.messaging.decay_rate,
                sensitivity: ai.personality.strength,
                message_threshold: 5.0,
                last_message_time: now - Duration::days(4),
            };

            state.update(now);

            if state.should_talk() {
                println!("💬 AI発話: {}", state.generate_message());
            } else {
                println!("🤫 今日は静かにしているみたい...");
            }
        });

    App::new("aigpt")
        .version("0.1.0")
        .description("AGE system CLI controller")
        .author("syui")
        .command(set_cmd)
        .command(show_cmd)
        .command(talk_cmd)
        .command(save_cmd())
        .command(export_cmd())
}
