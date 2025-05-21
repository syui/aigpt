// src/commands/mcp.rs

use std::fs;
use std::path::{PathBuf};
use std::process::Command as OtherCommand;
use serde_json::json;
use seahorse::{Command, Context, Flag, FlagType};

use crate::chat::ask_chat;
use crate::git::{git_init, git_status};
use crate::config::ConfigPaths;

pub fn mcp_setup() {
    let config = ConfigPaths::new();
    let dest_dir = config.base_dir.join("mcp");
    let repo_url = "https://github.com/microsoft/MCP.git";
    println!("📁 MCP ディレクトリ: {}", dest_dir.display());

   // 1. git clone（もしまだなければ）
    if !dest_dir.exists() {
        let status = OtherCommand::new("git")
            .args(&["clone", repo_url, dest_dir.to_str().unwrap()])
            .status()
            .expect("git clone に失敗しました");
        assert!(status.success(), "git clone 実行時にエラーが発生しました");
    }

    let asset_base = PathBuf::from("mcp");
    let files_to_copy = vec![
        "cli.py",
        "setup.py",
        "scripts/ask.py",
        "scripts/context_loader.py",
        "scripts/prompt_template.py",
    ];

    for rel_path in files_to_copy {
        let src = asset_base.join(rel_path);
        let dst = dest_dir.join(rel_path);
        if let Some(parent) = dst.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::copy(&src, &dst) {
            eprintln!("❌ コピー失敗: {} → {}: {}", src.display(), dst.display(), e);
        } else {
            println!("✅ コピー: {} → {}", src.display(), dst.display());
        }
    }

    // venvの作成
    let venv_path = dest_dir.join(".venv");
    if !venv_path.exists() {
        println!("🐍 仮想環境を作成しています...");
        let output = OtherCommand::new("python3")
            .args(&["-m", "venv", ".venv"])
            .current_dir(&dest_dir)
            .output()
            .expect("venvの作成に失敗しました");

        if !output.status.success() {
            eprintln!("❌ venv作成エラー: {}", String::from_utf8_lossy(&output.stderr));
            return;
        }
    }

    // `pip install -e .` を仮想環境で実行
    let pip_path = if cfg!(target_os = "windows") {
        dest_dir.join(".venv/Scripts/pip.exe").to_string_lossy().to_string()
    } else {
        dest_dir.join(".venv/bin/pip").to_string_lossy().to_string()
    };

    println!("📦 必要なパッケージをインストールしています...");
    let output = OtherCommand::new(&pip_path)
        .arg("install")
        .arg("openai")
        .current_dir(&dest_dir)
        .output()
        .expect("pip install に失敗しました");

    if !output.status.success() {
        eprintln!(
            "❌ pip エラー: {}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        return;
    }

    println!("📦 pip install -e . を実行します...");
    let output = OtherCommand::new(&pip_path)
        .arg("install")
        .arg("-e")
        .arg(".")
        .current_dir(&dest_dir)
        .output()
        .expect("pip install に失敗しました");

    if output.status.success() {
        println!("🎉 MCP セットアップが完了しました！");
    } else {
        eprintln!(
            "❌ pip エラー: {}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

fn set_api_key_cmd() -> Command {
    Command::new("set-api")
        .description("OpenAI APIキーを設定")
        .usage("mcp set-api --api <API_KEY>")
        .flag(Flag::new("api", FlagType::String).description("OpenAI APIキー").alias("a"))
        .action(|c: &Context| {
            if let Ok(api_key) = c.string_flag("api") {
                let config = ConfigPaths::new();
                let path = config.base_dir.join("openai.json");
                let json_data = json!({ "token": api_key });

                if let Err(e) = fs::write(&path, serde_json::to_string_pretty(&json_data).unwrap()) {
                    eprintln!("❌ ファイル書き込み失敗: {}", e);
                } else {
                    println!("✅ APIキーを保存しました: {}", path.display());
                }
            } else {
                eprintln!("❗ APIキーを --api で指定してください");
            }
        })
}

fn chat_cmd() -> Command {
    Command::new("chat")
        .description("チャットで質問を送る")
        .usage("mcp chat '質問内容' --host <OLLAMA_HOST> --model <MODEL> [--provider <ollama|openai>] [--api-key <KEY>]")
        .flag(
            Flag::new("host", FlagType::String)
                .description("OLLAMAホストのURL")
                .alias("H"),
        )
        .flag(
            Flag::new("model", FlagType::String)
                .description("モデル名 (OLLAMA_MODEL / OPENAI_MODEL)")
                .alias("m"),
        )
        .flag(
            Flag::new("provider", FlagType::String)
                .description("使用するプロバイダ (ollama / openai)")
                .alias("p"),
        )
        .flag(
            Flag::new("api-key", FlagType::String)
                .description("OpenAI APIキー")
                .alias("k"),
        )
        .action(|c: &Context| {
            if let Some(question) = c.args.get(0) {
                ask_chat(c, question);
            } else {
                eprintln!("❗ 質問が必要です: mcp chat 'こんにちは'");
            }
        })
}

fn init_cmd() -> Command {
    Command::new("init")
        .description("Git 初期化")
        .usage("mcp init")
        .action(|_| {
            git_init();
        })
}

fn status_cmd() -> Command {
    Command::new("status")
        .description("Git ステータス表示")
        .usage("mcp status")
        .action(|_| {
            git_status();
        })
}

fn setup_cmd() -> Command {
    Command::new("setup")
        .description("MCP の初期セットアップ")
        .usage("mcp setup")
        .action(|_| {
            mcp_setup();
        })
}

pub fn mcp_cmd() -> Command {
    Command::new("mcp")
        .description("MCP操作コマンド")
        .usage("mcp <subcommand>")
        .alias("m")
        .command(chat_cmd())
        .command(init_cmd())
        .command(status_cmd())
        .command(setup_cmd())
        .command(set_api_key_cmd())
}
