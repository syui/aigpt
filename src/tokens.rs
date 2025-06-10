use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::cli::TokenCommands;
use std::process::Command;

/// Token usage record from Claude Code JSONL files
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenRecord {
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub message: Option<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(default)]
    #[serde(rename = "costUSD")]
    pub cost_usd: Option<f64>,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

/// Token usage details
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub total_tokens: Option<u64>,
}

/// Cost calculation modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CostMode {
    /// Use costUSD if available, otherwise calculate from tokens
    Auto,
    /// Always calculate costs from token counts, ignore costUSD
    Calculate,
    /// Always use pre-calculated costUSD values, show 0 for missing costs
    Display,
}

impl From<&str> for CostMode {
    fn from(mode: &str) -> Self {
        match mode {
            "calculate" => CostMode::Calculate,
            "display" => CostMode::Display,
            _ => CostMode::Auto, // default
        }
    }
}

/// Cost calculation summary
#[derive(Debug, Clone, Serialize)]
pub struct CostSummary {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
    pub input_cost_usd: f64,
    pub output_cost_usd: f64,
    pub cache_cost_usd: f64,
    pub total_cost_usd: f64,
    pub total_cost_jpy: f64,
    pub record_count: usize,
}

/// Daily breakdown of token usage
#[derive(Debug, Clone, Serialize)]
pub struct DailyBreakdown {
    pub date: String,
    pub summary: CostSummary,
}

/// Project breakdown of token usage
#[derive(Debug, Clone, Serialize)]
pub struct ProjectBreakdown {
    pub project_path: String,
    pub project_name: String,
    pub summary: CostSummary,
}

/// Configuration for cost calculation
#[derive(Debug, Clone)]
pub struct CostConfig {
    pub input_cost_per_1m: f64,  // USD per 1M input tokens
    pub output_cost_per_1m: f64, // USD per 1M output tokens
    pub usd_to_jpy_rate: f64,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            input_cost_per_1m: 3.0,
            output_cost_per_1m: 15.0,
            usd_to_jpy_rate: 150.0,
        }
    }
}

/// Token analysis functionality
pub struct TokenAnalyzer {
    config: CostConfig,
}

impl TokenAnalyzer {
    pub fn new() -> Self {
        Self {
            config: CostConfig::default(),
        }
    }

    pub fn with_config(config: CostConfig) -> Self {
        Self { config }
    }

    /// Find Claude Code data directory
    pub fn find_claude_data_dir() -> Option<PathBuf> {
        let possible_dirs = [
            dirs::home_dir().map(|h| h.join(".claude")),
            dirs::config_dir().map(|c| c.join("claude")),
            Some(PathBuf::from(".claude")),
        ];

        for dir_opt in possible_dirs.iter() {
            if let Some(dir) = dir_opt {
                if dir.exists() && dir.is_dir() {
                    return Some(dir.clone());
                }
            }
        }

        None
    }

    /// Parse JSONL files from Claude data directory (recursive search)
    pub fn parse_jsonl_files<P: AsRef<Path>>(&self, claude_dir: P) -> Result<Vec<TokenRecord>> {
        let claude_dir = claude_dir.as_ref();
        let mut records = Vec::new();

        // Recursively look for JSONL files in the directory and subdirectories
        self.parse_jsonl_files_recursive(claude_dir, &mut records)?;

        Ok(records)
    }

