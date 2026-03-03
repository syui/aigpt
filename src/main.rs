use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

use aigpt::core::{config, reader, writer};
use aigpt::mcp::MCPServer;

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(about = "AI memory MCP server")]
#[command(disable_version_flag = true)]
struct Cli {
    #[arg(short = 'v', long = "version")]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version
    #[command(name = "v")]
    Version,

    /// Initial setup: link config, register MCP
    Setup,

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
    let cli = Cli::parse();

    if cli.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match &cli.command {
        Some(Commands::Version) => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(Commands::Setup) => return run_setup(),
        _ => {}
    }

    config::init();

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

        Some(Commands::Version) | Some(Commands::Setup) => unreachable!(),
    }

    Ok(())
}

fn run_setup() -> Result<()> {
    let cfg_dir = config::config_file()
        .parent()
        .unwrap()
        .to_path_buf();
    let cfg_file = config::config_file();
    let site_config = dirs::config_dir()
        .expect("Cannot find config directory")
        .join("ai.syui.log")
        .join("config.json");
    let aigpt_bin = which_command("aigpt")
        .unwrap_or_else(|| "aigpt".into());

    // 1. config symlink
    std::fs::create_dir_all(&cfg_dir)?;
    if cfg_file.is_symlink() || cfg_file.exists() {
        std::fs::remove_file(&cfg_file)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&site_config, &cfg_file)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&site_config, &cfg_file)?;
    println!("ok {} -> {}", cfg_file.display(), site_config.display());

    // 2. init data dirs
    config::init();
    println!("ok data initialized");

    // 3. claude mcp add
    if is_command_available("claude") {
        let status = Command::new("claude")
            .args([
                "mcp", "add",
                "--transport", "stdio",
                "aigpt",
                "--scope", "user",
                "--",
            ])
            .arg(&aigpt_bin)
            .arg("server")
            .status()?;
        if status.success() {
            println!("ok claude mcp add aigpt");
        } else {
            println!("warn claude mcp add failed");
        }
    } else {
        println!("skip claude mcp add (claude not found)");
    }

    println!("\ndone.");
    Ok(())
}

fn which_command(cmd: &str) -> Option<std::path::PathBuf> {
    Command::new("which")
        .arg(cmd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| std::path::PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
}

fn is_command_available(cmd: &str) -> bool {
    which_command(cmd).is_some()
}

fn print_status() {
    let cfg = config::load();
    let base = config::base_dir(&cfg);
    let count = reader::memory_count();

    println!("aigpt - AI memory MCP server\n");
    println!("config: {}", config::config_file().display());
    println!("did:    {}", cfg.did());
    println!("handle: {}", cfg.handle());
    println!("memory: {}", cfg.memory);
    println!();
    println!("path: {}/", base.display());
    println!("  {}/{}/self.json", cfg.identity(), config::COLLECTION_CORE);
    println!("  {}/{}/*.json", cfg.identity(), config::COLLECTION_MEMORY);
    println!();
    println!("records: {}/{}", count, cfg.memory);
}
