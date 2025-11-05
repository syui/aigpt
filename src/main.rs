use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use aigpt::memory::MemoryManager;
use aigpt::mcp::BaseMCPServer;

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(version)]
#[command(about = "Simple memory storage for Claude with MCP (v0.2.0)")]
#[command(long_about = "AI memory system with psychological priority scoring and game-style results!\nVersion: 0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP server
    Server,
    /// Start MCP server (alias for server)
    Serve,
    /// Import ChatGPT conversations
    Import {
        /// Path to conversations.json file
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server | Commands::Serve => {
            let mut server = BaseMCPServer::new().await?;
            server.run().await?;
        }
        Commands::Import { file } => {
            let mut memory_manager = MemoryManager::new().await?;
            memory_manager.import_chatgpt_conversations(&file).await?;
            println!("Import completed successfully");
        }
    }

    Ok(())
}