    /// Recursively parse JSONL files
    fn parse_jsonl_files_recursive(&self, dir: &Path, records: &mut Vec<TokenRecord>) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Recursively search subdirectories
                    self.parse_jsonl_files_recursive(&path, records)?;
                } else if path.extension().map_or(false, |ext| ext == "jsonl") {
                    match self.parse_jsonl_file(&path) {
                        Ok(mut file_records) => records.append(&mut file_records),
                        Err(e) => {
                            eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse a single JSONL file
    fn parse_jsonl_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<TokenRecord>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            match line {
                Ok(line_content) => {
                    if line_content.trim().is_empty() {
                        continue;
                    }
                    
                    match serde_json::from_str::<TokenRecord>(&line_content) {
                        Ok(record) => {
                            // Only include records with usage data in message or costUSD
                            let has_usage_data = record.cost_usd.is_some() ||
                                record.message.as_ref().and_then(|m| m.get("usage")).is_some();
                            if has_usage_data {
                                records.push(record);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read line {}: {}", line_num + 1, e);
                }
            }
        }

        Ok(records)
    }

    /// Calculate cost summary from records
    pub fn calculate_costs(&self, records: &[TokenRecord]) -> CostSummary {
        self.calculate_costs_with_mode(records, CostMode::Auto)
    }

    /// Calculate cost summary from records with specified cost mode
    pub fn calculate_costs_with_mode(&self, records: &[TokenRecord], mode: CostMode) -> CostSummary {
        let mut input_tokens = 0u64;
        let mut output_tokens = 0u64;
        let mut cache_creation_tokens = 0u64;
        let mut cache_read_tokens = 0u64;
        let mut total_cost_usd = 0.0;
        let mut cost_records_count = 0;

        for record in records {
            // Extract token usage from message.usage field
            if let Some(message) = &record.message {
                if let Some(usage) = message.get("usage") {
                    if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                        input_tokens += input;
                    }
                    if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                        output_tokens += output;
                    }
                    // Track cache tokens separately
                    if let Some(cache_creation) = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()) {
                        cache_creation_tokens += cache_creation;
                    }
                    if let Some(cache_read) = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()) {
                        cache_read_tokens += cache_read;
                    }
                }
            }

            // Calculate cost based on mode
            let record_cost = match mode {
                CostMode::Display => {
                    // Always use costUSD, even if undefined (0.0)
                    record.cost_usd.unwrap_or(0.0)
                }
                CostMode::Calculate => {
                    // Always calculate from tokens
                    if let Some(message) = &record.message {
                        if let Some(usage) = message.get("usage") {
                            let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
                            let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
                            let cache_creation = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
                            let cache_read = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
                            
                            // Regular tokens at normal price
                            let regular_cost = (input / 1_000_000.0) * self.config.input_cost_per_1m + 
                                             (output / 1_000_000.0) * self.config.output_cost_per_1m;
                            
                            // Cache tokens - cache creation at normal price, cache read at reduced price
                            let cache_cost = (cache_creation / 1_000_000.0) * self.config.input_cost_per_1m +
                                           (cache_read / 1_000_000.0) * (self.config.input_cost_per_1m * 0.1); // 10% of normal price for cache reads
                            
                            regular_cost + cache_cost
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
                CostMode::Auto => {
                    // Use costUSD if available, otherwise calculate from tokens
                    if let Some(cost) = record.cost_usd {
                        cost
                    } else if let Some(message) = &record.message {
                        if let Some(usage) = message.get("usage") {
                            let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
                            let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
                            let cache_creation = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
                            let cache_read = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
                            
                            // Regular tokens at normal price
                            let regular_cost = (input / 1_000_000.0) * self.config.input_cost_per_1m + 
                                             (output / 1_000_000.0) * self.config.output_cost_per_1m;
                            
                            // Cache tokens - cache creation at normal price, cache read at reduced price
                            let cache_cost = (cache_creation / 1_000_000.0) * self.config.input_cost_per_1m +
                                           (cache_read / 1_000_000.0) * (self.config.input_cost_per_1m * 0.1); // 10% of normal price for cache reads
                            
                            regular_cost + cache_cost
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                }
            };

            total_cost_usd += record_cost;
            if record.cost_usd.is_some() {
                cost_records_count += 1;
            }
        }
        
        // Debug info
        match mode {
            CostMode::Display => {
                if cost_records_count > 0 {
                    println!("Debug: Display mode - Found {} records with costUSD data, total: ${:.4}", cost_records_count, total_cost_usd);
                } else {
                    println!("Debug: Display mode - No costUSD data found, showing $0.00");
                }
            }
            CostMode::Calculate => {
                println!("Debug: Calculate mode - Using token-based calculation only, total: ${:.4}", total_cost_usd);
            }
            CostMode::Auto => {
                if cost_records_count > 0 {
                    println!("Debug: Auto mode - Found {} records with costUSD data, total: ${:.4}", cost_records_count, total_cost_usd);
                } else {
                    println!("Debug: Auto mode - No costUSD data found, using token-based calculation");
                }
            }
        }

        let total_tokens = input_tokens + output_tokens + cache_creation_tokens + cache_read_tokens;

        // Calculate component costs for display purposes
        let input_cost_usd = (input_tokens as f64 / 1_000_000.0) * self.config.input_cost_per_1m;
        let output_cost_usd = (output_tokens as f64 / 1_000_000.0) * self.config.output_cost_per_1m;
        let cache_cost_usd = (cache_creation_tokens as f64 / 1_000_000.0) * self.config.input_cost_per_1m +
                            (cache_read_tokens as f64 / 1_000_000.0) * (self.config.input_cost_per_1m * 0.1);
        let total_cost_jpy = total_cost_usd * self.config.usd_to_jpy_rate;

        CostSummary {
            input_tokens,
            output_tokens,
            cache_creation_tokens,
            cache_read_tokens,
            total_tokens,
            input_cost_usd,
            output_cost_usd,
            cache_cost_usd,
            total_cost_usd,
            total_cost_jpy,
            record_count: records.len(),
        }
    }

    /// Group records by date (JST timezone)
    pub fn group_by_date(&self, records: &[TokenRecord]) -> Result<HashMap<String, Vec<TokenRecord>>> {
        self.group_by_date_with_mode(records, CostMode::Auto)
    }

    /// Group records by date (JST timezone) with cost mode
    pub fn group_by_date_with_mode(&self, records: &[TokenRecord], _mode: CostMode) -> Result<HashMap<String, Vec<TokenRecord>>> {
        let mut grouped: HashMap<String, Vec<TokenRecord>> = HashMap::new();

        for record in records {
            if let Some(ref timestamp) = record.timestamp {
                if let Ok(date_str) = self.extract_date_jst(timestamp) {
                    grouped.entry(date_str).or_insert_with(Vec::new).push(record.clone());
                }
            }
        }

        Ok(grouped)
    }

    /// Extract date in JST from timestamp
    fn extract_date_jst(&self, timestamp: &str) -> Result<String> {
        if timestamp.is_empty() {
            return Err(anyhow!("Empty timestamp"));
        }

        // Try to parse various timestamp formats and convert to JST
        let dt = if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
            dt.with_timezone(&chrono::FixedOffset::east_opt(9 * 3600).unwrap())
        } else if let Ok(dt) = DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.fZ") {
            dt.with_timezone(&chrono::FixedOffset::east_opt(9 * 3600).unwrap())
        } else if let Ok(dt) = chrono::DateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S") {
            dt.with_timezone(&chrono::FixedOffset::east_opt(9 * 3600).unwrap())
        } else {
            return Err(anyhow!("Failed to parse timestamp: {}", timestamp));
        };

        Ok(dt.format("%Y-%m-%d").to_string())
    }

    /// Group records by project path
    pub fn group_by_project(&self, records: &[TokenRecord]) -> Result<HashMap<String, Vec<TokenRecord>>> {
        self.group_by_project_with_mode(records, CostMode::Auto)
    }

    /// Group records by project path with cost mode
    pub fn group_by_project_with_mode(&self, records: &[TokenRecord], _mode: CostMode) -> Result<HashMap<String, Vec<TokenRecord>>> {
        let mut grouped: HashMap<String, Vec<TokenRecord>> = HashMap::new();

        for record in records {
            // Extract project path from cwd field (at top level of JSON)
            let project_path = record.cwd
                .as_ref()
                .unwrap_or(&"Unknown Project".to_string())
                .clone();

            grouped.entry(project_path).or_insert_with(Vec::new).push(record.clone());
        }

        Ok(grouped)
    }

    /// Generate project breakdown with cost mode
    pub fn project_breakdown_with_mode(&self, records: &[TokenRecord], mode: CostMode) -> Result<Vec<ProjectBreakdown>> {
        let grouped = self.group_by_project_with_mode(records, mode)?;
        let mut breakdowns: Vec<ProjectBreakdown> = grouped
            .into_iter()
            .map(|(project_path, project_records)| {
                let project_name = std::path::Path::new(&project_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(&project_path)
                    .to_string();
                
                ProjectBreakdown {
                    project_path: project_path.clone(),
                    project_name,
                    summary: self.calculate_costs_with_mode(&project_records, mode),
                }
            })
            .collect();

        // Sort by total cost (highest first)
        breakdowns.sort_by(|a, b| b.summary.total_cost_usd.partial_cmp(&a.summary.total_cost_usd).unwrap_or(std::cmp::Ordering::Equal));

        Ok(breakdowns)
    }

    /// Generate project breakdown
    pub fn project_breakdown(&self, records: &[TokenRecord]) -> Result<Vec<ProjectBreakdown>> {
        self.project_breakdown_with_mode(records, CostMode::Auto)
    }

    /// Generate daily breakdown
    pub fn daily_breakdown(&self, records: &[TokenRecord]) -> Result<Vec<DailyBreakdown>> {
        let grouped = self.group_by_date(records)?;
        let mut breakdowns: Vec<DailyBreakdown> = grouped
            .into_iter()
            .map(|(date, date_records)| DailyBreakdown {
                date,
                summary: self.calculate_costs(&date_records),
            })
            .collect();

        // Sort by date (most recent first)
        breakdowns.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(breakdowns)
    }

    /// Filter records by time period
    pub fn filter_by_period(&self, records: &[TokenRecord], period: &str) -> Result<Vec<TokenRecord>> {
        let now = Local::now();
        let cutoff = match period {
            "today" => now.date_naive().and_hms_opt(0, 0, 0).unwrap(),
            "week" => (now - chrono::Duration::days(7)).naive_local(),
            "month" => (now - chrono::Duration::days(30)).naive_local(),
            "all" => return Ok(records.to_vec()),
            _ => return Err(anyhow!("Invalid period: {}", period)),
        };

        let filtered: Vec<TokenRecord> = records
            .iter()
            .filter(|record| {
                if let Some(ref timestamp) = record.timestamp {
                    if let Ok(date_str) = self.extract_date_jst(timestamp) {
                        if let Ok(record_date) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                            return record_date.and_hms_opt(0, 0, 0).unwrap() >= cutoff;
                        }
                    }
                }
                false
            })
            .cloned()
            .collect();

        Ok(filtered)
    }
}

/// Handle token-related commands
pub async fn handle_tokens(command: TokenCommands) -> Result<()> {
    match command {
        TokenCommands::Summary { period, claude_dir, details, format, mode } => {
            handle_summary(
                period.unwrap_or_else(|| "week".to_string()), 
                claude_dir, 
                details, 
                format.unwrap_or_else(|| "table".to_string()),
                mode.unwrap_or_else(|| "auto".to_string())
            ).await
        }
        TokenCommands::Daily { days, claude_dir } => {
            handle_daily(days.unwrap_or(7), claude_dir).await
        }
        TokenCommands::Status { claude_dir } => {
            handle_status(claude_dir).await
        }
        TokenCommands::Analyze { file } => {
            handle_analyze_file(file).await
        }
        TokenCommands::Report { days } => {
            handle_duckdb_report(days.unwrap_or(7)).await
        }
        TokenCommands::Cost { month } => {
            handle_duckdb_cost(month).await
        }
        TokenCommands::Projects { period, claude_dir, mode, details, top } => {
            handle_projects(
                period.unwrap_or_else(|| "week".to_string()),
                claude_dir,
                mode.unwrap_or_else(|| "calculate".to_string()),
                details,
                top.unwrap_or(10)
            ).await
        }
    }
}

/// Handle summary command
async fn handle_summary(
    period: String,
    claude_dir: Option<PathBuf>,
    details: bool,
    format: String,
    mode: String,
) -> Result<()> {
    let analyzer = TokenAnalyzer::new();
    let cost_mode = CostMode::from(mode.as_str());
    
    // Find Claude data directory
    let data_dir = claude_dir.or_else(|| TokenAnalyzer::find_claude_data_dir())
        .ok_or_else(|| anyhow!("Claude Code data directory not found"))?;

    println!("Loading data from: {}", data_dir.display());
    println!("Cost calculation mode: {:?}", cost_mode);

    // Parse records
    let all_records = analyzer.parse_jsonl_files(&data_dir)?;
    if all_records.is_empty() {
        println!("No token usage data found");
        return Ok(());
    }
    
    println!("Debug: Found {} total records", all_records.len());
    if let Some(latest) = all_records.iter().filter_map(|r| r.timestamp.as_ref()).max() {
        println!("Debug: Latest timestamp: {}", latest);
    }

    // Filter by period
    let filtered_records = analyzer.filter_by_period(&all_records, &period)?;
    if filtered_records.is_empty() {
        println!("No data found for period: {}", period);
        return Ok(());
    }

    // Calculate summary with specified mode
    let summary = analyzer.calculate_costs_with_mode(&filtered_records, cost_mode);

    // Output results
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        "table" | _ => {
            print_summary_table_with_mode(&summary, &period, details, &cost_mode);
        }
    }

    Ok(())
}

