// main.rs
mod cli;
mod config;
mod mcp;

use cli::{Args, Commands, ServerCommands, MemoryCommands};
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Server { command } => {
            match command {
                ServerCommands::Setup => {
                    mcp::server::setup();
                }
                ServerCommands::Run => {
                    mcp::server::run().await;
                }
            }
        }
        Commands::Chat { message, with_memory } => {
            if with_memory {
                if let Err(e) = mcp::memory::handle_chat_with_memory(&message).await {
                    eprintln!("❌ 記憶チャットエラー: {}", e);
                }
            } else {
                mcp::server::chat(&message).await;
            }
        }
        Commands::Memory { command } => {
            match command {
                MemoryCommands::Import { file } => {
                    if let Err(e) = mcp::memory::handle_import(&file).await {
                        eprintln!("❌ インポートエラー: {}", e);
                    }
                }
                MemoryCommands::Search { query, limit } => {
                    if let Err(e) = mcp::memory::handle_search(&query, limit).await {
                        eprintln!("❌ 検索エラー: {}", e);
                    }
                }
                MemoryCommands::List => {
                    if let Err(e) = mcp::memory::handle_list().await {
                        eprintln!("❌ 一覧取得エラー: {}", e);
                    }
                }
                MemoryCommands::Detail { filepath } => {
                    if let Err(e) = mcp::memory::handle_detail(&filepath).await {
                        eprintln!("❌ 詳細取得エラー: {}", e);
                    }
                }
            }
        }
    }
}
