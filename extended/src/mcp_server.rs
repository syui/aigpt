use anyhow::Result;
use std::env;

// Re-use core modules from parent (these imports removed as they're unused)

mod extended_mcp;
use extended_mcp::ExtendedMCPServer;

#[tokio::main]
async fn main() -> Result<()> {
    // 環境変数から拡張機能の設定を読み込み
    let auto_execute = env::var("MEMORY_AUTO_EXECUTE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    let auto_save = env::var("MEMORY_AUTO_SAVE")
        .unwrap_or_else(|_| "false".to_string()) 
        .parse::<bool>()
        .unwrap_or(false);
    
    let auto_search = env::var("MEMORY_AUTO_SEARCH")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    let trigger_sensitivity = env::var("TRIGGER_SENSITIVITY")
        .unwrap_or_else(|_| "medium".to_string());

    let enable_ai_analysis = cfg!(feature = "ai-analysis");
    let enable_semantic_search = cfg!(feature = "semantic-search");
    let enable_web_integration = cfg!(feature = "web-integration");

    // 拡張設定をログ出力
    eprintln!("Memory MCP Server (Extended) starting with config:");
    eprintln!("  AUTO_EXECUTE: {}", auto_execute);
    eprintln!("  AUTO_SAVE: {}", auto_save);
    eprintln!("  AUTO_SEARCH: {}", auto_search);
    eprintln!("  TRIGGER_SENSITIVITY: {}", trigger_sensitivity);
    eprintln!("  AI_ANALYSIS: {}", enable_ai_analysis);
    eprintln!("  SEMANTIC_SEARCH: {}", enable_semantic_search);
    eprintln!("  WEB_INTEGRATION: {}", enable_web_integration);

    let mut server = ExtendedMCPServer::new().await?;
    server.run().await?;
    
    Ok(())
}