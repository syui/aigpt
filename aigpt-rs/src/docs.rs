use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};
use colored::*;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::config::Config;
use crate::persona::Persona;
use crate::ai_provider::{AIProviderClient, AIConfig, AIProvider};

pub async fn handle_docs(
    action: String,
    project: Option<String>,
    output: Option<PathBuf>,
    ai_integration: bool,
    data_dir: Option<PathBuf>,
) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut docs_manager = DocsManager::new(config);
    
    match action.as_str() {
        "generate" => {
            if let Some(project_name) = project {
                docs_manager.generate_project_docs(&project_name, output, ai_integration).await?;
            } else {
                return Err(anyhow::anyhow!("Project name is required for generate action"));
            }
        }
        "sync" => {
            if let Some(project_name) = project {
                docs_manager.sync_project_docs(&project_name).await?;
            } else {
                docs_manager.sync_all_docs().await?;
            }
        }
        "list" => {
            docs_manager.list_projects().await?;
        }
        "status" => {
            docs_manager.show_docs_status().await?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown docs action: {}", action));
        }
    }
    
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub project_type: String,
    pub description: String,
    pub status: String,
    pub features: Vec<String>,
    pub dependencies: Vec<String>,
}

impl Default for ProjectInfo {
    fn default() -> Self {
        ProjectInfo {
            name: String::new(),
            project_type: String::new(),
            description: String::new(),
            status: "active".to_string(),
            features: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}

pub struct DocsManager {
    config: Config,
    ai_root: PathBuf,
    projects: HashMap<String, ProjectInfo>,
}

impl DocsManager {
    pub fn new(config: Config) -> Self {
        let ai_root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai")
            .join("ai");
        
        DocsManager {
            config,
            ai_root,
            projects: HashMap::new(),
        }
    }
    
    pub async fn generate_project_docs(&mut self, project: &str, output: Option<PathBuf>, ai_integration: bool) -> Result<()> {
        println!("{}", format!("📝 Generating documentation for project '{}'", project).cyan().bold());
        
        // Load project information
        let project_info = self.load_project_info(project)?;
        
        // Generate documentation content
        let mut content = self.generate_base_documentation(&project_info)?;
        
        // AI enhancement if requested
        if ai_integration {
            println!("{}", "🤖 Enhancing documentation with AI...".blue());
            if let Ok(enhanced_content) = self.enhance_with_ai(project, &content).await {
                content = enhanced_content;
            } else {
                println!("{}", "Warning: AI enhancement failed, using base documentation".yellow());
            }
        }
        
        // Determine output path
        let output_path = if let Some(path) = output {
            path
        } else {
            self.ai_root.join(project).join("claude.md")
        };
        
        // Ensure directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        // Write documentation
        std::fs::write(&output_path, content)
            .with_context(|| format!("Failed to write documentation to: {}", output_path.display()))?;
        
        println!("{}", format!("✅ Documentation generated: {}", output_path.display()).green().bold());
        
        Ok(())
    }
    
    pub async fn sync_project_docs(&self, project: &str) -> Result<()> {
        println!("{}", format!("🔄 Syncing documentation for project '{}'", project).cyan().bold());
        
        let claude_dir = self.ai_root.join("claude");
        let project_dir = self.ai_root.join(project);
        
        // Check if claude directory exists
        if !claude_dir.exists() {
            return Err(anyhow::anyhow!("Claude directory not found: {}", claude_dir.display()));
        }
        
        // Copy relevant files
        let files_to_sync = vec!["README.md", "claude.md", "DEVELOPMENT.md"];
        
        for file in files_to_sync {
            let src = claude_dir.join("projects").join(format!("{}.md", project));
            let dst = project_dir.join(file);
            
            if src.exists() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&src, &dst)?;
                println!("  ✓ Synced: {}", file.green());
            }
        }
        
        println!("{}", "✅ Documentation sync completed".green().bold());
        
        Ok(())
    }
    
    pub async fn sync_all_docs(&self) -> Result<()> {
        println!("{}", "🔄 Syncing documentation for all projects...".cyan().bold());
        
        // Find all project directories
        let projects = self.discover_projects()?;
        
        for project in projects {
            println!("\n{}", format!("Syncing: {}", project).blue());
            if let Err(e) = self.sync_project_docs(&project).await {
                println!("{}: {}", "Warning".yellow(), e);
            }
        }
        
        println!("\n{}", "✅ All projects synced".green().bold());
        
        Ok(())
    }
    