/// Handle daily command
async fn handle_daily(days: u32, claude_dir: Option<PathBuf>) -> Result<()> {
    let analyzer = TokenAnalyzer::new();
    
    // Find Claude data directory
    let data_dir = claude_dir.or_else(|| TokenAnalyzer::find_claude_data_dir())
        .ok_or_else(|| anyhow!("Claude Code data directory not found"))?;

    println!("Loading data from: {}", data_dir.display());

    // Parse records
    let records = analyzer.parse_jsonl_files(&data_dir)?;
    if records.is_empty() {
        println!("No token usage data found");
        return Ok(());
    }

    // Generate daily breakdown
    let breakdown = analyzer.daily_breakdown(&records)?;
    let limited_breakdown: Vec<_> = breakdown.into_iter().take(days as usize).collect();

    // Print daily breakdown
    print_daily_breakdown(&limited_breakdown);

    Ok(())
}

/// Handle status command
async fn handle_status(claude_dir: Option<PathBuf>) -> Result<()> {
    let analyzer = TokenAnalyzer::new();
    
    // Find Claude data directory
    let data_dir = claude_dir.or_else(|| TokenAnalyzer::find_claude_data_dir());

    match data_dir {
        Some(dir) => {
            println!("Claude Code data directory: {}", dir.display());
            
            // Parse records to get basic stats
            let records = analyzer.parse_jsonl_files(&dir)?;
            let summary = analyzer.calculate_costs(&records);
            
            println!("Total records: {}", summary.record_count);
            println!("Total tokens: {}", summary.total_tokens);
            println!("Estimated total cost: ${:.4} USD (¥{:.0} JPY)", 
                summary.total_cost_usd, summary.total_cost_jpy);
        }
        None => {
            println!("Claude Code data directory not found");
            println!("Checked locations:");
            println!("  - ~/.claude");
            println!("  - ~/.config/claude");
            println!("  - ./.claude");
        }
    }

    Ok(())
}

