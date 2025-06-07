use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum TokenCommands {
    /// Show Claude Code token usage summary and estimated costs
    Summary {
        /// Time period (today, week, month, all)
        #[arg(long, default_value = "today")]
        period: String,
        /// Claude Code data directory path
        #[arg(long)]
        claude_dir: Option<PathBuf>,
        /// Show detailed breakdown
        #[arg(long)]
        details: bool,
        /// Output format (table, json)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Show daily token usage breakdown
    Daily {
        /// Number of days to show
        #[arg(long, default_value = "7")]
        days: u32,
        /// Claude Code data directory path
        #[arg(long)]
        claude_dir: Option<PathBuf>,
    },
    /// Check Claude Code data availability and basic stats
    Status {
        /// Claude Code data directory path
        #[arg(long)]
        claude_dir: Option<PathBuf>,
    },
}