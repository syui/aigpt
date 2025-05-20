use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Write};
use std::time::{SystemTime, UNIX_EPOCH};

mod model;
use model::RelationalAutonomousAI;

fn load_config(path: &str) -> std::io::Result<RelationalAutonomousAI> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: RelationalAutonomousAI = serde_json::from_reader(reader)?;
    Ok(config)
}

fn save_config(config: &RelationalAutonomousAI, path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    let json = serde_json::to_string_pretty(config)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn should_send_message(config: &RelationalAutonomousAI) -> bool {
    // 簡易な送信条件: relationshipが高く、daily_luckが0.8以上
    config.core_components.relationship.parameters.contains(&"trust".to_string())
        && config.core_components.environment.daily_luck.range[1] >= 0.8
}

fn main() -> std::io::Result<()> {
    let path = "config.json";

    let mut config = load_config(path)?;

    if should_send_message(&config) {
        println!("💌 メッセージを送信できます: {:?}", config.core_components.personality.r#type);

        // ステート変化の例: メッセージ送信後に記録用トランジションを追加
        config.core_components.state_transition.transitions.push("message_sent".to_string());

        save_config(&config, path)?;
    } else {
        println!("😶 まだ送信条件に達していません。");
    }

    Ok(())
}
