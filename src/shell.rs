use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{self, Write};
use anyhow::{Result, Context};
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::history::{History, DefaultHistory};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;

use crate::config::Config;
use crate::persona::Persona;
use crate::ai_provider::{AIProviderClient, AIProvider, AIConfig};

pub async fn handle_shell(
    user_id: String,
    data_dir: Option<PathBuf>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<()> {
    let config = Config::new(data_dir)?;
    
    let mut shell = ShellMode::new(config, user_id)?
        .with_ai_provider(provider, model);
    
    shell.run().await
}

pub struct ShellMode {
    config: Config,
    persona: Persona,
    ai_provider: Option<AIProviderClient>,
    user_id: String,
    editor: Editor<ShellCompleter, DefaultHistory>,
}

struct ShellCompleter {
    completer: FilenameCompleter,
}

impl ShellCompleter {
    fn new() -> Self {
        ShellCompleter {
            completer: FilenameCompleter::new(),
        }
    }
}

impl Helper for ShellCompleter {}

impl Hinter for ShellCompleter {
    type Hint = String;
    
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for ShellCompleter {}

impl Validator for ShellCompleter {}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Custom completion for slash commands
        if line.starts_with('/') {
            let commands = vec![
                "/status", "/relationships", "/memories", "/analyze", 
                "/fortune", "/clear", "/history", "/help", "/exit"
            ];
            
            let word_start = line.rfind(' ').map_or(0, |i| i + 1);
            let word = &line[word_start..pos];
            
            let matches: Vec<Pair> = commands.iter()
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();
            
            return Ok((word_start, matches));
        }
        
        // Custom completion for shell commands starting with !
        if line.starts_with('!') {
            let shell_commands = vec![
                "ls", "pwd", "cd", "cat", "grep", "find", "ps", "top", 
                "echo", "mkdir", "rmdir", "cp", "mv", "rm", "touch",
                "git", "cargo", "npm", "python", "node"
            ];
            
            let word_start = line.rfind(' ').map_or(1, |i| i + 1); // Skip the '!'
            let word = &line[word_start..pos];
            
            let matches: Vec<Pair> = shell_commands.iter()
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();
            
            return Ok((word_start, matches));
        }
        
        // Fallback to filename completion
        self.completer.complete(line, pos, ctx)
    }
}

impl ShellMode {
    pub fn new(config: Config, user_id: String) -> Result<Self> {
        let persona = Persona::new(&config)?;
        
        // Setup rustyline editor with completer
        let completer = ShellCompleter::new();
        let mut editor = Editor::with_config(
            rustyline::Config::builder()
                .tab_stop(4)
                .build()
        )?;
        editor.set_helper(Some(completer));
        
        // Load history if exists
        let history_file = config.data_dir.join("shell_history.txt");
        if history_file.exists() {
            let _ = editor.load_history(&history_file);
        }
        
        Ok(ShellMode {
            config,
            persona,
            ai_provider: None,
            user_id,
            editor,
        })
    }
    
    pub fn with_ai_provider(mut self, provider: Option<String>, model: Option<String>) -> Self {
        // Use provided parameters or fall back to config defaults
        let provider_name = provider
            .or_else(|| Some(self.config.default_provider.clone()))
            .unwrap_or_else(|| "ollama".to_string());
            
        let model_name = model.or_else(|| {
            // Try to get default model from config for the chosen provider
            self.config.providers.get(&provider_name)
                .map(|p| p.default_model.clone())
        }).unwrap_or_else(|| {
            // Final fallback based on provider
            match provider_name.as_str() {
                "openai" => "gpt-4o-mini".to_string(),
                "ollama" => "qwen2.5-coder:latest".to_string(),
                _ => "qwen2.5-coder:latest".to_string(),
            }
        });

        let ai_provider = match provider_name.as_str() {
            "ollama" => AIProvider::Ollama,
            "openai" => AIProvider::OpenAI,
            "claude" => AIProvider::Claude,
            _ => AIProvider::Ollama, // Default fallback
        };
        
        let ai_config = AIConfig {
            provider: ai_provider,
            model: model_name,
            api_key: None, // Will be loaded from environment if needed
            base_url: None,
            max_tokens: Some(2000),
            temperature: Some(0.7),
        };
        
        let client = AIProviderClient::new(ai_config);
        self.ai_provider = Some(client);
        
        self
    }
    
