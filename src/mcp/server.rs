// src/mcp/server.rs
use crate::config::ConfigPaths;
//use std::fs;
use std::process::Command as OtherCommand;
use std::env;
use fs_extra::dir::{copy, CopyOptions};

pub fn setup() {
    println!("🔧 MCP Server環境をセットアップしています...");
    let config = ConfigPaths::new();
    let mcp_dir = config.mcp_dir();

    // プロジェクトのmcp/ディレクトリからファイルをコピー
    let current_dir = env::current_dir().expect("現在のディレクトリを取得できません");
    let project_mcp_dir = current_dir.join("mcp");
    if !project_mcp_dir.exists() {
        eprintln!("❌ プロジェクトのmcp/ディレクトリが見つかりません: {}", project_mcp_dir.display());
        return;
    }

    if mcp_dir.exists() {
        fs_extra::dir::remove(&mcp_dir).expect("既存のmcp_dirの削除に失敗しました");
    }

    let mut options = CopyOptions::new();
    options.overwrite = true; // 上書き
    options.copy_inside = true; // 中身だけコピー

    copy(&project_mcp_dir, &mcp_dir, &options).expect("コピーに失敗しました");   

    // 仮想環境の作成
    let venv_path = config.venv_path();
    if !venv_path.exists() {
        println!("🐍 仮想環境を作成しています...");
        let output = OtherCommand::new("python3")
            .args(&["-m", "venv", ".venv"])
            .current_dir(&mcp_dir)
            .output()
            .expect("venvの作成に失敗しました");

        if !output.status.success() {
            eprintln!("❌ venv作成エラー: {}", String::from_utf8_lossy(&output.stderr));
            return;
        }
        println!("✅ 仮想環境を作成しました");
    } else {
        println!("✅ 仮想環境は既に存在します");
    }

    // 依存関係のインストール
    println!("📦 依存関係をインストールしています...");
    let pip_path = config.pip_executable();
    let output = OtherCommand::new(&pip_path)
        .args(&["install", "-r", "requirements.txt"])
        .current_dir(&mcp_dir)
        .output()
        .expect("pipコマンドの実行に失敗しました");

    if !output.status.success() {
        eprintln!("❌ pip installエラー: {}", String::from_utf8_lossy(&output.stderr));
        return;
    }

    println!("✅ MCP Server環境のセットアップが完了しました!");
    println!("📍 セットアップ場所: {}", mcp_dir.display());
}

pub async fn run() {
    println!("🚀 MCP Serverを起動しています...");
    
    let config = ConfigPaths::new();
    let mcp_dir = config.mcp_dir();
    let python_path = config.python_executable();
    let server_py_path = mcp_dir.join("server.py");

    // セットアップの確認
    if !server_py_path.exists() {
        eprintln!("❌ server.pyが見つかりません。先に 'aigpt server setup' を実行してください。");
        return;
    }

    if !python_path.exists() {
        eprintln!("❌ Python実行ファイルが見つかりません。先に 'aigpt server setup' を実行してください。");
        return;
    }

    // サーバーの起動
    println!("🔗 サーバーを起動中... (Ctrl+Cで停止)");
    let mut child = OtherCommand::new(&python_path)
        .arg("server.py")
        .current_dir(&mcp_dir)
        .spawn()
        .expect("MCP Serverの起動に失敗しました");

    // サーバーの終了を待機
    match child.wait() {
        Ok(status) => {
            if status.success() {
                println!("✅ MCP Serverが正常に終了しました");
            } else {
                println!("❌ MCP Serverが異常終了しました: {}", status);
            }
        }
        Err(e) => {
            eprintln!("❌ MCP Serverの実行中にエラーが発生しました: {}", e);
        }
    }
}

pub async fn chat(message: &str) {
    println!("💬 チャットを開始しています...");
    
    let config = ConfigPaths::new();
    let mcp_dir = config.mcp_dir();
    let python_path = config.python_executable();
    let chat_py_path = mcp_dir.join("chat.py");

    // セットアップの確認
    if !chat_py_path.exists() {
        eprintln!("❌ chat.pyが見つかりません。先に 'aigpt server setup' を実行してください。");
        return;
    }

    if !python_path.exists() {
        eprintln!("❌ Python実行ファイルが見つかりません。先に 'aigpt server setup' を実行してください。");
        return;
    }

    // チャットの実行
    let output = OtherCommand::new(&python_path)
        .args(&["chat.py", message])
        .current_dir(&mcp_dir)
        .output()
        .expect("chat.pyの実行に失敗しました");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !stderr.is_empty() {
            print!("{}", stderr);
        }
        print!("{}", stdout);
    } else {
        eprintln!("❌ チャット実行エラー: {}", String::from_utf8_lossy(&output.stderr));
    }
}