/// Handle projects command
async fn handle_projects(
    period: String,
    claude_dir: Option<PathBuf>,
    mode: String,
    details: bool,
    top: u32,
) -> Result<()> {
    let analyzer = TokenAnalyzer::new();
    let cost_mode = CostMode::from(mode.as_str());
    
    // Find Claude data directory
    let data_dir = claude_dir.or_else(|| TokenAnalyzer::find_claude_data_dir())
        .ok_or_else(|| anyhow!("Claude Code data directory not found"))?;

    println!("Loading data from: {}", data_dir.display());
    println!("Cost calculation mode: {:?}", cost_mode);

    // Parse records
    let all_records = analyzer.parse_jsonl_files(&data_dir)?;
    if all_records.is_empty() {
        println!("No token usage data found");
        return Ok(());
    }
    
    println!("Debug: Found {} total records", all_records.len());

    // Filter by period
    let filtered_records = analyzer.filter_by_period(&all_records, &period)?;
    if filtered_records.is_empty() {
        println!("No data found for period: {}", period);
        return Ok(());
    }

    // Generate project breakdown
    let project_breakdown = analyzer.project_breakdown_with_mode(&filtered_records, cost_mode)?;
    let limited_breakdown: Vec<_> = project_breakdown.into_iter().take(top as usize).collect();

    // Print project breakdown
    print_project_breakdown(&limited_breakdown, &period, details, &cost_mode);

    Ok(())
}

