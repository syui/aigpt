use std::path::PathBuf;
use anyhow::Result;
use colored::*;

use crate::config::Config;
use crate::persona::Persona;
use crate::transmission::TransmissionController;
use crate::scheduler::AIScheduler;
use crate::mcp_server::MCPServer;

pub async fn handle_chat(
    user_id: String,
    message: String,
    data_dir: Option<PathBuf>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    
    // Try AI-powered response first, fallback to simple response
    let (response, relationship_delta) = if provider.is_some() || model.is_some() {
        // Use AI provider
        persona.process_ai_interaction(&user_id, &message, provider, model).await?
    } else {
        // Use simple response (backward compatibility)
        persona.process_interaction(&user_id, &message)?
    };
    
    // Display conversation
    println!("{}: {}", "User".cyan(), message);
    println!("{}: {}", "AI".green(), response);
    
    // Show relationship change if significant
    if relationship_delta.abs() >= 0.1 {
        if relationship_delta > 0.0 {
            println!("{}", format!("(+{:.2} relationship)", relationship_delta).green());
        } else {
            println!("{}", format!("({:.2} relationship)", relationship_delta).red());
        }
    }
    
    // Show current relationship status
    if let Some(relationship) = persona.get_relationship(&user_id) {
        println!("\n{}: {}", "Relationship Status".cyan(), relationship.status);
        println!("Score: {:.2} / {}", relationship.score, relationship.threshold);
        println!("Transmission: {}", if relationship.transmission_enabled { "✓ Enabled".green() } else { "✗ Disabled".yellow() });
        
        if relationship.is_broken {
            println!("{}", "⚠️  This relationship is broken and cannot be repaired.".red());
        }
    }
    
    Ok(())
}

pub async fn handle_fortune(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let persona = Persona::new(&config)?;
    let state = persona.get_current_state()?;
    
    // Fortune display
    let fortune_stars = "🌟".repeat(state.fortune_value as usize);
    let empty_stars = "☆".repeat((10 - state.fortune_value) as usize);
    
    println!("{}", "AI Fortune".yellow().bold());
    println!("{}{}", fortune_stars, empty_stars);
    println!("Today's Fortune: {}/10", state.fortune_value);
    println!("Date: {}", chrono::Utc::now().format("%Y-%m-%d"));
    
    if state.breakthrough_triggered {
        println!("\n{}", "⚡ BREAKTHROUGH! Special fortune activated!".yellow());
    }
    
    Ok(())
}

pub async fn handle_relationships(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let persona = Persona::new(&config)?;
    let relationships = persona.list_all_relationships();
    
    if relationships.is_empty() {
        println!("{}", "No relationships yet".yellow());
        return Ok(());
    }
    
    println!("{}", "All Relationships".cyan().bold());
    println!();
    
    for (user_id, rel) in relationships {
        let transmission = if rel.is_broken {
            "💔"
        } else if rel.transmission_enabled {
            "✓"
        } else {
            "✗"
        };
        
        let last_interaction = rel.last_interaction
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "Never".to_string());
        
        let user_display = if user_id.len() > 16 {
            format!("{}...", &user_id[..16])
        } else {
            user_id
        };
        
        println!("{:<20} {:<12} {:<8} {:<5} {}", 
                user_display.cyan(),
                rel.status,
                format!("{:.2}", rel.score),
                transmission,
                last_interaction.dimmed());
    }
    
    Ok(())
}

