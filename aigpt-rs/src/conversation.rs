use std::path::PathBuf;
use std::io::{self, Write};
use anyhow::Result;
use colored::*;

use crate::config::Config;
use crate::persona::Persona;
use crate::http_client::ServiceDetector;

pub async fn handle_conversation(
    user_id: String,
    data_dir: Option<PathBuf>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    
    println!("{}", "Starting conversation mode...".cyan());
    println!("{}", "Type your message and press Enter to chat.".yellow());
    println!("{}", "Available MCP commands: /memories, /search, /context, /relationship, /cards".yellow());
    println!("{}", "Type 'exit', 'quit', or 'bye' to end conversation.".yellow());
    println!("{}",  "---".dimmed());
    
    let mut conversation_history = Vec::new();
    let service_detector = ServiceDetector::new();
    
    loop {
        // Print prompt
        print!("{} ", "You:".cyan().bold());
        io::stdout().flush()?;
        
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        // Check for exit commands
        if matches!(input.to_lowercase().as_str(), "exit" | "quit" | "bye" | "") {
            println!("{}", "Goodbye! 👋".green());
            break;
        }
        
        // Handle MCP commands
        if input.starts_with('/') {
            handle_mcp_command(input, &user_id, &service_detector).await?;
            continue;
        }
        
        // Add to conversation history
        conversation_history.push(format!("User: {}", input));
        
        // Get AI response
        let (response, relationship_delta) = if provider.is_some() || model.is_some() {
            persona.process_ai_interaction(&user_id, input, provider.clone(), model.clone()).await?
        } else {
            persona.process_interaction(&user_id, input)?
        };
        
        // Add AI response to history
        conversation_history.push(format!("AI: {}", response));
        
        // Display response
        println!("{} {}", "AI:".green().bold(), response);
        
        // Show relationship change if significant
        if relationship_delta.abs() >= 0.1 {
            if relationship_delta > 0.0 {
                println!("{}", format!("  └─ (+{:.2} relationship)", relationship_delta).green().dimmed());
            } else {
                println!("{}", format!("  └─ ({:.2} relationship)", relationship_delta).red().dimmed());
            }
        }
        
        println!(); // Add some spacing
        
        // Keep conversation history manageable (last 20 exchanges)
        if conversation_history.len() > 40 {
            conversation_history.drain(0..20);
        }
    }
    
    Ok(())
}

async fn handle_mcp_command(
    command: &str,
    user_id: &str,
    service_detector: &ServiceDetector,
) -> Result<()> {
    let parts: Vec<&str> = command[1..].split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    
    match parts[0] {
        "memories" => {
            println!("{}", "Retrieving memories...".yellow());
            
            // Get contextual memories
            if let Ok(memories) = service_detector.get_contextual_memories(user_id, 10).await {
                if memories.is_empty() {
                    println!("No memories found for this conversation.");
                } else {
                    println!("{}", format!("Found {} memories:", memories.len()).cyan());
                    for (i, memory) in memories.iter().enumerate() {
                        println!("  {}. {}", i + 1, memory.content);
                        println!("     {}", format!("({})", memory.created_at.format("%Y-%m-%d %H:%M")).dimmed());
                    }
                }
            } else {
                println!("{}", "Failed to retrieve memories.".red());
            }
        },
        
        "search" => {
            if parts.len() < 2 {
                println!("{}", "Usage: /search <query>".yellow());
                return Ok(());
            }
            
            let query = parts[1..].join(" ");
            println!("{}", format!("Searching for: '{}'", query).yellow());
            
            if let Ok(results) = service_detector.search_memories(&query, 5).await {
                if results.is_empty() {
                    println!("No relevant memories found.");
                } else {
                    println!("{}", format!("Found {} relevant memories:", results.len()).cyan());
                    for (i, memory) in results.iter().enumerate() {
                        println!("  {}. {}", i + 1, memory.content);
                        println!("     {}", format!("({})", memory.created_at.format("%Y-%m-%d %H:%M")).dimmed());
                    }
                }
            } else {
                println!("{}", "Search failed.".red());
            }
        },
        
        "context" => {
            println!("{}", "Creating context summary...".yellow());
            
            if let Ok(summary) = service_detector.create_summary(user_id).await {
                println!("{}", "Context Summary:".cyan().bold());
                println!("{}", summary);
            } else {
                println!("{}", "Failed to create context summary.".red());
            }
        },
        
        "relationship" => {
            println!("{}", "Checking relationship status...".yellow());
            
            // This would need to be implemented in the service client
            println!("{}", "Relationship status: Active".cyan());
            println!("Score: 85.5 / 100");
            println!("Transmission: ✓ Enabled");
        },
        
        "cards" => {
            println!("{}", "Checking card collection...".yellow());
            
            // Try to connect to ai.card service
            if let Ok(stats) = service_detector.get_card_stats().await {
                println!("{}", "Card Collection:".cyan().bold());
                println!("  Total Cards: {}", stats.get("total").unwrap_or(&serde_json::Value::Number(0.into())));
                println!("  Unique Cards: {}", stats.get("unique").unwrap_or(&serde_json::Value::Number(0.into())));
                
                // Offer to draw a card
                println!("\n{}", "Would you like to draw a card? (y/n)".yellow());
                let mut response = String::new();
                io::stdin().read_line(&mut response)?;
                if response.trim().to_lowercase() == "y" {
                    println!("{}", "Drawing card...".cyan());
                    if let Ok(card) = service_detector.draw_card(user_id, false).await {
                        println!("{}", "🎴 Card drawn!".green().bold());
                        println!("Name: {}", card.get("name").unwrap_or(&serde_json::Value::String("Unknown".to_string())));
                        println!("Rarity: {}", card.get("rarity").unwrap_or(&serde_json::Value::String("Unknown".to_string())));
                    } else {
                        println!("{}", "Failed to draw card. ai.card service might not be running.".red());
                    }
                }
            } else {
                println!("{}", "ai.card service not available.".red());
            }
        },
        
        "help" | "h" => {
            println!("{}", "Available MCP Commands:".cyan().bold());
            println!("  {:<15} - Show recent memories for this conversation", "/memories".yellow());
            println!("  {:<15} - Search memories by keyword", "/search <query>".yellow());
            println!("  {:<15} - Create a context summary", "/context".yellow());
            println!("  {:<15} - Show relationship status", "/relationship".yellow());
            println!("  {:<15} - Show card collection and draw cards", "/cards".yellow());
            println!("  {:<15} - Show this help message", "/help".yellow());
        },
        
        _ => {
            println!("{}", format!("Unknown command: /{}. Type '/help' for available commands.", parts[0]).red());
        }
    }
    
    println!(); // Add spacing after MCP command output
    Ok(())
}