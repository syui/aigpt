use anyhow::Result;
use std::env;

use aigpt::mcp::BaseMCPServer;

#[tokio::main]
async fn main() -> Result<()> {
    // 環境変数から設定を読み込み
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

    // 設定をログ出力
    eprintln!("Memory MCP Server (Standard) starting with config:");
    eprintln!("  AUTO_EXECUTE: {}", auto_execute);
    eprintln!("  AUTO_SAVE: {}", auto_save);
    eprintln!("  AUTO_SEARCH: {}", auto_search);
    eprintln!("  TRIGGER_SENSITIVITY: {}", trigger_sensitivity);

    let mut server = BaseMCPServer::new().await?;
    server.run().await?;
    
    Ok(())
}