    pub async fn run(&mut self) -> Result<()> {
        println!("{}", "🚀 Starting ai.gpt Interactive Shell".cyan().bold());
        
        // Show AI provider info
        if let Some(ai_provider) = &self.ai_provider {
            println!("{}: {} ({})", 
                "AI Provider".green().bold(), 
                ai_provider.get_provider().to_string(),
                ai_provider.get_model());
        } else {
            println!("{}: {}", "AI Provider".yellow().bold(), "Simple mode (no AI)");
        }
        
        println!("{}", "Type 'help' for commands, 'exit' to quit".dimmed());
        println!("{}", "Use Tab for command completion, Ctrl+C to interrupt, Ctrl+D to exit".dimmed());
        
        loop {
            // Read user input with rustyline (supports completion, history, etc.)
            let readline = self.editor.readline("ai.shell> ");
            
            match readline {
                Ok(line) => {
                    let input = line.trim();
                    
                    // Skip empty input
                    if input.is_empty() {
                        continue;
                    }
                    
                    // Add to history
                    self.editor.add_history_entry(input)
                        .context("Failed to add to history")?;
                    
                    // Handle input
                    if let Err(e) = self.handle_input(input).await {
                        println!("{}: {}", "Error".red().bold(), e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C
                    println!("{}",  "Use 'exit' or Ctrl+D to quit".yellow());
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D
                    println!("\n{}", "Goodbye!".cyan());
                    break;
                }
                Err(err) => {
                    println!("{}: {}", "Input error".red().bold(), err);
                    break;
                }
            }
        }
        
        // Save history before exit
        self.save_history()?;
        
        Ok(())
    }
    
    async fn handle_input(&mut self, input: &str) -> Result<()> {
        match input {
            // Exit commands
            "exit" | "quit" | "/exit" | "/quit" => {
                println!("{}", "Goodbye!".cyan());
                std::process::exit(0);
            }
            // Help command
            "help" | "/help" => {
                self.show_help();
            }
            // Shell commands (starting with !)
            input if input.starts_with('!') => {
                self.execute_shell_command(&input[1..]).await?;
            }
            // Slash commands (starting with /)
            input if input.starts_with('/') => {
                self.execute_slash_command(input).await?;
            }
            // AI conversation
            _ => {
                self.handle_ai_conversation(input).await?;
            }
        }
        
        Ok(())
    }
    
    fn show_help(&self) {
        println!("\n{}", "ai.gpt Interactive Shell Commands".cyan().bold());
        println!();
        
        println!("{}", "Navigation & Input:".yellow().bold());
        println!("  {} - Tab completion for commands and files", "Tab".green());
        println!("  {} - Command history (previous/next)", "↑/↓ or Ctrl+P/N".green());
        println!("  {} - Interrupt current input", "Ctrl+C".green());
        println!("  {} - Exit shell", "Ctrl+D".green());
        println!();
        
        println!("{}", "Basic Commands:".yellow().bold());
        println!("  {} - Show this help", "help".green());
        println!("  {} - Exit the shell", "exit, quit".green());
        println!("  {} - Clear screen", "/clear".green());
        println!("  {} - Show command history", "/history".green());
        println!();
        
        println!("{}", "Shell Commands:".yellow().bold());
        println!("  {} - Execute shell command (Tab completion)", "!<command>".green());
        println!("  {} - List files", "!ls".green());
        println!("  {} - Show current directory", "!pwd".green());
        println!("  {} - Git status", "!git status".green());
        println!("  {} - Cargo build", "!cargo build".green());
        println!();
        
        println!("{}", "AI Commands:".yellow().bold());
        println!("  {} - Show AI status and relationship", "/status".green());
        println!("  {} - List all relationships", "/relationships".green());
        println!("  {} - Show recent memories", "/memories".green());
        println!("  {} - Analyze current directory", "/analyze".green());
        println!("  {} - Show today's fortune", "/fortune".green());
        println!();
        
        println!("{}", "Conversation:".yellow().bold());
        println!("  {} - Chat with AI using configured provider", "Any other input".green());
        println!("  {} - AI responses track relationship changes", "Relationship tracking".dimmed());
        println!();
    }
    
    async fn execute_shell_command(&self, command: &str) -> Result<()> {
        println!("{} {}", "Executing:".blue().bold(), command.yellow());
        
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .context("Failed to execute command")?
        } else {
            Command::new("sh")
                .args(["-c", command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .context("Failed to execute command")?
        };
        
        // Print stdout
        if !output.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("{}", stdout);
        }
        
        // Print stderr in red
        if !output.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{}", stderr.red());
        }
        
        // Show exit code if not successful
        if !output.status.success() {
            if let Some(code) = output.status.code() {
                println!("{}: {}", "Exit code".red().bold(), code);
            }
        }
        
        Ok(())
    }
    
    async fn execute_slash_command(&mut self, command: &str) -> Result<()> {
        match command {
            "/status" => {
                self.show_ai_status().await?;
            }
            "/relationships" => {
                self.show_relationships().await?;
            }
            "/memories" => {
                self.show_memories().await?;
            }
            "/analyze" => {
                self.analyze_directory().await?;
            }
            "/fortune" => {
                self.show_fortune().await?;
            }
            "/clear" => {
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush()?;
            }
            "/history" => {
                self.show_history();
            }
            _ => {
                println!("{}: {}", "Unknown command".red().bold(), command);
                println!("Type '{}' for available commands", "help".green());
            }
        }
        
        Ok(())
    }
    
    async fn handle_ai_conversation(&mut self, input: &str) -> Result<()> {
        let (response, relationship_delta) = if let Some(ai_provider) = &self.ai_provider {
            // Use AI provider for response
            self.persona.process_ai_interaction(&self.user_id, input, 
                Some(ai_provider.get_provider().to_string()), 
                Some(ai_provider.get_model().to_string())).await?
        } else {
            // Use simple response
            self.persona.process_interaction(&self.user_id, input)?
        };
        
        // Display conversation
        println!("{}: {}", "You".cyan().bold(), input);
        println!("{}: {}", "AI".green().bold(), response);
        
        // Show relationship change if significant
        if relationship_delta.abs() >= 0.1 {
            if relationship_delta > 0.0 {
                println!("{}", format!("(+{:.2} relationship)", relationship_delta).green());
            } else {
                println!("{}", format!("({:.2} relationship)", relationship_delta).red());
            }
        }
        
        println!(); // Add spacing
        
        Ok(())
    }
    
    async fn show_ai_status(&self) -> Result<()> {
        let state = self.persona.get_current_state()?;
        
        println!("\n{}", "AI Status".cyan().bold());
        println!("Mood: {}", state.current_mood.yellow());
        println!("Fortune: {}/10", state.fortune_value.to_string().yellow());
        
        if let Some(relationship) = self.persona.get_relationship(&self.user_id) {
            println!("\n{}", "Your Relationship".cyan().bold());
            println!("Status: {}", relationship.status.to_string().yellow());
            println!("Score: {:.2} / {}", relationship.score, relationship.threshold);
            println!("Interactions: {}", relationship.total_interactions);
        }
        
        println!();
        Ok(())
    }
    
    async fn show_relationships(&self) -> Result<()> {
        let relationships = self.persona.list_all_relationships();
        
        if relationships.is_empty() {
            println!("{}", "No relationships yet".yellow());
            return Ok(());
        }
        
        println!("\n{}", "All Relationships".cyan().bold());
        println!();
        
        for (user_id, rel) in relationships {
            let transmission = if rel.is_broken {
                "💔"
            } else if rel.transmission_enabled {
                "✓"
            } else {
                "✗"
            };
            
            let user_display = if user_id.len() > 20 {
                format!("{}...", &user_id[..20])
            } else {
                user_id
            };
            
            println!("{:<25} {:<12} {:<8} {}", 
                    user_display.cyan(),
                    rel.status.to_string(),
                    format!("{:.2}", rel.score),
                    transmission);
        }
        
        println!();
        Ok(())
    }
    
    async fn show_memories(&mut self) -> Result<()> {
        let memories = self.persona.get_memories(&self.user_id, 10);
        
        if memories.is_empty() {
            println!("{}", "No memories yet".yellow());
            return Ok(());
        }
        
        println!("\n{}", "Recent Memories".cyan().bold());
        println!();
        
        for (i, memory) in memories.iter().enumerate() {
            println!("{}: {}", 
                     format!("Memory {}", i + 1).dimmed(),
                     memory);
            println!();
        }
        
        Ok(())
    }
    
    async fn analyze_directory(&self) -> Result<()> {
        println!("{}", "Analyzing current directory...".blue().bold());
        
        // Get current directory
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory")?;
        
        println!("Directory: {}", current_dir.display().to_string().yellow());
        
        // List files and directories
        let entries = std::fs::read_dir(&current_dir)
            .context("Failed to read directory")?;
        
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            
            if path.is_dir() {
                dirs.push(name.to_string());
            } else {
                files.push(name.to_string());
            }
        }
        
        if !dirs.is_empty() {
            println!("\n{}: {}", "Directories".blue().bold(), dirs.join(", "));
        }
        
        if !files.is_empty() {
            println!("{}: {}", "Files".blue().bold(), files.join(", "));
        }
        
        // Check for common project files
        let project_files = ["Cargo.toml", "package.json", "requirements.txt", "Makefile", "README.md"];
        let found_files: Vec<_> = project_files.iter()
            .filter(|&&file| files.contains(&file.to_string()))
            .collect();
        
        if !found_files.is_empty() {
            println!("\n{}: {}", "Project files detected".green().bold(), 
                     found_files.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "));
        }
        
        println!();
        Ok(())
    }
    
    async fn show_fortune(&self) -> Result<()> {
        let state = self.persona.get_current_state()?;
        
        let fortune_stars = "🌟".repeat(state.fortune_value as usize);
        let empty_stars = "☆".repeat((10 - state.fortune_value) as usize);
        
        println!("\n{}", "AI Fortune".yellow().bold());
        println!("{}{}", fortune_stars, empty_stars);
        println!("Today's Fortune: {}/10", state.fortune_value);
        
        if state.breakthrough_triggered {
            println!("{}", "⚡ BREAKTHROUGH! Special fortune activated!".yellow());
        }
        
        println!();
        Ok(())
    }
    
    fn show_history(&self) {
        println!("\n{}", "Command History".cyan().bold());
        
        let history = self.editor.history();
        if history.is_empty() {
            println!("{}", "No commands in history".yellow());
            return;
        }
        
        // Show last 20 commands
        let start = if history.len() > 20 { history.len() - 20 } else { 0 };
        for (i, entry) in history.iter().enumerate().skip(start) {
            println!("{:2}: {}", i + 1, entry);
        }
        
        println!();
    }
    
    fn save_history(&mut self) -> Result<()> {
        let history_file = self.config.data_dir.join("shell_history.txt");
        self.editor.save_history(&history_file)
            .context("Failed to save shell history")?;
        Ok(())
    }
}

// Extend AIProvider to have Display and helper methods
impl AIProvider {
    fn to_string(&self) -> String {
        match self {
            AIProvider::OpenAI => "openai".to_string(),
            AIProvider::Ollama => "ollama".to_string(),
            AIProvider::Claude => "claude".to_string(),
        }
    }
}