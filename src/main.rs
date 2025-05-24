// main.rs
mod cli;
mod config;
mod mcp;

use cli::{Args, Commands, ServerCommands};
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
        Commands::Chat { message } => {
            mcp::server::chat(&message).await;
        }
    }
}
