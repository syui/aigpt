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
        "session-end" => {
            docs_manager.session_end_processing().await?;
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
        
        // Generate ai.wiki content after all project syncs
        println!("\n{}", "📝 Updating ai.wiki...".blue());
        if let Err(e) = self.update_ai_wiki().await {
            println!("{}: Failed to update ai.wiki: {}", "Warning".yellow(), e);
        }
        
        // Update repository wiki (Gitea wiki) as well
        println!("\n{}", "📝 Updating repository wiki...".blue());
        if let Err(e) = self.update_repository_wiki().await {
            println!("{}: Failed to update repository wiki: {}", "Warning".yellow(), e);
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
    
    /// セッション終了時の処理（ドキュメント記録・同期）
    pub async fn session_end_processing(&mut self) -> Result<()> {
        println!("{}", "🔄 Session end processing started...".cyan());
        
        // 1. 現在のプロジェクト状況を記録
        println!("📊 Recording current project status...");
        self.record_session_summary().await?;
        
        // 2. 全プロジェクトのドキュメント同期
        println!("🔄 Syncing all project documentation...");
        self.sync_all_docs().await?;
        
        // 3. READMEの自動更新
        println!("📝 Updating project README files...");
        self.update_project_readmes().await?;
        
        // 4. メタデータの更新
        println!("🏷️  Updating project metadata...");
        self.update_project_metadata().await?;
        
        println!("{}", "✅ Session end processing completed!".green());
        Ok(())
    }
    
    /// セッション概要を記録
    async fn record_session_summary(&self) -> Result<()> {
        let session_log_path = self.ai_root.join("session_logs");
        std::fs::create_dir_all(&session_log_path)?;
        
        let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let log_file = session_log_path.join(format!("session_{}.md", timestamp));
        
        let summary = format!(
            "# Session Summary - {}\n\n\
            ## Timestamp\n{}\n\n\
            ## Projects Status\n{}\n\n\
            ## Next Actions\n- Documentation sync completed\n- README files updated\n- Metadata refreshed\n\n\
            ---\n*Generated by aigpt session-end processing*\n",
            timestamp,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            self.generate_projects_status().await.unwrap_or_else(|_| "Status unavailable".to_string())
        );
        
        std::fs::write(log_file, summary)?;
        Ok(())
    }
    
    /// プロジェクト状況を生成
    async fn generate_projects_status(&self) -> Result<String> {
        let projects = self.discover_projects()?;
        let mut status = String::new();
        
        for project in projects {
            let claude_md = self.ai_root.join(&project).join("claude.md");
            let readme_md = self.ai_root.join(&project).join("README.md");
            
            status.push_str(&format!("- **{}**: ", project));
            if claude_md.exists() {
                status.push_str("claude.md ✅ ");
            } else {
                status.push_str("claude.md ❌ ");
            }
            if readme_md.exists() {
                status.push_str("README.md ✅");
            } else {
                status.push_str("README.md ❌");
            }
            status.push('\n');
        }
        
        Ok(status)
    }
    
    /// ai.wikiの更新処理
    async fn update_ai_wiki(&self) -> Result<()> {
        let ai_wiki_path = self.ai_root.join("ai.wiki");
        
        // ai.wikiディレクトリが存在することを確認
        if !ai_wiki_path.exists() {
            return Err(anyhow::anyhow!("ai.wiki directory not found at {:?}", ai_wiki_path));
        }
        
        // Home.mdの生成
        let home_content = self.generate_wiki_home_content().await?;
        let home_path = ai_wiki_path.join("Home.md");
        std::fs::write(&home_path, &home_content)?;
        println!("  ✓ Updated: {}", "Home.md".green());
        
        // title.mdの生成 (Gitea wiki特別ページ用)
        let title_path = ai_wiki_path.join("title.md");
        std::fs::write(&title_path, &home_content)?;
        println!("  ✓ Updated: {}", "title.md".green());
        
        // プロジェクト個別ディレクトリの更新
        let projects = self.discover_projects()?;
        for project in projects {
            let project_dir = ai_wiki_path.join(&project);
            std::fs::create_dir_all(&project_dir)?;
            
            let project_content = self.generate_auto_project_content(&project).await?;
            let project_file = project_dir.join(format!("{}.md", project));
            std::fs::write(&project_file, project_content)?;
            println!("  ✓ Updated: {}", format!("{}/{}.md", project, project).green());
        }
        
        println!("{}", "✅ ai.wiki updated successfully".green().bold());
        Ok(())
    }
    
    /// ai.wiki/Home.mdのコンテンツ生成
    async fn generate_wiki_home_content(&self) -> Result<String> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        let mut content = String::new();
        
        content.push_str("# AI Ecosystem Wiki\n\n");
        content.push_str("AI生態系プロジェクトの概要とドキュメント集約ページです。\n\n");
        content.push_str("## プロジェクト一覧\n\n");
        
        let projects = self.discover_projects()?;
        let mut project_sections = std::collections::HashMap::new();
        
        // プロジェクトをカテゴリ別に分類
        for project in &projects {
            let info = self.load_project_info(project).unwrap_or_default();
            let category = match project.as_str() {
                "ai" => "🧠 AI・知能システム",
                "gpt" => "🤖 自律・対話システム", 
                "os" => "💻 システム・基盤",
                "game" => "📁 device",
                "card" => "🎮 ゲーム・エンターテイメント",
                "bot" | "moji" | "api" | "log" => "📁 その他",
                "verse" => "📁 metaverse",
                "shell" => "⚡ ツール・ユーティリティ",
                _ => "📁 その他",
            };
            
            project_sections.entry(category).or_insert_with(Vec::new).push((project.clone(), info));
        }
        
        // カテゴリ別にプロジェクトを出力
        let mut categories: Vec<_> = project_sections.keys().collect();
        categories.sort();
        
        for category in categories {
            content.push_str(&format!("### {}\n\n", category));
            
            if let Some(projects_in_category) = project_sections.get(category) {
                for (project, info) in projects_in_category {
                    content.push_str(&format!("#### [{}]({}.md)\n", project, project));
                    
                    if !info.description.is_empty() {
                        content.push_str(&format!("- **名前**: ai.{} - **パッケージ**: ai{} - **タイプ**: {} - **役割**: {}\n\n", 
                                                project, project, info.project_type, info.description));
                    }
                    
                    content.push_str(&format!("**Status**: {}  \n", info.status));
                    let branch = self.get_project_branch(project);
                    content.push_str(&format!("**Links**: [Repo](https://git.syui.ai/ai/{}) | [Docs](https://git.syui.ai/ai/{}/src/branch/{}/claude.md)\n\n", project, project, branch));
                }
            }
        }
        
        content.push_str("---\n\n");
        content.push_str("## ディレクトリ構成\n\n");
        content.push_str("- `{project}/` - プロジェクト個別ドキュメント\n");
        content.push_str("- `claude/` - Claude Code作業記録\n");
        content.push_str("- `manual/` - 手動作成ドキュメント\n\n");
        content.push_str("---\n\n");
        content.push_str("*このページは ai.json と claude/projects/ から自動生成されました*  \n");
        content.push_str(&format!("*最終更新: {}*\n", timestamp));
        
        Ok(content)
    }
    
    /// プロジェクト個別ファイルのコンテンツ生成
    async fn generate_auto_project_content(&self, project: &str) -> Result<String> {
        let info = self.load_project_info(project).unwrap_or_default();
        let mut content = String::new();
        
        content.push_str(&format!("# {}\n\n", project));
        content.push_str("## 概要\n");
        content.push_str(&format!("- **名前**: ai.{} - **パッケージ**: ai{} - **タイプ**: {} - **役割**: {}\n\n", 
                                project, project, info.project_type, info.description));
        
        content.push_str("## プロジェクト情報\n");
        content.push_str(&format!("- **タイプ**: {}\n", info.project_type));
        content.push_str(&format!("- **説明**: {}\n", info.description));
        content.push_str(&format!("- **ステータス**: {}\n", info.status));
        let branch = self.get_project_branch(project);
        content.push_str(&format!("- **ブランチ**: {}\n", branch));
        content.push_str("- **最終更新**: Unknown\n\n");
        
        // プロジェクト固有の機能情報を追加
        if !info.features.is_empty() {
            content.push_str("## 主な機能・特徴\n");
            for feature in &info.features {
                content.push_str(&format!("- {}\n", feature));
            }
            content.push_str("\n");
        }
        
        content.push_str("## リンク\n");
        content.push_str(&format!("- **Repository**: https://git.syui.ai/ai/{}\n", project));
        content.push_str(&format!("- **Project Documentation**: [claude/projects/{}.md](https://git.syui.ai/ai/ai/src/branch/main/claude/projects/{}.md)\n", project, project));
        let branch = self.get_project_branch(project);
        content.push_str(&format!("- **Generated Documentation**: [{}/claude.md](https://git.syui.ai/ai/{}/src/branch/{}/claude.md)\n\n", project, project, branch));
        
        content.push_str("---\n");
        content.push_str(&format!("*このページは claude/projects/{}.md から自動生成されました*\n", project));
        
        Ok(content)
    }
    
    /// リポジトリwiki (Gitea wiki) の更新処理
    async fn update_repository_wiki(&self) -> Result<()> {
        println!("  ℹ️ Repository wiki is now unified with ai.wiki");
        println!("  ℹ️ ai.wiki serves as the source of truth (git@git.syui.ai:ai/ai.wiki.git)");
        println!("  ℹ️ Special pages generated: Home.md, title.md for Gitea wiki compatibility");
        
        Ok(())
    }

    /// プロジェクトREADMEファイルの更新
    async fn update_project_readmes(&self) -> Result<()> {
        let projects = self.discover_projects()?;
        
        for project in projects {
            let readme_path = self.ai_root.join(&project).join("README.md");
            let claude_md_path = self.ai_root.join(&project).join("claude.md");
            
            // claude.mdが存在する場合、READMEに同期
            if claude_md_path.exists() {
                let claude_content = std::fs::read_to_string(&claude_md_path)?;
                
                // READMEが存在しない場合は新規作成
                if !readme_path.exists() {
                    println!("📝 Creating README.md for {}", project);
                    std::fs::write(&readme_path, &claude_content)?;
                } else {
                    // 既存READMEがclaude.mdより古い場合は更新
                    let readme_metadata = std::fs::metadata(&readme_path)?;
                    let claude_metadata = std::fs::metadata(&claude_md_path)?;
                    
                    if claude_metadata.modified()? > readme_metadata.modified()? {
                        println!("🔄 Updating README.md for {}", project);
                        std::fs::write(&readme_path, &claude_content)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// プロジェクトメタデータの更新
    async fn update_project_metadata(&self) -> Result<()> {
        let projects = self.discover_projects()?;
        
        for project in projects {
            let ai_json_path = self.ai_root.join(&project).join("ai.json");
            
            if ai_json_path.exists() {
                let mut content = std::fs::read_to_string(&ai_json_path)?;
                let mut json_data: serde_json::Value = serde_json::from_str(&content)?;
                
                // last_updated フィールドを更新
                if let Some(project_data) = json_data.get_mut(&project) {
                    if let Some(obj) = project_data.as_object_mut() {
                        obj.insert("last_updated".to_string(), 
                                 serde_json::Value::String(Utc::now().to_rfc3339()));
                        obj.insert("status".to_string(), 
                                 serde_json::Value::String("active".to_string()));
                        
                        content = serde_json::to_string_pretty(&json_data)?;
                        std::fs::write(&ai_json_path, content)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// メインai.jsonからプロジェクトのブランチ情報を取得
    fn get_project_branch(&self, project: &str) -> String {
        let main_ai_json_path = self.ai_root.join("ai.json");
        
        if main_ai_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&main_ai_json_path) {
                if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ai_section) = json_data.get("ai") {
                        if let Some(project_data) = ai_section.get(project) {
                            if let Some(branch) = project_data.get("branch").and_then(|v| v.as_str()) {
                                return branch.to_string();
                            }
                        }
                    }
                }
            }
        }
        
        // デフォルトはmain
        "main".to_string()
    }
}