/// Print project breakdown
fn print_project_breakdown(breakdown: &[ProjectBreakdown], period: &str, details: bool, mode: &CostMode) {
    println!("\n=== Claude Code Token Usage by Project ({}) ===", period);
    println!();
    
    for (i, project) in breakdown.iter().enumerate() {
        println!("{}. 📁 {} ({})", 
            i + 1, 
            project.project_name,
            if project.project_path.len() > 50 { 
                format!("...{}", &project.project_path[project.project_path.len()-47..])
            } else {
                project.project_path.clone()
            }
        );
        
        println!("   📊 Tokens: {} total", format_number(project.summary.total_tokens));
        
        if details {
            println!("      • Input:         {:>12}", format_number(project.summary.input_tokens));
            println!("      • Output:        {:>12}", format_number(project.summary.output_tokens));
            println!("      • Cache create:  {:>12}", format_number(project.summary.cache_creation_tokens));
            println!("      • Cache read:    {:>12}", format_number(project.summary.cache_read_tokens));
        }
        
        println!("   💰 Cost: ${:.4} USD (¥{:.0} JPY)", 
            project.summary.total_cost_usd, 
            project.summary.total_cost_jpy);
        
        if details {
            println!("      • Records: {}", project.summary.record_count);
        }
        
        println!();
    }
    
    if breakdown.len() > 1 {
        let total_tokens: u64 = breakdown.iter().map(|p| p.summary.total_tokens).sum();
        let total_cost: f64 = breakdown.iter().map(|p| p.summary.total_cost_usd).sum();
        
        println!("📈 Summary:");
        println!("   Total tokens: {}", format_number(total_tokens));
        println!("   Total cost: ${:.4} USD (¥{:.0} JPY)", total_cost, total_cost * 150.0);
        println!("   Projects shown: {}", breakdown.len());
        println!();
    }
    
    println!("💡 Cost calculation (Mode: {:?})", mode);
}

/// Print summary table
fn print_summary_table(summary: &CostSummary, period: &str, details: bool) {
    print_summary_table_with_mode(summary, period, details, &CostMode::Auto)
}

