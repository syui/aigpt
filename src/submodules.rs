use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};
use colored::*;
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub async fn handle_submodules(
    action: String,
    module: Option<String>,
    all: bool,
    dry_run: bool,
    auto_commit: bool,
    verbose: bool,
    data_dir: Option<PathBuf>,
) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut submodule_manager = SubmoduleManager::new(config);
    
    match action.as_str() {
        "list" => {
            submodule_manager.list_submodules(verbose).await?;
        }
        "update" => {
            submodule_manager.update_submodules(module, all, dry_run, auto_commit, verbose).await?;
        }
        "status" => {
            submodule_manager.show_submodule_status().await?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown submodule action: {}", action));
        }
    }
    
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoduleInfo {
    pub name: String,
    pub path: String,
    pub branch: String,
    pub current_commit: Option<String>,
    pub target_commit: Option<String>,
    pub status: String,
}

impl Default for SubmoduleInfo {
    fn default() -> Self {
        SubmoduleInfo {
            name: String::new(),
            path: String::new(),
            branch: "main".to_string(),
            current_commit: None,
            target_commit: None,
            status: "unknown".to_string(),
        }
    }
}

pub struct SubmoduleManager {
    config: Config,
    ai_root: PathBuf,
    submodules: HashMap<String, SubmoduleInfo>,
}

impl SubmoduleManager {
    pub fn new(config: Config) -> Self {
        let ai_root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai")
            .join("ai");
        
        SubmoduleManager {
            config,
            ai_root,
            submodules: HashMap::new(),
        }
    }
    
    pub async fn list_submodules(&mut self, verbose: bool) -> Result<()> {
        println!("{}", "📋 Submodules Status".cyan().bold());
        println!();
        
        let submodules = self.parse_gitmodules()?;
        
        if submodules.is_empty() {
            println!("{}", "No submodules found".yellow());
            return Ok(());
        }
        
        // Display submodules in a table format
        println!("{:<15} {:<25} {:<15} {}", 
                "Module".cyan().bold(), 
                "Path".cyan().bold(), 
                "Branch".cyan().bold(), 
                "Status".cyan().bold());
        println!("{}", "-".repeat(80));
        
        for (module_name, module_info) in &submodules {
            let status_color = match module_info.status.as_str() {
                "clean" => module_info.status.green(),
                "modified" => module_info.status.yellow(),
                "missing" => module_info.status.red(),
                "conflicts" => module_info.status.red(),
                _ => module_info.status.normal(),
            };
            
            println!("{:<15} {:<25} {:<15} {}", 
                    module_name.blue(),
                    module_info.path,
                    module_info.branch.green(),
                    status_color);
        }
        
        println!();
        
        if verbose {
            println!("Total submodules: {}", submodules.len().to_string().cyan());
            println!("Repository root: {}", self.ai_root.display().to_string().blue());
        }
        
        Ok(())
    }
    
