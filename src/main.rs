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

    /// Read core.json
    ReadCore,

    /// Read latest memory record
    ReadMemory,

    /// Create new memory record
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
            match reader::read_memory()? {
                Some(record) => println!("{}", serde_json::to_string_pretty(&record)?),
                None => println!("No memory records found"),
            }
        }

        Some(Commands::SaveMemory { content }) => {
            writer::save_memory(&content)?;
            println!("Saved.");
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

    let memory_dir = config::collection_dir(&cfg, "ai.syui.gpt.memory");
    let memory_count = std::fs::read_dir(&memory_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .count()
        })
        .unwrap_or(0);

    let latest_version = match reader::read_memory() {
        Ok(Some(record)) => record["value"]["version"].as_u64().unwrap_or(0),
        _ => 0,
    };

    println!("aigpt - AI memory MCP server\n");
    println!("config: {}", config::config_file().display());
    println!("did:    {}", did);
    println!("handle: {}", handle);
    println!();
    println!("path: {}/", base.display());
    println!("  {}/{}", id, "ai.syui.gpt.core/self.json");
    println!("  {}/{}", id, "ai.syui.gpt.memory/*.json");
    println!();
    println!("memory: {} records (version: {})", memory_count, latest_version);
}