/// Print summary table with cost mode information
fn print_summary_table_with_mode(summary: &CostSummary, period: &str, details: bool, mode: &CostMode) {
    println!("\n=== Claude Code Token Usage Summary ({}) ===", period);
    println!();
    
    println!("📊 Token Usage:");
    println!("  Input tokens:       {:>12}", format_number(summary.input_tokens));
    println!("  Output tokens:      {:>12}", format_number(summary.output_tokens));
    println!("  Cache creation:     {:>12}", format_number(summary.cache_creation_tokens));
    println!("  Cache read:         {:>12}", format_number(summary.cache_read_tokens));
    println!("  Total tokens:       {:>12}", format_number(summary.total_tokens));
    println!();
    
    println!("💰 Cost Estimation:");
    println!("  Input cost:         {:>12}", format!("${:.4} USD", summary.input_cost_usd));
    println!("  Output cost:        {:>12}", format!("${:.4} USD", summary.output_cost_usd));
    println!("  Cache cost:         {:>12}", format!("${:.4} USD", summary.cache_cost_usd));
    println!("  Total cost:         {:>12}", format!("${:.4} USD", summary.total_cost_usd));
    println!("  Total cost:         {:>12}", format!("¥{:.0} JPY", summary.total_cost_jpy));
    println!();
    
    if details {
        println!("📈 Additional Details:");
        println!("  Records:       {:>12}", format_number(summary.record_count as u64));
        println!("  Avg per record:{:>12}", format!("${:.4} USD", 
            if summary.record_count > 0 { summary.total_cost_usd / summary.record_count as f64 } else { 0.0 }));
        println!();
    }
    
    println!("💡 Cost calculation (Mode: {:?}):", mode);
    match mode {
        CostMode::Display => {
            println!("   Using pre-calculated costUSD values only");
            println!("   Missing costs shown as $0.00");
        }
        CostMode::Calculate => {
            println!("   Input: $3.00 per 1M tokens");
            println!("   Output: $15.00 per 1M tokens");
            println!("   Ignoring pre-calculated costUSD values");
        }
        CostMode::Auto => {
            println!("   Input: $3.00 per 1M tokens");
            println!("   Output: $15.00 per 1M tokens");
            println!("   Using costUSD when available, tokens otherwise");
        }
    }
    println!("   USD to JPY: 150.0");
}

/// Print daily breakdown
fn print_daily_breakdown(breakdown: &[DailyBreakdown]) {
    println!("\n=== Daily Token Usage Breakdown ===");
    println!();
    
    for daily in breakdown {
        println!("📅 {} (Records: {})", daily.date, daily.summary.record_count);
        println!("   Tokens: {} input + {} output = {} total", 
            format_number(daily.summary.input_tokens),
            format_number(daily.summary.output_tokens),
            format_number(daily.summary.total_tokens));
        println!("   Cost: ${:.4} USD (¥{:.0} JPY)", 
            daily.summary.total_cost_usd, 
            daily.summary.total_cost_jpy);
        println!();
    }
}