    pub async fn update_submodules(
        &mut self, 
        module: Option<String>, 
        all: bool, 
        dry_run: bool, 
        auto_commit: bool, 
        verbose: bool
    ) -> Result<()> {
        if !module.is_some() && !all {
            return Err(anyhow::anyhow!("Either --module or --all is required"));
        }
        
        if module.is_some() && all {
            return Err(anyhow::anyhow!("Cannot use both --module and --all"));
        }
        
        let submodules = self.parse_gitmodules()?;
        
        if submodules.is_empty() {
            println!("{}", "No submodules found".yellow());
            return Ok(());
        }
        
        // Determine which modules to update
        let modules_to_update: Vec<String> = if all {
            submodules.keys().cloned().collect()
        } else if let Some(module_name) = module {
            if !submodules.contains_key(&module_name) {
                return Err(anyhow::anyhow!(
                    "Submodule '{}' not found. Available modules: {}", 
                    module_name, 
                    submodules.keys().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
            vec![module_name]
        } else {
            vec![]
        };
        
        if dry_run {
            println!("{}", "🔍 DRY RUN MODE - No changes will be made".yellow().bold());
        }
        
        println!("{}", format!("🔄 Updating {} submodule(s)...", modules_to_update.len()).cyan().bold());
        
        let mut updated_modules = Vec::new();
        
        for module_name in modules_to_update {
            if let Some(module_info) = submodules.get(&module_name) {
                println!("\n{}", format!("📦 Processing: {}", module_name).blue().bold());
                
                let module_path = PathBuf::from(&module_info.path);
                let full_path = self.ai_root.join(&module_path);
                
                if !full_path.exists() {
                    println!("{}", format!("❌ Module directory not found: {}", module_info.path).red());
                    continue;
                }
                
                // Get current commit
                let current_commit = self.get_current_commit(&full_path)?;
                
                if dry_run {
                    println!("{}", format!("🔍 Would update {} to branch {}", module_name, module_info.branch).yellow());
                    if let Some(ref commit) = current_commit {
                        println!("{}", format!("Current: {}", commit).dimmed());
                    }
                    continue;
                }
                
                // Perform update
                if let Err(e) = self.update_single_module(&module_name, &module_info, &full_path).await {
                    println!("{}", format!("❌ Failed to update {}: {}", module_name, e).red());
                    continue;
                }
                
                // Get new commit
                let new_commit = self.get_current_commit(&full_path)?;
                
                if current_commit != new_commit {
                    println!("{}", format!("✅ Updated {} ({:?} → {:?})", 
                           module_name, 
                           current_commit.as_deref().unwrap_or("unknown"),
                           new_commit.as_deref().unwrap_or("unknown")).green());
                    updated_modules.push((module_name.clone(), current_commit, new_commit));
                } else {
                    println!("{}", "✅ Already up to date".green());
                }
            }
        }
        
        // Summary
        if !updated_modules.is_empty() {
            println!("\n{}", format!("🎉 Successfully updated {} module(s)", updated_modules.len()).green().bold());
            
            if verbose {
                for (module_name, old_commit, new_commit) in &updated_modules {
                    println!("  • {}: {:?} → {:?}", 
                           module_name,
                           old_commit.as_deref().unwrap_or("unknown"),
                           new_commit.as_deref().unwrap_or("unknown"));
                }
            }
            
            if auto_commit && !dry_run {
                self.auto_commit_changes(&updated_modules).await?;
            } else if !dry_run {
                println!("{}", "💾 Changes staged but not committed".yellow());
                println!("Run with --auto-commit to commit automatically");
            }
        } else if !dry_run {
            println!("{}", "No modules needed updating".yellow());
        }
        
        Ok(())
    }
    
    pub async fn show_submodule_status(&self) -> Result<()> {
        println!("{}", "📊 Submodule Status Overview".cyan().bold());
        println!();
        
        let submodules = self.parse_gitmodules()?;
        let mut total_modules = 0;
        let mut clean_modules = 0;
        let mut modified_modules = 0;
        let mut missing_modules = 0;
        
        for (module_name, module_info) in submodules {
            let module_path = self.ai_root.join(&module_info.path);
            
            if module_path.exists() {
                total_modules += 1;
                match module_info.status.as_str() {
                    "clean" => clean_modules += 1,
                    "modified" => modified_modules += 1,
                    _ => {}
                }
            } else {
                missing_modules += 1;
            }
            
            println!("{}: {}", 
                    module_name.blue(), 
                    if module_path.exists() {
                        module_info.status.green()
                    } else {
                        "missing".red()
                    });
        }
        
        println!();
        println!("Summary: {} total, {} clean, {} modified, {} missing", 
                total_modules.to_string().cyan(), 
                clean_modules.to_string().green(),
                modified_modules.to_string().yellow(),
                missing_modules.to_string().red());
        
        Ok(())
    }
    
    fn parse_gitmodules(&self) -> Result<HashMap<String, SubmoduleInfo>> {
        let gitmodules_path = self.ai_root.join(".gitmodules");
        
        if !gitmodules_path.exists() {
            return Ok(HashMap::new());
        }
        
        let content = std::fs::read_to_string(&gitmodules_path)
            .with_context(|| format!("Failed to read .gitmodules file: {}", gitmodules_path.display()))?;
        
        let mut submodules = HashMap::new();
        let mut current_name: Option<String> = None;
        let mut current_path: Option<String> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("[submodule \"") && line.ends_with("\"]") {
                // Save previous submodule if complete
                if let (Some(name), Some(path)) = (current_name.take(), current_path.take()) {
                    let mut info = SubmoduleInfo::default();
                    info.name = name.clone();
                    info.path = path;
                    info.branch = self.get_target_branch(&name);
                    info.status = self.get_submodule_status(&name, &info.path)?;
                    submodules.insert(name, info);
                }
                
                // Extract new submodule name
                current_name = Some(line[12..line.len()-2].to_string());
            } else if line.starts_with("path = ") {
                current_path = Some(line[7..].to_string());
            }
        }
        
        // Save last submodule
        if let (Some(name), Some(path)) = (current_name, current_path) {
            let mut info = SubmoduleInfo::default();
            info.name = name.clone();
            info.path = path;
            info.branch = self.get_target_branch(&name);
            info.status = self.get_submodule_status(&name, &info.path)?;
            submodules.insert(name, info);
        }
        
        Ok(submodules)
    }
    
    fn get_target_branch(&self, module_name: &str) -> String {
        // Try to get from ai.json configuration
        match module_name {
            "verse" => "main".to_string(),
            "card" => "main".to_string(),
            "bot" => "main".to_string(),
            _ => "main".to_string(),
        }
    }
    
    fn get_submodule_status(&self, _module_name: &str, module_path: &str) -> Result<String> {
        let full_path = self.ai_root.join(module_path);
        
        if !full_path.exists() {
            return Ok("missing".to_string());
        }
        
        // Check git status
        let output = std::process::Command::new("git")
            .args(&["submodule", "status", module_path])
            .current_dir(&self.ai_root)
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(status_char) = stdout.chars().next() {
                    match status_char {
                        ' ' => Ok("clean".to_string()),
                        '+' => Ok("modified".to_string()),
                        '-' => Ok("not_initialized".to_string()),
                        'U' => Ok("conflicts".to_string()),
                        _ => Ok("unknown".to_string()),
                    }
                } else {
                    Ok("unknown".to_string())
                }
            }
            _ => Ok("unknown".to_string())
        }
    }
    
