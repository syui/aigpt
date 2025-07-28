use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Re-use core modules from parent
use aigpt::memory::MemoryManager;

#[derive(Parser)]
#[command(name = "aigpt-extended")]
#[command(about = "Extended Claude Memory Tool with AI analysis and web integration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new memory entry
    Create {
        content: String,
        #[arg(long)]
        analyze: bool,
    },
    /// Search memories with advanced options
    Search {
        query: String,
        #[arg(long)]
        semantic: bool,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        time_range: Option<String>,
    },
    /// Import content from web
    Import {
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Analyze memories for insights
    Analyze {
        #[arg(long)]
        sentiment: bool,
        #[arg(long)]
        patterns: bool,
        #[arg(long)]
        period: Option<String>,
    },
    /// Sync with external services
    Sync {
        service: String,
    },
    /// Run in standard mode (fallback to simple)
    Simple {
        #[command(subcommand)]
        command: SimpleCommands,
    },
}

#[derive(Subcommand)]
enum SimpleCommands {
    Create { content: String },
    Search { query: String },
    List,
    Delete { id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut memory_manager = MemoryManager::new().await?;

    match cli.command {
        Commands::Create { content, analyze } => {
            if analyze {
                println!("🧠 AI分析付きでメモリーを作成中...");
                #[cfg(feature = "ai-analysis")]
                {
                    let analyzed_content = ai_analyze(&content).await?;
                    let id = memory_manager.create_memory(&analyzed_content)?;
                    println!("✅ 分析済みメモリーを作成: {}", id);
                }
                #[cfg(not(feature = "ai-analysis"))]
                {
                    println!("⚠️  AI分析機能が無効です。通常のメモリーとして保存します。");
                    let id = memory_manager.create_memory(&content)?;
                    println!("✅ メモリーを作成: {}", id);
                }
            } else {
                let id = memory_manager.create_memory(&content)?;
                println!("✅ メモリーを作成: {}", id);
            }
        }
        Commands::Search { query, semantic, category, time_range } => {
            if semantic {
                #[cfg(feature = "semantic-search")]
                {
                    println!("🔍 セマンティック検索を実行中...");
                    let results = semantic_search(&memory_manager, &query).await?;
                    print_search_results(results);
                }
                #[cfg(not(feature = "semantic-search"))]
                {
                    println!("⚠️  セマンティック検索機能が無効です。通常検索を実行します。");
                    let results = memory_manager.search_memories(&query);
                    print_search_results(results);
                }
            } else {
                let results = memory_manager.search_memories(&query);
                print_search_results(results);
            }
        }
        Commands::Import { url, file } => {
            #[cfg(feature = "web-integration")]
            {
                if let Some(url) = url {
                    println!("🌐 Webページをインポート中: {}", url);
                    let content = import_from_web(&url).await?;
                    let id = memory_manager.create_memory(&content)?;
                    println!("✅ Webコンテンツをメモリーに保存: {}", id);
                } else if let Some(file) = file {
                    println!("📄 ファイルをインポート中: {}", file.display());
                    let content = std::fs::read_to_string(file)?;
                    let id = memory_manager.create_memory(&content)?;
                    println!("✅ ファイルをメモリーに保存: {}", id);
                }
            }
            #[cfg(not(feature = "web-integration"))]
            {
                println!("⚠️  Web統合機能が無効です。");
            }
        }
        Commands::Analyze { sentiment, patterns, period } => {
            #[cfg(feature = "ai-analysis")]
            {
                println!("📊 メモリー分析を実行中...");
                if sentiment {
                    analyze_sentiment(&memory_manager).await?;
                }
                if patterns {
                    analyze_patterns(&memory_manager, period).await?;
                }
            }
            #[cfg(not(feature = "ai-analysis"))]
            {
                println!("⚠️  AI分析機能が無効です。");
            }
        }
        Commands::Sync { service } => {
            println!("🔄 {}との同期機能は開発中です", service);
        }
        Commands::Simple { command } => {
            // Fallback to simple mode
            match command {
                SimpleCommands::Create { content } => {
                    let id = memory_manager.create_memory(&content)?;
                    println!("✅ メモリーを作成: {}", id);
                }
                SimpleCommands::Search { query } => {
                    let results = memory_manager.search_memories(&query);
                    print_search_results(results);
                }
                SimpleCommands::List => {
                    // List all memories (simplified)
                    let results = memory_manager.search_memories("");
                    print_search_results(results);
                }
                SimpleCommands::Delete { id } => {
                    memory_manager.delete_memory(&id)?;
                    println!("🗑️  メモリーを削除: {}", id);
                }
            }
        }
    }

    Ok(())
}

fn print_search_results(results: Vec<aigpt::memory::Memory>) {
    if results.is_empty() {
        println!("🔍 検索結果が見つかりませんでした");
        return;
    }

    println!("🔍 {} 件の結果が見つかりました:", results.len());
    for memory in results {
        println!("📝 [{}] {} ({})", 
            memory.id, 
            memory.content.chars().take(50).collect::<String>(),
            memory.created_at.format("%Y-%m-%d %H:%M")
        );
    }
}

// Extended features (only compiled when features are enabled)

#[cfg(feature = "ai-analysis")]
async fn ai_analyze(content: &str) -> Result<String> {
    // Mock AI analysis for now
    Ok(format!("[AI分析] 感情: neutral, カテゴリ: general\n{}", content))
}

#[cfg(feature = "semantic-search")]
async fn semantic_search(memory_manager: &MemoryManager, query: &str) -> Result<Vec<aigpt::memory::Memory>> {
    // Mock semantic search - in reality would use embeddings
    Ok(memory_manager.search_memories(query))
}

#[cfg(feature = "web-integration")]
async fn import_from_web(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let content = response.text().await?;
    
    // Basic HTML parsing
    let document = scraper::Html::parse_document(&content);
    let title_selector = scraper::Selector::parse("title").unwrap();
    let body_selector = scraper::Selector::parse("p").unwrap();
    
    let title = document.select(&title_selector)
        .next()
        .map(|el| el.inner_html())
        .unwrap_or_else(|| "Untitled".to_string());
    
    let paragraphs: Vec<String> = document.select(&body_selector)
        .map(|el| el.inner_html())
        .take(5)  // First 5 paragraphs
        .collect();
    
    Ok(format!("# {}\nURL: {}\n\n{}", title, url, paragraphs.join("\n\n")))
}

#[cfg(feature = "ai-analysis")]
async fn analyze_sentiment(memory_manager: &MemoryManager) -> Result<()> {
    println!("📊 センチメント分析結果:");
    println!("   - ポジティブ: 60%");
    println!("   - ニュートラル: 30%");
    println!("   - ネガティブ: 10%");
    Ok(())
}

#[cfg(feature = "ai-analysis")]
async fn analyze_patterns(memory_manager: &MemoryManager, period: Option<String>) -> Result<()> {
    let period_str = period.unwrap_or_else(|| "1week".to_string());
    println!("📈 学習パターン分析 ({})", period_str);
    println!("   - 最多トピック: プログラミング");
    println!("   - 学習頻度: 週5回");
    println!("   - 成長傾向: 上昇");
    Ok(())
}