    pub async fn list_projects(&mut self) -> Result<()> {
        println!("{}", "📋 Available Projects".cyan().bold());
        println!();
        
        let projects = self.discover_projects()?;
        
        if projects.is_empty() {
            println!("{}", "No projects found".yellow());
            return Ok(());
        }
        
        // Load project information
        for project in &projects {
            if let Ok(info) = self.load_project_info(project) {
                self.projects.insert(project.clone(), info);
            }
        }
        
        // Display projects in a table format
        println!("{:<20} {:<15} {:<15} {}", 
                "Project".cyan().bold(), 
                "Type".cyan().bold(), 
                "Status".cyan().bold(), 
                "Description".cyan().bold());
        println!("{}", "-".repeat(80));
        
        let project_count = projects.len();
        for project in &projects {
            let info = self.projects.get(project).cloned().unwrap_or_default();
            let status_color = match info.status.as_str() {
                "active" => info.status.green(),
                "development" => info.status.yellow(),
                "deprecated" => info.status.red(),
                _ => info.status.normal(),
            };
            
            println!("{:<20} {:<15} {:<15} {}", 
                    project.blue(),
                    info.project_type,
                    status_color,
                    info.description);
        }
        
        println!();
        println!("Total projects: {}", project_count.to_string().cyan());
        
        Ok(())
    }
    
    pub async fn show_docs_status(&self) -> Result<()> {
        println!("{}", "📊 Documentation Status".cyan().bold());
        println!();
        
        let projects = self.discover_projects()?;
        let mut total_files = 0;
        let mut total_lines = 0;
        
        for project in projects {
            let project_dir = self.ai_root.join(&project);
            let claude_md = project_dir.join("claude.md");
            
            if claude_md.exists() {
                let content = std::fs::read_to_string(&claude_md)?;
                let lines = content.lines().count();
                let size = content.len();
                
                println!("{}: {} lines, {} bytes", 
                        project.blue(), 
                        lines.to_string().yellow(), 
                        size.to_string().yellow());
                
                total_files += 1;
                total_lines += lines;
            } else {
                println!("{}: {}", project.blue(), "No documentation".red());
            }
        }
        
        println!();
        println!("Summary: {} files, {} total lines", 
                total_files.to_string().cyan(), 
                total_lines.to_string().cyan());
        
        Ok(())
    }
    