pub async fn handle_transmit(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    let mut transmission_controller = TransmissionController::new(&config)?;
    
    println!("{}", "🚀 Checking for autonomous transmissions...".cyan().bold());
    
    // Check all types of transmissions
    let autonomous = transmission_controller.check_autonomous_transmissions(&mut persona).await?;
    let breakthrough = transmission_controller.check_breakthrough_transmissions(&mut persona).await?;
    let maintenance = transmission_controller.check_maintenance_transmissions(&mut persona).await?;
    
    let total_transmissions = autonomous.len() + breakthrough.len() + maintenance.len();
    
    if total_transmissions == 0 {
        println!("{}", "No transmissions needed at this time.".yellow());
        return Ok(());
    }
    
    println!("\n{}", "📨 Transmission Results:".green().bold());
    
    // Display autonomous transmissions
    if !autonomous.is_empty() {
        println!("\n{}", "🤖 Autonomous Transmissions:".blue());
        for transmission in autonomous {
            println!("  {} → {}", transmission.user_id.cyan(), transmission.message);
            println!("    {} {}", "Type:".dimmed(), transmission.transmission_type);
            println!("    {} {}", "Time:".dimmed(), transmission.timestamp.format("%H:%M:%S"));
        }
    }
    
    // Display breakthrough transmissions
    if !breakthrough.is_empty() {
        println!("\n{}", "⚡ Breakthrough Transmissions:".yellow());
        for transmission in breakthrough {
            println!("  {} → {}", transmission.user_id.cyan(), transmission.message);
            println!("    {} {}", "Time:".dimmed(), transmission.timestamp.format("%H:%M:%S"));
        }
    }
    
    // Display maintenance transmissions
    if !maintenance.is_empty() {
        println!("\n{}", "🔧 Maintenance Transmissions:".green());
        for transmission in maintenance {
            println!("  {} → {}", transmission.user_id.cyan(), transmission.message);
            println!("    {} {}", "Time:".dimmed(), transmission.timestamp.format("%H:%M:%S"));
        }
    }
    
    // Show transmission stats
    let stats = transmission_controller.get_transmission_stats();
    println!("\n{}", "📊 Transmission Stats:".magenta().bold());
    println!("Total: {} | Today: {} | Success Rate: {:.1}%", 
             stats.total_transmissions, 
             stats.today_transmissions,
             stats.success_rate * 100.0);
    
    Ok(())
}

pub async fn handle_maintenance(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    let mut transmission_controller = TransmissionController::new(&config)?;
    
    println!("{}", "🔧 Running daily maintenance...".cyan().bold());
    
    // Run daily maintenance on persona (time decay, etc.)
    persona.daily_maintenance()?;
    println!("✓ {}", "Applied relationship time decay".green());
    
    // Check for maintenance transmissions
    let maintenance_transmissions = transmission_controller.check_maintenance_transmissions(&mut persona).await?;
    
    if maintenance_transmissions.is_empty() {
        println!("✓ {}", "No maintenance transmissions needed".green());
    } else {
        println!("📨 {}", format!("Sent {} maintenance messages:", maintenance_transmissions.len()).green());
        for transmission in maintenance_transmissions {
            println!("  {} → {}", transmission.user_id.cyan(), transmission.message);
        }
    }
    
    // Show relationship stats after maintenance
    if let Some(rel_stats) = persona.get_relationship_stats() {
        println!("\n{}", "📊 Relationship Statistics:".magenta().bold());
        println!("Total: {} | Active: {} | Transmission Enabled: {} | Broken: {}", 
                 rel_stats.total_relationships,
                 rel_stats.active_relationships,
                 rel_stats.transmission_enabled,
                 rel_stats.broken_relationships);
        println!("Average Score: {:.2}", rel_stats.avg_score);
    }
    
    // Show transmission history
    let recent_transmissions = transmission_controller.get_recent_transmissions(5);
    if !recent_transmissions.is_empty() {
        println!("\n{}", "📝 Recent Transmissions:".blue().bold());
        for transmission in recent_transmissions {
            println!("  {} {} → {} ({})", 
                     transmission.timestamp.format("%m-%d %H:%M").to_string().dimmed(),
                     transmission.user_id.cyan(),
                     transmission.message,
                     transmission.transmission_type.to_string().yellow());
        }
    }
    
    println!("\n{}", "✅ Daily maintenance completed!".green().bold());
    
    Ok(())
}

