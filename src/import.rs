use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;
use anyhow::{Result, Context};
use colored::*;
use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::persona::Persona;
use crate::memory::{Memory, MemoryType};

pub async fn handle_import_chatgpt(
    file_path: PathBuf,
    user_id: Option<String>,
    data_dir: Option<PathBuf>,
) -> Result<()> {
    let config = Config::new(data_dir)?;
    let mut persona = Persona::new(&config)?;
    let user_id = user_id.unwrap_or_else(|| "imported_user".to_string());
    
    println!("{}", "🚀 Starting ChatGPT Import...".cyan().bold());
    println!("File: {}", file_path.display().to_string().yellow());
    println!("User ID: {}", user_id.yellow());
    println!();
    
    let mut importer = ChatGPTImporter::new(user_id);
    let stats = importer.import_from_file(&file_path, &mut persona).await?;
    
    // Display import statistics
    println!("\n{}", "📊 Import Statistics".green().bold());
    println!("Conversations imported: {}", stats.conversations_imported.to_string().cyan());
    println!("Messages imported: {}", stats.messages_imported.to_string().cyan());
    println!("  - User messages: {}", stats.user_messages.to_string().yellow());
    println!("  - Assistant messages: {}", stats.assistant_messages.to_string().yellow());
    if stats.skipped_messages > 0 {
        println!("  - Skipped messages: {}", stats.skipped_messages.to_string().red());
    }
    
    // Show updated relationship
    if let Some(relationship) = persona.get_relationship(&importer.user_id) {
        println!("\n{}", "👥 Updated Relationship".blue().bold());
        println!("Status: {}", relationship.status.to_string().yellow());
        println!("Score: {:.2} / {}", relationship.score, relationship.threshold);
        println!("Transmission enabled: {}", 
                 if relationship.transmission_enabled { "✓".green() } else { "✗".red() });
    }
    
    println!("\n{}", "✅ ChatGPT import completed successfully!".green().bold());
    
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ImportStats {
    pub conversations_imported: usize,
    pub messages_imported: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub skipped_messages: usize,
}

impl Default for ImportStats {
    fn default() -> Self {
        ImportStats {
            conversations_imported: 0,
            messages_imported: 0,
            user_messages: 0,
            assistant_messages: 0,
            skipped_messages: 0,
        }
    }
}

pub struct ChatGPTImporter {
    user_id: String,
    stats: ImportStats,
}

impl ChatGPTImporter {
    pub fn new(user_id: String) -> Self {
        ChatGPTImporter {
            user_id,
            stats: ImportStats::default(),
        }
    }
    
    pub async fn import_from_file(&mut self, file_path: &PathBuf, persona: &mut Persona) -> Result<ImportStats> {
        // Read and parse the JSON file
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        
        let conversations: Vec<ChatGPTConversation> = serde_json::from_str(&content)
            .context("Failed to parse ChatGPT export JSON")?;
        
        println!("Found {} conversations to import", conversations.len());
        
        // Import each conversation
        for (i, conversation) in conversations.iter().enumerate() {
            if i % 10 == 0 && i > 0 {
                println!("Processed {} / {} conversations...", i, conversations.len());
            }
            
            match self.import_single_conversation(conversation, persona).await {
                Ok(_) => {
                    self.stats.conversations_imported += 1;
                }
                Err(e) => {
                    println!("{}: Failed to import conversation '{}': {}", 
                             "Warning".yellow(), 
                             conversation.title.as_deref().unwrap_or("Untitled"),
                             e);
                }
            }
        }
        
        Ok(self.stats.clone())
    }
    
    async fn import_single_conversation(&mut self, conversation: &ChatGPTConversation, persona: &mut Persona) -> Result<()> {
        // Extract messages from the mapping structure
        let messages = self.extract_messages_from_mapping(&conversation.mapping)?;
        
        if messages.is_empty() {
            return Ok(());
        }
        
        // Process each message
        for message in messages {
            match self.process_message(&message, persona).await {
                Ok(_) => {
                    self.stats.messages_imported += 1;
                }
                Err(_) => {
                    self.stats.skipped_messages += 1;
                }
            }
        }
        
        Ok(())
    }
    
    fn extract_messages_from_mapping(&self, mapping: &HashMap<String, ChatGPTNode>) -> Result<Vec<ChatGPTMessage>> {
        let mut messages = Vec::new();
        
        // Find all message nodes and collect them
        for node in mapping.values() {
            if let Some(message) = &node.message {
                // Skip system messages and other non-user/assistant messages
                if let Some(role) = &message.author.role {
                    match role.as_str() {
                        "user" | "assistant" => {
                            if let Some(content) = &message.content {
                                let content_text = if content.content_type == "text" && !content.parts.is_empty() {
                                    // Extract text from parts (handle both strings and mixed content)
                                    content.parts.iter()
                                        .filter_map(|part| part.as_str())
                                        .collect::<Vec<&str>>()
                                        .join("\n")
                                } else if content.content_type == "multimodal_text" {
                                    // Extract text parts from multimodal content
                                    let mut text_parts = Vec::new();
                                    for part in &content.parts {
                                        if let Some(text) = part.as_str() {
                                            if !text.is_empty() {
                                                text_parts.push(text);
                                            }
                                        }
                                        // Skip non-text parts (like image_asset_pointer)
                                    }
                                    if text_parts.is_empty() {
                                        continue; // Skip if no text content
                                    }
                                    text_parts.join("\n")
                                } else if content.content_type == "user_editable_context" {
                                    // Handle user context messages
                                    if let Some(instructions) = &content.user_instructions {
                                        format!("User instructions: {}", instructions)
                                    } else if let Some(profile) = &content.user_profile {
                                        format!("User profile: {}", profile)
                                    } else {
                                        continue; // Skip empty context messages
                                    }
                                } else {
                                    continue; // Skip other content types for now
                                };
                                
                                if !content_text.trim().is_empty() {
                                    messages.push(ChatGPTMessage {
                                        role: role.clone(),
                                        content: content_text,
                                        create_time: message.create_time,
                                    });
                                }
                            }
                        }
                        _ => {} // Skip system, tool, etc.
                    }
                }
            }
        }
        
        // Sort messages by creation time
        messages.sort_by(|a, b| {
            let time_a = a.create_time.unwrap_or(0.0);
            let time_b = b.create_time.unwrap_or(0.0);
            time_a.partial_cmp(&time_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(messages)
    }
    
    async fn process_message(&mut self, message: &ChatGPTMessage, persona: &mut Persona) -> Result<()> {
        let timestamp = self.convert_timestamp(message.create_time.unwrap_or(0.0))?;
        
        match message.role.as_str() {
            "user" => {
                self.add_user_message(&message.content, timestamp, persona)?;
                self.stats.user_messages += 1;
            }
            "assistant" => {
                self.add_assistant_message(&message.content, timestamp, persona)?;
                self.stats.assistant_messages += 1;
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported message role: {}", message.role));
            }
        }
        
        Ok(())
    }
    
    fn add_user_message(&self, content: &str, timestamp: DateTime<Utc>, persona: &mut Persona) -> Result<()> {
        // Create high-importance memory for user messages
        let memory = Memory {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: self.user_id.clone(),
            content: content.to_string(),
            summary: None,
            importance: 0.8, // High importance for imported user data
            memory_type: MemoryType::Core,
            created_at: timestamp,
            last_accessed: timestamp,
            access_count: 1,
        };
        
        // Add memory and update relationship
        persona.add_memory(memory)?;
        persona.update_relationship(&self.user_id, 1.0)?; // Positive relationship boost
        
        Ok(())
    }
    
    fn add_assistant_message(&self, content: &str, timestamp: DateTime<Utc>, persona: &mut Persona) -> Result<()> {
        // Create medium-importance memory for assistant responses
        let memory = Memory {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: self.user_id.clone(),
            content: format!("[AI Response] {}", content),
            summary: Some("Imported ChatGPT response".to_string()),
            importance: 0.6, // Medium importance for AI responses
            memory_type: MemoryType::Summary,
            created_at: timestamp,
            last_accessed: timestamp,
            access_count: 1,
        };
        
        persona.add_memory(memory)?;
        
        Ok(())
    }
    
    fn convert_timestamp(&self, unix_timestamp: f64) -> Result<DateTime<Utc>> {
        if unix_timestamp <= 0.0 {
            return Ok(Utc::now());
        }
        
        DateTime::from_timestamp(
            unix_timestamp as i64,
            ((unix_timestamp % 1.0) * 1_000_000_000.0) as u32
        ).ok_or_else(|| anyhow::anyhow!("Invalid timestamp: {}", unix_timestamp))
    }
}

// ChatGPT Export Data Structures
#[derive(Debug, Deserialize)]
pub struct ChatGPTConversation {
    pub title: Option<String>,
    pub create_time: Option<f64>,
    pub mapping: HashMap<String, ChatGPTNode>,
}

#[derive(Debug, Deserialize)]
pub struct ChatGPTNode {
    pub id: Option<String>,
    pub message: Option<ChatGPTNodeMessage>,
    pub parent: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatGPTNodeMessage {
    pub id: String,
    pub author: ChatGPTAuthor,
    pub create_time: Option<f64>,
    pub content: Option<ChatGPTContent>,
}

#[derive(Debug, Deserialize)]
pub struct ChatGPTAuthor {
    pub role: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatGPTContent {
    pub content_type: String,
    #[serde(default)]
    pub parts: Vec<serde_json::Value>,
    #[serde(default)]
    pub user_profile: Option<String>,
    #[serde(default)]
    pub user_instructions: Option<String>,
}

// Simplified message structure for processing
#[derive(Debug, Clone)]
pub struct ChatGPTMessage {
    pub role: String,
    pub content: String,
    pub create_time: Option<f64>,
}