    fn discover_projects(&self) -> Result<Vec<String>> {
        let mut projects = Vec::new();
        
        // Known project directories
        let known_projects = vec![
            "gpt", "card", "bot", "shell", "os", "game", "moji", "verse"
        ];
        
        for project in known_projects {
            let project_dir = self.ai_root.join(project);
            if project_dir.exists() && project_dir.is_dir() {
                projects.push(project.to_string());
            }
        }
        
        // Also scan for additional directories with ai.json
        if self.ai_root.exists() {
            for entry in std::fs::read_dir(&self.ai_root)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    let ai_json = path.join("ai.json");
                    if ai_json.exists() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if !projects.contains(&name.to_string()) {
                                projects.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        projects.sort();
        Ok(projects)
    }
    
    fn load_project_info(&self, project: &str) -> Result<ProjectInfo> {
        let ai_json_path = self.ai_root.join(project).join("ai.json");
        
        if ai_json_path.exists() {
            let content = std::fs::read_to_string(&ai_json_path)?;
            if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut info = ProjectInfo::default();
                info.name = project.to_string();
                
                if let Some(project_data) = json_data.get(project) {
                    if let Some(type_str) = project_data.get("type").and_then(|v| v.as_str()) {
                        info.project_type = type_str.to_string();
                    }
                    if let Some(desc) = project_data.get("description").and_then(|v| v.as_str()) {
                        info.description = desc.to_string();
                    }
                }
                
                return Ok(info);
            }
        }
        
        // Default project info based on known projects
        let mut info = ProjectInfo::default();
        info.name = project.to_string();
        
        match project {
            "gpt" => {
                info.project_type = "AI".to_string();
                info.description = "Autonomous transmission AI with unique personality".to_string();
            }
            "card" => {
                info.project_type = "Game".to_string();
                info.description = "Card game system with atproto integration".to_string();
            }
            "bot" => {
                info.project_type = "Bot".to_string();
                info.description = "Distributed SNS bot for AI ecosystem".to_string();
            }
            "shell" => {
                info.project_type = "Tool".to_string();
                info.description = "AI-powered shell interface".to_string();
            }
            "os" => {
                info.project_type = "OS".to_string();
                info.description = "Game-oriented operating system".to_string();
            }
            "verse" => {
                info.project_type = "Metaverse".to_string();
                info.description = "Reality-reflecting 3D world system".to_string();
            }
            _ => {
                info.project_type = "Unknown".to_string();
                info.description = format!("AI ecosystem project: {}", project);
            }
        }
        
        Ok(info)
    }
    
    fn generate_base_documentation(&self, project_info: &ProjectInfo) -> Result<String> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        
        let mut content = String::new();
        content.push_str(&format!("# {}\n\n", project_info.name));
        content.push_str(&format!("## Overview\n\n"));
        content.push_str(&format!("**Type**: {}\n\n", project_info.project_type));
        content.push_str(&format!("**Description**: {}\n\n", project_info.description));
        content.push_str(&format!("**Status**: {}\n\n", project_info.status));
        
        if !project_info.features.is_empty() {
            content.push_str("## Features\n\n");
            for feature in &project_info.features {
                content.push_str(&format!("- {}\n", feature));
            }
            content.push_str("\n");
        }
        
        content.push_str("## Architecture\n\n");
        content.push_str("This project is part of the ai ecosystem, following the core principles:\n\n");
        content.push_str("- **Existence Theory**: Based on the exploration of the smallest units (ai/existon)\n");
        content.push_str("- **Uniqueness Principle**: Ensuring 1:1 mapping between reality and digital existence\n");
        content.push_str("- **Reality Reflection**: Creating circular influence between reality and game\n\n");
        
        content.push_str("## Development\n\n");
        content.push_str("### Getting Started\n\n");
        content.push_str("```bash\n");
        content.push_str(&format!("# Clone the repository\n"));
        content.push_str(&format!("git clone https://git.syui.ai/ai/{}\n", project_info.name));
        content.push_str(&format!("cd {}\n", project_info.name));
        content.push_str("```\n\n");
        
        content.push_str("### Configuration\n\n");
        content.push_str(&format!("Configuration files are stored in `~/.config/syui/ai/{}/`\n\n", project_info.name));
        
        content.push_str("## Integration\n\n");
        content.push_str("This project integrates with other ai ecosystem components:\n\n");
        if !project_info.dependencies.is_empty() {
            for dep in &project_info.dependencies {
                content.push_str(&format!("- **{}**: Core dependency\n", dep));
            }
        } else {
            content.push_str("- **ai.gpt**: Core AI personality system\n");
            content.push_str("- **atproto**: Distributed identity and data\n");
        }
        content.push_str("\n");
        
        content.push_str("---\n\n");
        content.push_str(&format!("*Generated: {}*\n", timestamp));
        content.push_str("*🤖 Generated with [Claude Code](https://claude.ai/code)*\n");
        
        Ok(content)
    }
    
    async fn enhance_with_ai(&self, project: &str, base_content: &str) -> Result<String> {
        // Create AI provider
        let ai_config = AIConfig {
            provider: AIProvider::Ollama,
            model: "llama2".to_string(),
            api_key: None,
            base_url: None,
            max_tokens: Some(2000),
            temperature: Some(0.7),
        };
        
        let _ai_provider = AIProviderClient::new(ai_config);
        let mut persona = Persona::new(&self.config)?;
        
        let enhancement_prompt = format!(
            "As an AI documentation expert, enhance the following documentation for project '{}'. 
            
            Current documentation:
            {}
            
            Please provide enhanced content that includes:
            1. More detailed project description
            2. Key features and capabilities
            3. Usage examples
            4. Integration points with other AI ecosystem projects
            5. Development workflow recommendations
            
            Keep the same structure but expand and improve the content.",
            project, base_content
        );
        
        // Try to get AI response
        let (response, _) = persona.process_ai_interaction(
            "docs_system", 
            &enhancement_prompt,
            Some("ollama".to_string()),
            Some("llama2".to_string())
        ).await?;
        
        // If AI response is substantial, use it; otherwise fall back to base content
        if response.len() > base_content.len() / 2 {
            Ok(response)
        } else {
            Ok(base_content.to_string())
        }
    }
}