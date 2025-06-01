// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(about = "AI GPT CLI with MCP Server and Memory")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// MCP Server management
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
    /// Chat with AI
    Chat {
        /// Message to send
        message: String,
        /// Use memory context
        #[arg(long)]
        with_memory: bool,
    },
    /// Memory management
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
}

#[derive(Subcommand)]
pub enum ServerCommands {
    /// Setup Python MCP server environment
    Setup,
    /// Run the MCP server
    Run,
}

#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Import ChatGPT conversation export file
    Import {
        /// Path to ChatGPT export JSON file
        file: String,
    },
    /// Search memories
    Search {
        /// Search query
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// List all memories
    List,
    /// Show memory details
    Detail {
        /// Path to memory file
        filepath: String,
    },
}
