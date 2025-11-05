use anyhow::Result;
use clap::{Parser, Subcommand};

use aigpt::core::{Memory, MemoryStore};
use aigpt::mcp::BaseMCPServer;

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(about = "Simple memory storage for Claude with MCP - Layer 1")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP server
    Server,

    /// Create a new memory
    Create {
        /// Content of the memory
        content: String,
    },

    /// Get a memory by ID
    Get {
        /// Memory ID
        id: String,
    },

    /// Update a memory
    Update {
        /// Memory ID
        id: String,
        /// New content
        content: String,
    },

    /// Delete a memory
    Delete {
        /// Memory ID
        id: String,
    },

    /// List all memories
    List,

    /// Search memories by content
    Search {
        /// Search query
        query: String,
    },

    /// Show statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server => {
            let server = BaseMCPServer::new()?;
            server.run()?;
        }

        Commands::Create { content } => {
            let store = MemoryStore::default()?;
            let memory = Memory::new(content);
            store.create(&memory)?;
            println!("Created memory: {}", memory.id);
        }

        Commands::Get { id } => {
            let store = MemoryStore::default()?;
            let memory = store.get(&id)?;
            println!("ID: {}", memory.id);
            println!("Content: {}", memory.content);
            println!("Created: {}", memory.created_at);
            println!("Updated: {}", memory.updated_at);
        }

        Commands::Update { id, content } => {
            let store = MemoryStore::default()?;
            let mut memory = store.get(&id)?;
            memory.update_content(content);
            store.update(&memory)?;
            println!("Updated memory: {}", memory.id);
        }

        Commands::Delete { id } => {
            let store = MemoryStore::default()?;
            store.delete(&id)?;
            println!("Deleted memory: {}", id);
        }

        Commands::List => {
            let store = MemoryStore::default()?;
            let memories = store.list()?;
            if memories.is_empty() {
                println!("No memories found");
            } else {
                for memory in memories {
                    println!("\n[{}]", memory.id);
                    println!("  {}", memory.content);
                    println!("  Created: {}", memory.created_at);
                }
            }
        }

        Commands::Search { query } => {
            let store = MemoryStore::default()?;
            let memories = store.search(&query)?;
            if memories.is_empty() {
                println!("No memories found matching '{}'", query);
            } else {
                println!("Found {} memory(ies):", memories.len());
                for memory in memories {
                    println!("\n[{}]", memory.id);
                    println!("  {}", memory.content);
                    println!("  Created: {}", memory.created_at);
                }
            }
        }

        Commands::Stats => {
            let store = MemoryStore::default()?;
            let count = store.count()?;
            println!("Total memories: {}", count);
        }
    }

    Ok(())
}