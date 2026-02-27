use anyhow::Result;
use clap::{Parser, Subcommand};

use aigpt::core::{reader, writer};
use aigpt::mcp::MCPServer;

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(about = "AI memory MCP server - read/write core.md and memory.md")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP server (JSON-RPC over stdio)
    Server,

    /// Read core.md
    ReadCore,

    /// Read memory.md
    ReadMemory,

    /// Save content to memory.md
    SaveMemory {
        /// Content to write
        content: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server => {
            let server = MCPServer::new();
            server.run()?;
        }

        Commands::ReadCore => {
            let content = reader::read_core()?;
            print!("{}", content);
        }

        Commands::ReadMemory => {
            let content = reader::read_memory()?;
            print!("{}", content);
        }

        Commands::SaveMemory { content } => {
            writer::save_memory(&content)?;
            println!("Saved to memory.md");
        }
    }

    Ok(())
}
