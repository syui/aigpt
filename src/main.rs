use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod ai_provider;
mod cli;
use cli::TokenCommands;
mod config;
mod conversation;
mod docs;
mod http_client;
mod import;
mod mcp_server;
mod memory;
mod persona;
mod relationship;
mod scheduler;
mod shell;
mod status;
mod submodules;
mod tokens;
mod transmission;

#[derive(Parser)]
#[command(name = "aigpt")]
#[command(about = "AI.GPT - Autonomous transmission AI with unique personality")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check AI status and relationships
    Status {
        /// User ID to check status for
        user_id: Option<String>,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Chat with the AI
    Chat {
        /// User ID (atproto DID)
        user_id: String,
        /// Message to send to AI
        message: String,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
        /// AI model to use
        #[arg(short, long)]
        model: Option<String>,
        /// AI provider (ollama/openai)
        #[arg(long)]
        provider: Option<String>,
    },
    /// Start continuous conversation mode with MCP integration
    Conversation {
        /// User ID (atproto DID)
        user_id: String,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
        /// AI model to use
        #[arg(short, long)]
        model: Option<String>,
        /// AI provider (ollama/openai)
        #[arg(long)]
        provider: Option<String>,
    },
    /// Start continuous conversation mode with MCP integration (alias)
    Conv {
        /// User ID (atproto DID)
        user_id: String,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
        /// AI model to use
        #[arg(short, long)]
        model: Option<String>,
        /// AI provider (ollama/openai)
        #[arg(long)]
        provider: Option<String>,
    },
    /// Check today's AI fortune
    Fortune {
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// List all relationships
    Relationships {
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Check and send autonomous transmissions
    Transmit {
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Run daily maintenance tasks
    Maintenance {
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Run scheduled tasks
    Schedule {
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Start MCP server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Interactive shell mode
    Shell {
        /// User ID (atproto DID)
        user_id: String,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
        /// AI model to use
        #[arg(short, long)]
        model: Option<String>,
        /// AI provider (ollama/openai)
        #[arg(long)]
        provider: Option<String>,
    },
    /// Import ChatGPT conversation data
    ImportChatgpt {
        /// Path to ChatGPT export JSON file
        file_path: PathBuf,
        /// User ID for imported conversations
        #[arg(short, long)]
        user_id: Option<String>,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Documentation management
    Docs {
        /// Action to perform (generate, sync, list, status)
        action: String,
        /// Project name for generate/sync actions
        #[arg(short, long)]
        project: Option<String>,
        /// Output path for generated documentation
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Enable AI integration for documentation enhancement
        #[arg(long)]
        ai_integration: bool,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Submodule management
    Submodules {
        /// Action to perform (list, update, status)
        action: String,
        /// Specific module to update
        #[arg(short, long)]
        module: Option<String>,
        /// Update all submodules
        #[arg(long)]
        all: bool,
        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,
        /// Auto-commit changes after update
        #[arg(long)]
        auto_commit: bool,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,
    },
    /// Token usage analysis and cost estimation
    Tokens {
        #[command(subcommand)]
        command: TokenCommands,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { user_id, data_dir } => {
            status::handle_status(user_id, data_dir).await
        }
        Commands::Chat { user_id, message, data_dir, model, provider } => {
            cli::handle_chat(user_id, message, data_dir, model, provider).await
        }
        Commands::Conversation { user_id, data_dir, model, provider } => {
            conversation::handle_conversation(user_id, data_dir, model, provider).await
        }
        Commands::Conv { user_id, data_dir, model, provider } => {
            conversation::handle_conversation(user_id, data_dir, model, provider).await
        }
        Commands::Fortune { data_dir } => {
            cli::handle_fortune(data_dir).await
        }
        Commands::Relationships { data_dir } => {
            cli::handle_relationships(data_dir).await
        }
        Commands::Transmit { data_dir } => {
            cli::handle_transmit(data_dir).await
        }
        Commands::Maintenance { data_dir } => {
            cli::handle_maintenance(data_dir).await
        }
        Commands::Schedule { data_dir } => {
            cli::handle_schedule(data_dir).await
        }
        Commands::Server { port, data_dir } => {
            cli::handle_server(Some(port), data_dir).await
        }
        Commands::Shell { user_id, data_dir, model, provider } => {
            shell::handle_shell(user_id, data_dir, model, provider).await
        }
        Commands::ImportChatgpt { file_path, user_id, data_dir } => {
            import::handle_import_chatgpt(file_path, user_id, data_dir).await
        }
        Commands::Docs { action, project, output, ai_integration, data_dir } => {
            docs::handle_docs(action, project, output, ai_integration, data_dir).await
        }
        Commands::Submodules { action, module, all, dry_run, auto_commit, verbose, data_dir } => {
            submodules::handle_submodules(action, module, all, dry_run, auto_commit, verbose, data_dir).await
        }
        Commands::Tokens { command } => {
            tokens::handle_tokens(command).await
        }
    }
}
