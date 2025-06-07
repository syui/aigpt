use std::path::PathBuf;
use anyhow::Result;
use colored::*;

use crate::config::Config;
use crate::persona::Persona;

pub async fn handle_status(user_id: Option<String>, data_dir: Option<PathBuf>) -> Result<()> {
    // Load configuration
    let config = Config::new(data_dir)?;
    
    // Initialize persona
    let persona = Persona::new(&config)?;
    
    // Get current state
    let state = persona.get_current_state()?;
    
    // Display AI status
    println!("{}", "ai.gpt Status".cyan().bold());
    println!("Mood: {}", state.current_mood);
    println!("Fortune: {}/10", state.fortune_value);
    
    if state.breakthrough_triggered {
        println!("{}", "⚡ Breakthrough triggered!".yellow());
    }
    
    // Show personality traits
    println!("\n{}", "Current Personality".cyan().bold());
    for (trait_name, value) in &state.base_personality {
        println!("{}: {:.2}", trait_name.cyan(), value);
    }
    
    // Show specific relationship if requested
    if let Some(user_id) = user_id {
        if let Some(relationship) = persona.get_relationship(&user_id) {
            println!("\n{}: {}", "Relationship with".cyan(), user_id);
            println!("Status: {}", relationship.status);
            println!("Score: {:.2}", relationship.score);
            println!("Total Interactions: {}", relationship.total_interactions);
            println!("Transmission Enabled: {}", relationship.transmission_enabled);
            
            if relationship.is_broken {
                println!("{}", "⚠️  This relationship is broken and cannot be repaired.".red());
            }
        } else {
            println!("\n{}: {}", "No relationship found with".yellow(), user_id);
        }
    }
    
    Ok(())
}