use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

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
    /// Initial setup: clone repo, link config, register MCP
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

    if let Some(Commands::Setup) = &cli.command {
        return run_setup();
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

        Some(Commands::Setup) => unreachable!(),
    }

    Ok(())
}

fn run_setup() -> Result<()> {
    let home = dirs::home_dir().expect("Cannot find home directory");
    let ai_dir = home.join("ai");
    let log_dir = ai_dir.join("log");
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

    // 1. ~/ai/
    std::fs::create_dir_all(&ai_dir)?;
    println!("ok {}/", ai_dir.display());

    // 2. git clone
    if !log_dir.exists() {
        println!("cloning ai/log...");
        let status = Command::new("git")
            .args(["clone", "https://git.syui.ai/ai/log"])
            .current_dir(&ai_dir)
            .status()?;
        if !status.success() {
            anyhow::bail!("git clone failed");
        }
        println!("ok {}/", log_dir.display());
    } else {
        println!("skip {} (exists)", log_dir.display());
    }

    // 3. config symlink
    std::fs::create_dir_all(&cfg_dir)?;
    if cfg_file.is_symlink() || cfg_file.exists() {
        std::fs::remove_file(&cfg_file)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&site_config, &cfg_file)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&site_config, &cfg_file)?;
    println!("ok {} -> {}", cfg_file.display(), site_config.display());

    // 4. init data dirs
    config::init();
    println!("ok data initialized");

    // 5. claude mcp add
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
