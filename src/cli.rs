// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(about = "AI GPT CLI with MCP Server")]
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
    },
}

#[derive(Subcommand)]
pub enum ServerCommands {
    /// Setup Python MCP server environment
    Setup,
    /// Run the MCP server
    Run,
}