/// Format large numbers with commas
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Handle DuckDB-based token report (daily breakdown)
async fn handle_duckdb_report(days: u32) -> Result<()> {
    if !check_duckdb_available() {
        print_duckdb_help();
        return Err(anyhow!("DuckDB is not available"));
    }

    let claude_dir = TokenAnalyzer::find_claude_data_dir()
        .ok_or_else(|| anyhow!("Claude Code data directory not found"))?;

    println!("\x1b[1;34m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!("\x1b[1;36m                     Claude Code トークン使用状況レポート                  \x1b[0m");
    println!("\x1b[1;34m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!();

    let duckdb_query = format!(r#"
SELECT
  日付,
  入力トークン,
  出力トークン,
  合計トークン,
  料金
FROM (
  SELECT
    strftime(DATE(timestamp::TIMESTAMP AT TIME ZONE 'UTC' AT TIME ZONE 'Asia/Tokyo'), '%Y年%m月%d日') AS 日付,
    LPAD(FORMAT('{{:,}}', SUM(CAST(message -> 'usage' ->> 'input_tokens' AS INTEGER))), 12, ' ') AS 入力トークン,
    LPAD(FORMAT('{{:,}}', SUM(CAST(message -> 'usage' ->> 'output_tokens' AS INTEGER))), 12, ' ') AS 出力トークン,
    LPAD(FORMAT('{{:,}}', SUM(CAST(message -> 'usage' ->> 'input_tokens' AS INTEGER) + CAST(message -> 'usage' ->> 'output_tokens' AS INTEGER))), 12, ' ') AS 合計トークン,
    LPAD(FORMAT('¥{{:,}}', CAST(ROUND(SUM(costUSD) * 150, 0) AS INTEGER)), 10, ' ') AS 料金,
    DATE(timestamp::TIMESTAMP AT TIME ZONE 'UTC' AT TIME ZONE 'Asia/Tokyo') as sort_date
  FROM read_json('{}/**/*.jsonl')
  WHERE timestamp IS NOT NULL
  GROUP BY DATE(timestamp::TIMESTAMP AT TIME ZONE 'UTC' AT TIME ZONE 'Asia/Tokyo')

  UNION ALL

  SELECT
    '────────────────' AS 日付,
    '────────────' AS 入力トークン,
    '────────────' AS 出力トークン,
    '────────────' AS 合計トークン,
    '──────────' AS 料金,
    '9999-12-30' as sort_date

  UNION ALL

  SELECT
    '【合計】' AS 日付,
    LPAD(FORMAT('{{:,}}', SUM(CAST(message -> 'usage' ->> 'input_tokens' AS INTEGER))), 12, ' ') AS 入力トークン,
    LPAD(FORMAT('{{:,}}', SUM(CAST(message -> 'usage' ->> 'output_tokens' AS INTEGER))), 12, ' ') AS 出力トークン,
    LPAD(FORMAT('{{:,}}', SUM(CAST(message -> 'usage' ->> 'input_tokens' AS INTEGER) + CAST(message -> 'usage' ->> 'output_tokens' AS INTEGER))), 12, ' ') AS 合計トークン,
    LPAD(FORMAT('¥{{:,}}', CAST(ROUND(SUM(costUSD) * 150, 0) AS INTEGER)), 10, ' ') AS 料金,
    '9999-12-31' as sort_date
  FROM read_json('{}/**/*.jsonl')
  WHERE timestamp IS NOT NULL
)
ORDER BY sort_date DESC NULLS LAST
LIMIT {};
"#, claude_dir.display(), claude_dir.display(), days + 2);

    let output = Command::new("duckdb")
        .arg("-c")
        .arg(&duckdb_query)
        .output()?;

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("Error running DuckDB query:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    println!("\x1b[1;34m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");

    Ok(())
}

/// Handle DuckDB-based cost analysis (session breakdown)
async fn handle_duckdb_cost(month: Option<String>) -> Result<()> {
    if !check_duckdb_available() {
        print_duckdb_help();
        return Err(anyhow!("DuckDB is not available"));
    }

    let claude_dir = TokenAnalyzer::find_claude_data_dir()
        .ok_or_else(|| anyhow!("Claude Code data directory not found"))?;

    let date_filter = match month.as_deref() {
        Some("today") => "CURRENT_DATE".to_string(),
        Some(date) if date.contains('-') => format!("'{}'", date),
        Some("current") | None => "CURRENT_DATE".to_string(),
        Some(month_str) => {
            // Try to parse as YYYY-MM format
            if month_str.len() == 7 && month_str.contains('-') {
                format!("'{}-01'", month_str)
            } else {
                "CURRENT_DATE".to_string()
            }
        }
    };

    println!("\x1b[1;34m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!("\x1b[1;36m                    Claude Code 本日のセッション一覧                      \x1b[0m");
    println!("\x1b[1;34m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!();

    let duckdb_query = format!(r#"
WITH session_stats AS (
  SELECT
    sessionId,
    MIN(timestamp)::TIMESTAMP as session_start,
    MAX(timestamp)::TIMESTAMP as session_end,
    COUNT(DISTINCT CASE WHEN type = 'user' THEN uuid END) as user_messages,
    COUNT(DISTINCT CASE WHEN type = 'assistant' THEN uuid END) as assistant_messages,
    SUM(CASE WHEN type = 'assistant' AND json_extract(message, '$.usage.input_tokens') IS NOT NULL THEN CAST(json_extract(message, '$.usage.input_tokens') AS INTEGER) ELSE 0 END) as total_input_tokens,
    SUM(CASE WHEN type = 'assistant' AND json_extract(message, '$.usage.output_tokens') IS NOT NULL THEN CAST(json_extract(message, '$.usage.output_tokens') AS INTEGER) ELSE 0 END) as total_output_tokens,
    SUM(CASE WHEN type = 'assistant' AND costUSD IS NOT NULL THEN costUSD ELSE 0 END) as total_cost
  FROM read_json('{}/**/*.jsonl')
  WHERE type IN ('user', 'assistant')
    AND sessionId IS NOT NULL
  GROUP BY sessionId
),
today_sessions AS (
  SELECT * FROM session_stats s
  WHERE DATE(s.session_start AT TIME ZONE 'UTC' AT TIME ZONE 'Asia/Tokyo') = {}
)
SELECT 
  ID,
  開始時刻,
  時間,
  メッセージ数,
  料金,
  概要
FROM (
  SELECT
    SUBSTR(CAST(s.sessionId AS VARCHAR), 1, 8) || '...' as ID,
    STRFTIME((s.session_start AT TIME ZONE 'UTC' AT TIME ZONE 'Asia/Tokyo'), '%m/%d %H:%M') as 開始時刻,
    LPAD(CAST(ROUND(EXTRACT(EPOCH FROM (s.session_end - s.session_start)) / 60, 0) AS INTEGER) || '分', 5, ' ') as 時間,
    LPAD(CAST(s.user_messages AS VARCHAR), 6, ' ') as メッセージ数,
    LPAD(FORMAT('¥{{:,}}', CAST(ROUND(s.total_cost * 150, 0) AS INTEGER)), 8, ' ') as 料金,
    '本日のセッション' as 概要,
    s.session_start as sort_key
  FROM today_sessions s

  UNION ALL

  SELECT
    '────────────' as ID,
    '────────────' as 開始時刻,
    '─────' as 時間,
    '──────' as メッセージ数,
    '────────' as 料金,
    '────────────────────────────────────────────────────────────' as 概要,
    '9999-12-31'::TIMESTAMP as sort_key

  UNION ALL

  SELECT
    '【合計】' as ID,
    CAST(COUNT(*) AS VARCHAR) || '件' as 開始時刻,
    LPAD(CAST(SUM(ROUND(EXTRACT(EPOCH FROM (session_end - session_start)) / 60, 0)) AS INTEGER) || '分', 5, ' ') as 時間,
    LPAD(CAST(SUM(user_messages) AS VARCHAR), 6, ' ') as メッセージ数,
    LPAD(FORMAT('¥{{:,}}', CAST(ROUND(SUM(total_cost) * 150, 0) AS INTEGER)), 8, ' ') as 料金,
    '本日の合計' as 概要,
    '9999-12-31'::TIMESTAMP as sort_key
  FROM today_sessions
)
ORDER BY sort_key DESC;
"#, claude_dir.display(), date_filter);

    let output = Command::new("duckdb")
        .arg("-c")
        .arg(&duckdb_query)
        .output()?;

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("Error running DuckDB query:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    println!("\x1b[1;34m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");

    Ok(())
}

/// Handle analyze file command
async fn handle_analyze_file(file: PathBuf) -> Result<()> {
    let analyzer = TokenAnalyzer::new();
    
    if !file.exists() {
        return Err(anyhow!("File does not exist: {}", file.display()));
    }

    println!("Analyzing file: {}", file.display());
    
    let records = analyzer.parse_jsonl_file(&file)?;
    if records.is_empty() {
        println!("No token usage records found in file");
        return Ok(());
    }
    
    let summary = analyzer.calculate_costs(&records);
    print_summary_table(&summary, &format!("File: {}", file.file_name().unwrap_or_default().to_string_lossy()), true);
    
    Ok(())
}

/// Check if DuckDB is available on the system
fn check_duckdb_available() -> bool {
    Command::new("duckdb")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Print DuckDB installation help
pub fn print_duckdb_help() {
    println!("\n🦆 DuckDB is required for advanced token analysis features!");
    println!();
    println!("📦 Installation:");
    println!("  macOS:   brew install duckdb");
    println!("  Windows: Download from https://duckdb.org/docs/installation/");
    println!("  Linux:   apt install duckdb (or download from website)");
    println!();
    println!("🚀 After installation, try:");
    println!("  aigpt tokens report --days 7");
    println!("  aigpt tokens cost --month today");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let analyzer = TokenAnalyzer::new();
        let records = vec![
            TokenRecord {
                timestamp: "2024-01-01T10:00:00Z".to_string(),
                usage: Some(TokenUsage {
                    input_tokens: Some(1000),
                    output_tokens: Some(500),
                    total_tokens: Some(1500),
                }),
                model: Some("claude-3".to_string()),
                conversation_id: Some("test".to_string()),
            },
        ];

        let summary = analyzer.calculate_costs(&records);
        assert_eq!(summary.input_tokens, 1000);
        assert_eq!(summary.output_tokens, 500);
        assert_eq!(summary.total_tokens, 1500);
        assert_eq!(summary.record_count, 1);
    }

    #[test]
    fn test_date_extraction() {
        let analyzer = TokenAnalyzer::new();
        let result = analyzer.extract_date_jst("2024-01-01T10:00:00Z");
        assert!(result.is_ok());
        // Note: The exact date depends on JST conversion
    }
}