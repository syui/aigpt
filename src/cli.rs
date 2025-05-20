//src/cli.rs
use seahorse::{App, Command, Context};

use crate::model::{AiSystem};
use std::fs;

use crate::agent::AIState;
use chrono::{Duration, Local};

fn load_config(path: &str) -> AiSystem {
    let data = fs::read_to_string(path).expect("JSON読み込み失敗");
    serde_json::from_str(&data).expect("JSONパース失敗")
}

fn save_config(path: &str, ai: &AiSystem) {
    let json = serde_json::to_string_pretty(&ai).expect("JSONシリアライズ失敗");
    fs::write(path, json).expect("JSON保存失敗");
}

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

            let path = "config/config.json";
            let mut ai = load_config(path);

            match field.as_str() {
                "trust" => ai.relationship.trust = value,
                "intimacy" => ai.relationship.intimacy = value,
                "curiosity" => ai.relationship.curiosity = value,
                _ => {
                    eprintln!("trust / intimacy / curiosity のいずれかを指定してください");
                    std::process::exit(1);
                }
            }

            save_config(path, &ai);
            println!("✅ {field} を {value} に更新しました");
        });

    let show_cmd = Command::new("show")
        .usage("show")
        .action(|_c: &Context| {
            let ai = load_config("config/config.json");
            println!("🧠 現在のAI状態:\n{:#?}", ai);
        });

    let talk_cmd = Command::new("talk")
        .usage("talk")
        .action(|_c: &Context| {
            let ai = load_config("config/config.json");

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
}