pub async fn handle_schedule(data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    let mut transmission_controller = TransmissionController::new(&config)?;
    let mut scheduler = AIScheduler::new(&config)?;
    
    println!("{}", "⏰ Running scheduled tasks...".cyan().bold());
    
    // Run all due scheduled tasks
    let executions = scheduler.run_scheduled_tasks(&mut persona, &mut transmission_controller).await?;
    
    if executions.is_empty() {
        println!("{}", "No scheduled tasks due at this time.".yellow());
    } else {
        println!("\n{}", "📋 Task Execution Results:".green().bold());
        
        for execution in &executions {
            let status_icon = if execution.success { "✅" } else { "❌" };
            let _status_color = if execution.success { "green" } else { "red" };
            
            println!("  {} {} ({:.0}ms)", 
                     status_icon, 
                     execution.task_id.cyan(),
                     execution.duration_ms);
            
            if let Some(result) = &execution.result {
                println!("    {}", result);
            }
            
            if let Some(error) = &execution.error {
                println!("    {} {}", "Error:".red(), error);
            }
        }
    }
    
    // Show scheduler statistics
    let stats = scheduler.get_scheduler_stats();
    println!("\n{}", "📊 Scheduler Statistics:".magenta().bold());
    println!("Total Tasks: {} | Enabled: {} | Due: {}", 
             stats.total_tasks, 
             stats.enabled_tasks,
             stats.due_tasks);
    println!("Executions: {} | Today: {} | Success Rate: {:.1}%", 
             stats.total_executions,
             stats.today_executions,
             stats.success_rate * 100.0);
    println!("Average Duration: {:.1}ms", stats.avg_duration_ms);
    
    // Show upcoming tasks
    let tasks = scheduler.list_tasks();
    if !tasks.is_empty() {
        println!("\n{}", "📅 Upcoming Tasks:".blue().bold());
        
        let mut upcoming_tasks: Vec<_> = tasks.values()
            .filter(|task| task.enabled)
            .collect();
        upcoming_tasks.sort_by_key(|task| task.next_run);
        
        for task in upcoming_tasks.iter().take(5) {
            let time_until = (task.next_run - chrono::Utc::now()).num_minutes();
            let time_display = if time_until > 60 {
                format!("{}h {}m", time_until / 60, time_until % 60)
            } else if time_until > 0 {
                format!("{}m", time_until)
            } else {
                "overdue".to_string()
            };
            
            println!("  {} {} ({})", 
                     task.next_run.format("%m-%d %H:%M").to_string().dimmed(),
                     task.task_type.to_string().cyan(),
                     time_display.yellow());
        }
    }
    
    // Show recent execution history
    let recent_executions = scheduler.get_execution_history(Some(5));
    if !recent_executions.is_empty() {
        println!("\n{}", "📝 Recent Executions:".blue().bold());
        for execution in recent_executions {
            let status_icon = if execution.success { "✅" } else { "❌" };
            println!("  {} {} {} ({:.0}ms)", 
                     execution.execution_time.format("%m-%d %H:%M").to_string().dimmed(),
                     status_icon,
                     execution.task_id.cyan(),
                     execution.duration_ms);
        }
    }
    
    println!("\n{}", "⏰ Scheduler check completed!".green().bold());
    
    Ok(())
}

pub async fn handle_server(port: Option<u16>, data_dir: Option<PathBuf>) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut mcp_server = MCPServer::new(config)?;
    let port = port.unwrap_or(8080);
    
    println!("{}", "🚀 Starting ai.gpt MCP Server...".cyan().bold());
    
    // Start the MCP server
    mcp_server.start_server(port).await?;
    
    // Show server info
    let tools = mcp_server.get_tools();
    println!("\n{}", "📋 Available MCP Tools:".green().bold());
    
    for (i, tool) in tools.iter().enumerate() {
        println!("{}. {} - {}", 
                 (i + 1).to_string().cyan(), 
                 tool.name.green(), 
                 tool.description);
    }
    
    println!("\n{}", "💡 Usage Examples:".blue().bold());
    println!("  • {}: Get AI status and mood", "get_status".green());
    println!("  • {}: Chat with the AI", "chat_with_ai".green());
    println!("  • {}: View all relationships", "get_relationships".green());
    println!("  • {}: Run autonomous transmissions", "check_transmissions".green());
    println!("  • {}: Execute scheduled tasks", "run_scheduler".green());
    
    println!("\n{}", "🔧 Server Configuration:".magenta().bold());
    println!("Port: {}", port.to_string().yellow());
    println!("Tools: {}", tools.len().to_string().yellow());
    println!("Protocol: MCP (Model Context Protocol)");
    
    println!("\n{}", "✅ MCP Server is ready to accept requests".green().bold());
    
    // In a real implementation, the server would keep running here
    // For now, we just show the configuration and exit
    println!("\n{}", "ℹ️  Server simulation complete. In production, this would run continuously.".blue());
    
    Ok(())
}