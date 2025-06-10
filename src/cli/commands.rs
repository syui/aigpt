use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum TokenCommands {
    /// Show Claude Code token usage summary and estimated costs
    Summary {
        /// Time period (today, week, month, all)
        #[arg(long, default_value = "week")]
        period: Option<String>,
        /// Claude Code data directory path
        #[arg(long)]
        claude_dir: Option<PathBuf>,
        /// Show detailed breakdown
        #[arg(long)]
        details: bool,
        /// Output format (table, json)
        #[arg(long, default_value = "table")]
        format: Option<String>,
        /// Cost calculation mode (auto, calculate, display)
        #[arg(long, default_value = "auto")]
        mode: Option<String>,
    },
    /// Show daily token usage breakdown
    Daily {
        /// Number of days to show
        #[arg(long, default_value = "7")]
        days: Option<u32>,
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
    /// Analyze specific JSONL file (advanced)
    Analyze {
        /// Path to JSONL file to analyze
        file: PathBuf,
    },
    /// Generate beautiful token usage report using DuckDB (like the viral Claude Code usage visualization)
    Report {
        /// Number of days to include in report
        #[arg(long, default_value = "7")]
        days: Option<u32>,
    },
    /// Show detailed cost breakdown by session (requires DuckDB)
    Cost {
        /// Month to analyze (YYYY-MM, 'today', 'current')
        #[arg(long)]
        month: Option<String>,
    },
    /// Show token usage breakdown by project
    Projects {
        /// Time period (today, week, month, all)
        #[arg(long, default_value = "week")]
        period: Option<String>,
        /// Claude Code data directory path
        #[arg(long)]
        claude_dir: Option<PathBuf>,
        /// Cost calculation mode (auto, calculate, display)
        #[arg(long, default_value = "calculate")]
        mode: Option<String>,
        /// Show detailed breakdown
        #[arg(long)]
        details: bool,
        /// Number of top projects to show
        #[arg(long, default_value = "10")]
        top: Option<u32>,
    },
}