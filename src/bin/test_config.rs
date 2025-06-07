use aigpt::config::Config;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Testing configuration loading...");
    
    // Debug: check which JSON files exist
    let possible_paths = vec![
        "../config.json",
        "config.json", 
        "gpt/config.json",
        "/Users/syui/ai/ai/gpt/config.json",
    ];
    
    println!("Checking for config.json files:");
    for path in &possible_paths {
        let path_buf = std::path::PathBuf::from(path);
        if path_buf.exists() {
            println!("  ✓ Found: {}", path);
        } else {
            println!("  ✗ Not found: {}", path);
        }
    }
    
    // Load configuration
    let config = Config::new(None)?;
    
    println!("Configuration loaded successfully!");
    println!("Default provider: {}", config.default_provider);
    println!("Available providers:");
    for (name, provider) in &config.providers {
        println!("  - {}: model={}, host={:?}", 
                 name, 
                 provider.default_model,
                 provider.host);
    }
    
    if let Some(mcp) = &config.mcp {
        println!("\nMCP Configuration:");
        println!("  Enabled: {}", mcp.enabled);
        println!("  Auto-detect: {}", mcp.auto_detect);
        println!("  Servers: {}", mcp.servers.len());
    }
    
    if let Some(atproto) = &config.atproto {
        println!("\nATProto Configuration:");
        println!("  Host: {}", atproto.host);
        println!("  Handle: {:?}", atproto.handle);
    }
    
    println!("\nConfig file path: {}", config.data_dir.join("config.json").display());
    
    Ok(())
}