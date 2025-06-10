use std::path::PathBuf;
use anyhow::Result;
use crate::config::Config;
use crate::mcp_server::MCPServer;
use crate::persona::Persona;
use crate::transmission::TransmissionController;
use crate::scheduler::AIScheduler;

// Re-export from commands module
pub use commands::TokenCommands;

mod commands;

pub async fn handle_server(port: Option<u16>, data_dir: Option<PathBuf>) -> Result<()> {
    let port = port.unwrap_or(8080);
    let config = Config::new(data_dir.clone())?;
    
    let mut server = MCPServer::new(config, "mcp_user".to_string(), data_dir)?;
    server.start_server(port).await
}

pub async fn handle_chat(
    user_id: String,
    message: String,
    data_dir: Option<PathBuf>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    
    let (response, relationship_delta) = if provider.is_some() || model.is_some() {
        persona.process_ai_interaction(&user_id, &message, provider, model).await?
    } else {
        persona.process_interaction(&user_id, &message)?
    };
    
    println!("AI Response: {}", response);
    println!("Relationship Change: {:+.2}", relationship_delta);
    
    if let Some(relationship) = persona.get_relationship(&user_id) {
        println!("Relationship Status: {} (Score: {:.2})", 
                 relationship.status, relationship.score);
    }
    
    Ok(())
}

pub async fn handle_fortune(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let persona = Persona::new(&config)?;
    
    let state = persona.get_current_state()?;
    println!("🔮 Today's Fortune: {}", state.fortune_value);
    println!("😊 Current Mood: {}", state.current_mood);
    println!("✨ Breakthrough Status: {}", 
             if state.breakthrough_triggered { "Active" } else { "Inactive" });
    
    Ok(())
}

pub async fn handle_relationships(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let persona = Persona::new(&config)?;
    
    let relationships = persona.list_all_relationships();
    
    if relationships.is_empty() {
        println!("No relationships found.");
        return Ok(());
    }
    
    println!("📊 Relationships ({}):", relationships.len());
    for (user_id, rel) in relationships {
        println!("  {} - {} (Score: {:.2}, Interactions: {})", 
                 user_id, rel.status, rel.score, rel.total_interactions);
    }
    
    Ok(())
}

pub async fn handle_transmit(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    let mut transmission_controller = TransmissionController::new(config)?;
    
    let autonomous = transmission_controller.check_autonomous_transmissions(&mut persona).await?;
    let breakthrough = transmission_controller.check_breakthrough_transmissions(&mut persona).await?;
    let maintenance = transmission_controller.check_maintenance_transmissions(&mut persona).await?;
    
    let total = autonomous.len() + breakthrough.len() + maintenance.len();
    
    println!("📡 Transmission Check Complete:");
    println!("  Autonomous: {}", autonomous.len());
    println!("  Breakthrough: {}", breakthrough.len());
    println!("  Maintenance: {}", maintenance.len());
    println!("  Total: {}", total);
    
    Ok(())
}

pub async fn handle_maintenance(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    let mut transmission_controller = TransmissionController::new(config)?;
    
    persona.daily_maintenance()?;
    let maintenance_transmissions = transmission_controller.check_maintenance_transmissions(&mut persona).await?;
    
    let stats = persona.get_relationship_stats();
    
    println!("🔧 Daily maintenance completed");
    println!("📤 Maintenance transmissions sent: {}", maintenance_transmissions.len());
    println!("📊 Relationship stats: {:?}", stats);
    
    Ok(())
}

pub async fn handle_schedule(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    let mut transmission_controller = TransmissionController::new(config.clone())?;
    let mut scheduler = AIScheduler::new(&config)?;
    
    let executions = scheduler.run_scheduled_tasks(&mut persona, &mut transmission_controller).await?;
    let stats = scheduler.get_scheduler_stats();
    
    println!("⏰ Scheduler run completed");
    println!("📋 Tasks executed: {}", executions.len());
    println!("📊 Stats: {} total tasks, {} enabled, {:.2}% success rate", 
             stats.total_tasks, stats.enabled_tasks, stats.success_rate);
    
    Ok(())
}