    fn get_current_commit(&self, module_path: &PathBuf) -> Result<Option<String>> {
        let output = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(module_path)
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if commit.len() >= 8 {
                    Ok(Some(commit[..8].to_string()))
                } else {
                    Ok(Some(commit))
                }
            }
            _ => Ok(None)
        }
    }
    
    async fn update_single_module(
        &self, 
        _module_name: &str, 
        module_info: &SubmoduleInfo, 
        module_path: &PathBuf
    ) -> Result<()> {
        // Fetch latest changes
        println!("{}", "Fetching latest changes...".dimmed());
        let fetch_output = std::process::Command::new("git")
            .args(&["fetch", "origin"])
            .current_dir(module_path)
            .output()?;
        
        if !fetch_output.status.success() {
            return Err(anyhow::anyhow!("Failed to fetch: {}", 
                String::from_utf8_lossy(&fetch_output.stderr)));
        }
        
        // Switch to target branch
        println!("{}", format!("Switching to branch {}...", module_info.branch).dimmed());
        let checkout_output = std::process::Command::new("git")
            .args(&["checkout", &module_info.branch])
            .current_dir(module_path)
            .output()?;
        
        if !checkout_output.status.success() {
            return Err(anyhow::anyhow!("Failed to checkout {}: {}", 
                module_info.branch, String::from_utf8_lossy(&checkout_output.stderr)));
        }
        
        // Pull latest changes
        let pull_output = std::process::Command::new("git")
            .args(&["pull", "origin", &module_info.branch])
            .current_dir(module_path)
            .output()?;
        
        if !pull_output.status.success() {
            return Err(anyhow::anyhow!("Failed to pull: {}", 
                String::from_utf8_lossy(&pull_output.stderr)));
        }
        
        // Stage the submodule update
        let add_output = std::process::Command::new("git")
            .args(&["add", &module_info.path])
            .current_dir(&self.ai_root)
            .output()?;
        
        if !add_output.status.success() {
            return Err(anyhow::anyhow!("Failed to stage submodule: {}", 
                String::from_utf8_lossy(&add_output.stderr)));
        }
        
        Ok(())
    }
    
    async fn auto_commit_changes(&self, updated_modules: &[(String, Option<String>, Option<String>)]) -> Result<()> {
        println!("{}", "💾 Auto-committing changes...".blue());
        
        let mut commit_message = format!("Update submodules\n\n📦 Updated modules: {}\n", updated_modules.len());
        for (module_name, old_commit, new_commit) in updated_modules {
            commit_message.push_str(&format!(
                "- {}: {} → {}\n", 
                module_name,
                old_commit.as_deref().unwrap_or("unknown"),
                new_commit.as_deref().unwrap_or("unknown")
            ));
        }
        commit_message.push_str("\n🤖 Generated with aigpt-rs submodules update");
        
        let commit_output = std::process::Command::new("git")
            .args(&["commit", "-m", &commit_message])
            .current_dir(&self.ai_root)
            .output()?;
        
        if commit_output.status.success() {
            println!("{}", "✅ Changes committed successfully".green());
        } else {
            return Err(anyhow::anyhow!("Failed to commit: {}", 
                String::from_utf8_lossy(&commit_output.stderr)));
        }
        
        Ok(())
    }
}