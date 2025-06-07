use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::cli::TokenCommands;

/// Token usage record from Claude Code JSONL files
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenRecord {
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub usage: Option<TokenUsage>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub conversation_id: Option<String>,
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

/// Cost calculation summary
#[derive(Debug, Clone, Serialize)]
pub struct CostSummary {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub input_cost_usd: f64,
    pub output_cost_usd: f64,
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

    /// Parse JSONL files from Claude data directory
    pub fn parse_jsonl_files<P: AsRef<Path>>(&self, claude_dir: P) -> Result<Vec<TokenRecord>> {
        let claude_dir = claude_dir.as_ref();
        let mut records = Vec::new();

        // Look for JSONL files in the directory
        if let Ok(entries) = std::fs::read_dir(claude_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "jsonl") {
                    match self.parse_jsonl_file(&path) {
                        Ok(mut file_records) => records.append(&mut file_records),
                        Err(e) => {
                            eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(records)
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
                            // Only include records with usage data
                            if record.usage.is_some() {
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
        let mut input_tokens = 0u64;
        let mut output_tokens = 0u64;

        for record in records {
            if let Some(usage) = &record.usage {
                input_tokens += usage.input_tokens.unwrap_or(0);
                output_tokens += usage.output_tokens.unwrap_or(0);
            }
        }

        let total_tokens = input_tokens + output_tokens;
        let input_cost_usd = (input_tokens as f64 / 1_000_000.0) * self.config.input_cost_per_1m;
        let output_cost_usd = (output_tokens as f64 / 1_000_000.0) * self.config.output_cost_per_1m;
        let total_cost_usd = input_cost_usd + output_cost_usd;
        let total_cost_jpy = total_cost_usd * self.config.usd_to_jpy_rate;

        CostSummary {
            input_tokens,
            output_tokens,
            total_tokens,
            input_cost_usd,
            output_cost_usd,
            total_cost_usd,
            total_cost_jpy,
            record_count: records.len(),
        }
    }

    /// Group records by date (JST timezone)
    pub fn group_by_date(&self, records: &[TokenRecord]) -> Result<HashMap<String, Vec<TokenRecord>>> {
        let mut grouped: HashMap<String, Vec<TokenRecord>> = HashMap::new();

        for record in records {
            let date_str = self.extract_date_jst(&record.timestamp)?;
            grouped.entry(date_str).or_insert_with(Vec::new).push(record.clone());
        }

        Ok(grouped)
    }

    /// Extract date in JST from timestamp
    fn extract_date_jst(&self, timestamp: &str) -> Result<String> {
        if timestamp.is_empty() {
            return Err(anyhow!("Empty timestamp"));
        }

        // Try to parse various timestamp formats
        let dt = if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
            dt.with_timezone(&chrono_tz::Asia::Tokyo)
        } else if let Ok(dt) = DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.fZ") {
            dt.with_timezone(&chrono_tz::Asia::Tokyo)
        } else if let Ok(dt) = chrono::DateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S") {
            dt.with_timezone(&chrono_tz::Asia::Tokyo)
        } else {
            return Err(anyhow!("Failed to parse timestamp: {}", timestamp));
        };

        Ok(dt.format("%Y-%m-%d").to_string())
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
                if let Ok(date_str) = self.extract_date_jst(&record.timestamp) {
                    if let Ok(record_date) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                        return record_date.and_hms_opt(0, 0, 0).unwrap() >= cutoff;
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
        TokenCommands::Summary { period, claude_dir, details, format } => {
            handle_summary(
                period.unwrap_or_else(|| "week".to_string()), 
                claude_dir, 
                details, 
                format.unwrap_or_else(|| "table".to_string())
            ).await
        }
        TokenCommands::Daily { days, claude_dir } => {
            handle_daily(days.unwrap_or(7), claude_dir).await
        }
        TokenCommands::Status { claude_dir } => {
            handle_status(claude_dir).await
        }
        TokenCommands::Analyze { file } => {
            println!("Token analysis for file: {:?} - Not implemented yet", file);
            Ok(())
        }
        TokenCommands::Report { days } => {
            println!("Token report for {} days - Not implemented yet", days.unwrap_or(7));
            Ok(())
        }
        TokenCommands::Cost { month } => {
            println!("Token cost for month: {} - Not implemented yet", month.unwrap_or_else(|| "current".to_string()));
            Ok(())
        }
    }
}

/// Handle summary command
async fn handle_summary(
    period: String,
    claude_dir: Option<PathBuf>,
    details: bool,
    format: String,
) -> Result<()> {
    let analyzer = TokenAnalyzer::new();
    
    // Find Claude data directory
    let data_dir = claude_dir.or_else(|| TokenAnalyzer::find_claude_data_dir())
        .ok_or_else(|| anyhow!("Claude Code data directory not found"))?;

    println!("Loading data from: {}", data_dir.display());

    // Parse records
    let all_records = analyzer.parse_jsonl_files(&data_dir)?;
    if all_records.is_empty() {
        println!("No token usage data found");
        return Ok(());
    }

    // Filter by period
    let filtered_records = analyzer.filter_by_period(&all_records, &period)?;
    if filtered_records.is_empty() {
        println!("No data found for period: {}", period);
        return Ok(());
    }

    // Calculate summary
    let summary = analyzer.calculate_costs(&filtered_records);

    // Output results
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        "table" | _ => {
            print_summary_table(&summary, &period, details);
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

/// Print summary table
fn print_summary_table(summary: &CostSummary, period: &str, details: bool) {
    println!("\n=== Claude Code Token Usage Summary ({}) ===", period);
    println!();
    
    println!("📊 Token Usage:");
    println!("  Input tokens:  {:>12}", format_number(summary.input_tokens));
    println!("  Output tokens: {:>12}", format_number(summary.output_tokens));
    println!("  Total tokens:  {:>12}", format_number(summary.total_tokens));
    println!();
    
    println!("💰 Cost Estimation:");
    println!("  Input cost:    {:>12}", format!("${:.4} USD", summary.input_cost_usd));
    println!("  Output cost:   {:>12}", format!("${:.4} USD", summary.output_cost_usd));
    println!("  Total cost:    {:>12}", format!("${:.4} USD", summary.total_cost_usd));
    println!("  Total cost:    {:>12}", format!("¥{:.0} JPY", summary.total_cost_jpy));
    println!();
    
    if details {
        println!("📈 Additional Details:");
        println!("  Records:       {:>12}", format_number(summary.record_count as u64));
        println!("  Avg per record:{:>12}", format!("${:.4} USD", 
            if summary.record_count > 0 { summary.total_cost_usd / summary.record_count as f64 } else { 0.0 }));
        println!();
    }
    
    println!("💡 Cost calculation based on:");
    println!("   Input: $3.00 per 1M tokens");
    println!("   Output: $15.00 per 1M tokens");
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