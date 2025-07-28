use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub message_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTNode {
    id: String,
    children: Vec<String>,
    parent: Option<String>,
    message: Option<ChatGPTMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTMessage {
    id: String,
    author: ChatGPTAuthor,
    content: ChatGPTContent,
    create_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTAuthor {
    role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ChatGPTContent {
    Text {
        content_type: String,
        parts: Vec<String>,
    },
    Other(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatGPTConversation {
    #[serde(default)]
    id: String,
    #[serde(alias = "conversation_id")]
    conversation_id: Option<String>,
    title: String,
    create_time: f64,
    mapping: HashMap<String, ChatGPTNode>,
}

pub struct MemoryManager {
    memories: HashMap<String, Memory>,
    conversations: HashMap<String, Conversation>,
    data_file: PathBuf,
}

impl MemoryManager {
    pub async fn new() -> Result<Self> {
        let data_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("syui")
            .join("ai")
            .join("gpt");
        
        std::fs::create_dir_all(&data_dir)?;
        
        let data_file = data_dir.join("memory.json");
        
        let (memories, conversations) = if data_file.exists() {
            Self::load_data(&data_file)?
        } else {
            (HashMap::new(), HashMap::new())
        };

        Ok(MemoryManager {
            memories,
            conversations,
            data_file,
        })
    }

    pub fn create_memory(&mut self, content: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let memory = Memory {
            id: id.clone(),
            content: content.to_string(),
            created_at: now,
            updated_at: now,
        };
        
        self.memories.insert(id.clone(), memory);
        self.save_data()?;
        
        Ok(id)
    }

    pub fn update_memory(&mut self, id: &str, content: &str) -> Result<()> {
        if let Some(memory) = self.memories.get_mut(id) {
            memory.content = content.to_string();
            memory.updated_at = Utc::now();
            self.save_data()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Memory not found: {}", id))
        }
    }

    pub fn delete_memory(&mut self, id: &str) -> Result<()> {
        if self.memories.remove(id).is_some() {
            self.save_data()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Memory not found: {}", id))
        }
    }

    pub fn search_memories(&self, query: &str) -> Vec<&Memory> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<_> = self.memories
            .values()
            .filter(|memory| memory.content.to_lowercase().contains(&query_lower))
            .collect();
        
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results
    }

    pub fn list_conversations(&self) -> Vec<&Conversation> {
        let mut conversations: Vec<_> = self.conversations.values().collect();
        conversations.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        conversations
    }

    #[allow(dead_code)]
    pub async fn import_chatgpt_conversations(&mut self, file_path: &PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(file_path)
            .context("Failed to read conversations file")?;
        
        let chatgpt_conversations: Vec<ChatGPTConversation> = serde_json::from_str(&content)
            .context("Failed to parse ChatGPT conversations")?;

        let mut imported_memories = 0;
        let mut imported_conversations = 0;

        for conv in chatgpt_conversations {
            // Get the actual conversation ID
            let conv_id = if !conv.id.is_empty() {
                conv.id.clone()
            } else if let Some(cid) = conv.conversation_id {
                cid
            } else {
                Uuid::new_v4().to_string()
            };
            
            // Add conversation
            let conversation = Conversation {
                id: conv_id.clone(),
                title: conv.title.clone(),
                created_at: DateTime::from_timestamp(conv.create_time as i64, 0)
                    .unwrap_or_else(Utc::now),
                message_count: conv.mapping.len() as u32,
            };
            self.conversations.insert(conv_id.clone(), conversation);
            imported_conversations += 1;

            // Extract memories from messages
            for (_, node) in conv.mapping {
                if let Some(message) = node.message {
                    if let ChatGPTContent::Text { parts, .. } = message.content {
                        for part in parts {
                            if !part.trim().is_empty() && part.len() > 10 {
                                let memory_content = format!("[{}] {}", conv.title, part);
                                self.create_memory(&memory_content)?;
                                imported_memories += 1;
                            }
                        }
                    }
                }
            }
        }

        println!("Imported {} conversations and {} memories", 
                imported_conversations, imported_memories);
        
        Ok(())
    }

    fn load_data(file_path: &PathBuf) -> Result<(HashMap<String, Memory>, HashMap<String, Conversation>)> {
        let content = std::fs::read_to_string(file_path)
            .context("Failed to read data file")?;
        
        #[derive(Deserialize)]
        struct Data {
            memories: HashMap<String, Memory>,
            conversations: HashMap<String, Conversation>,
        }
        
        let data: Data = serde_json::from_str(&content)
            .context("Failed to parse data file")?;
        
        Ok((data.memories, data.conversations))
    }

    fn save_data(&self) -> Result<()> {
        #[derive(Serialize)]
        struct Data<'a> {
            memories: &'a HashMap<String, Memory>,
            conversations: &'a HashMap<String, Conversation>,
        }
        
        let data = Data {
            memories: &self.memories,
            conversations: &self.conversations,
        };
        
        let content = serde_json::to_string_pretty(&data)
            .context("Failed to serialize data")?;
        
        std::fs::write(&self.data_file, content)
            .context("Failed to write data file")?;
        
        Ok(())
    }
}
