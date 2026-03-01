use anyhow::Result;
use clap::{Parser, Subcommand};

use aigpt::core::{config, reader, writer};
use aigpt::mcp::MCPServer;

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(about = "AI memory MCP server")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP server (JSON-RPC over stdio)
    Server,

    /// Read core record
    ReadCore,

    /// Read all memory records
    ReadMemory,

    /// Add a single memory element
    SaveMemory {
        /// Content to write
        content: String,
    },
}

fn main() -> Result<()> {
    config::init();
    let cli = Cli::parse();

    match cli.command {
        None => {
            print_status();
        }

        Some(Commands::Server) => {
            let server = MCPServer::new();
            server.run()?;
        }

        Some(Commands::ReadCore) => {
            let record = reader::read_core()?;
            println!("{}", serde_json::to_string_pretty(&record)?);
        }

        Some(Commands::ReadMemory) => {
            let records = reader::read_memory_all()?;
            if records.is_empty() {
                println!("No memory records found");
            } else {
                for record in &records {
                    println!("{}", serde_json::to_string_pretty(record)?);
                }
            }
        }

        Some(Commands::SaveMemory { content }) => {
            writer::save_memory(&content)?;
            println!("Saved. ({} records)", reader::memory_count());
        }
    }

    Ok(())
}

fn print_status() {
    let cfg = config::load();
    let did = cfg.did.clone().unwrap_or_else(|| "self".to_string());
    let handle = cfg.handle.clone().unwrap_or_else(|| "self".to_string());
    let base = config::base_dir(&cfg);
    let id = config::identity(&cfg);
    let count = reader::memory_count();

    println!("aigpt - AI memory MCP server\n");
    println!("config: {}", config::config_file().display());
    println!("did:    {}", did);
    println!("handle: {}", handle);
    println!("memory: {}", cfg.memory);
    println!();
    println!("path: {}/", base.display());
    println!("  {}/{}", id, "ai.syui.gpt.core/self.json");
    println!("  {}/{}", id, "ai.syui.gpt.memory/*.json");
    println!();
    println!("records: {}/{}", count, cfg.memory);
}
