use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod memory;
pub mod mcp;

use memory::MemoryManager;
use mcp::BaseMCPServer;

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(about = "Simple memory storage for Claude with MCP")]
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