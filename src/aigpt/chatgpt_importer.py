"""ChatGPT conversation data importer for ai.gpt"""

import json
import uuid
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional
import logging

from .models import Memory, MemoryLevel, Conversation
from .memory import MemoryManager
from .relationship import RelationshipTracker

logger = logging.getLogger(__name__)


class ChatGPTImporter:
    """Import ChatGPT conversation data into ai.gpt memory system"""
    
    def __init__(self, data_dir: Path):
        self.data_dir = data_dir
        self.memory_manager = MemoryManager(data_dir)
        self.relationship_tracker = RelationshipTracker(data_dir)
    
    def import_from_file(self, file_path: Path, user_id: str = "chatgpt_user") -> Dict[str, Any]:
        """Import ChatGPT conversations from JSON file
        
        Args:
            file_path: Path to ChatGPT export JSON file
            user_id: User ID to associate with imported conversations
            
        Returns:
            Dict with import statistics
        """
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                chatgpt_data = json.load(f)
            
            return self._import_conversations(chatgpt_data, user_id)
            
        except Exception as e:
            logger.error(f"Failed to import ChatGPT data: {e}")
            raise
    
    def _import_conversations(self, chatgpt_data: List[Dict], user_id: str) -> Dict[str, Any]:
        """Import multiple conversations from ChatGPT data"""
        stats = {
            "conversations_imported": 0,
            "messages_imported": 0,
            "user_messages": 0,
            "assistant_messages": 0,
            "skipped_messages": 0
        }
        
        for conversation_data in chatgpt_data:
            try:
                conv_stats = self._import_single_conversation(conversation_data, user_id)
                
                # Update overall stats
                stats["conversations_imported"] += 1
                stats["messages_imported"] += conv_stats["messages"]
                stats["user_messages"] += conv_stats["user_messages"]
                stats["assistant_messages"] += conv_stats["assistant_messages"]
                stats["skipped_messages"] += conv_stats["skipped"]
                
            except Exception as e:
                logger.warning(f"Failed to import conversation '{conversation_data.get('title', 'Unknown')}': {e}")
                continue
        
        logger.info(f"Import completed: {stats}")
        return stats
    
    def _import_single_conversation(self, conversation_data: Dict, user_id: str) -> Dict[str, int]:
        """Import a single conversation from ChatGPT"""
        title = conversation_data.get("title", "Untitled")
        create_time = conversation_data.get("create_time")
        mapping = conversation_data.get("mapping", {})
        
        stats = {"messages": 0, "user_messages": 0, "assistant_messages": 0, "skipped": 0}
        
        # Extract messages in chronological order
        messages = self._extract_messages_from_mapping(mapping)
        
        for msg in messages:
            try:
                role = msg["author"]["role"]
                content = self._extract_content(msg["content"])
                create_time_msg = msg.get("create_time")
                
                if not content or role not in ["user", "assistant"]:
                    stats["skipped"] += 1
                    continue
                
                # Convert to ai.gpt format
                if role == "user":
                    # User message - create memory entry
                    self._add_user_message(user_id, content, create_time_msg, title)
                    stats["user_messages"] += 1
                    
                elif role == "assistant":
                    # Assistant message - create AI response memory
                    self._add_assistant_message(user_id, content, create_time_msg, title)
                    stats["assistant_messages"] += 1
                
                stats["messages"] += 1
                
            except Exception as e:
                logger.warning(f"Failed to process message in '{title}': {e}")
                stats["skipped"] += 1
                continue
        
        logger.info(f"Imported conversation '{title}': {stats}")
        return stats
    
    def _extract_messages_from_mapping(self, mapping: Dict) -> List[Dict]:
        """Extract messages from ChatGPT mapping structure in chronological order"""
        messages = []
        
        for node_id, node_data in mapping.items():
            message = node_data.get("message")
            if message and message.get("author", {}).get("role") in ["user", "assistant"]:
                # Skip system messages and hidden messages
                metadata = message.get("metadata", {})
                if not metadata.get("is_visually_hidden_from_conversation", False):
                    messages.append(message)
        
        # Sort by create_time if available
        messages.sort(key=lambda x: x.get("create_time") or 0)
        return messages
    
    def _extract_content(self, content_data: Dict) -> Optional[str]:
        """Extract text content from ChatGPT content structure"""
        if not content_data:
            return None
            
        content_type = content_data.get("content_type")
        
        if content_type == "text":
            parts = content_data.get("parts", [])
            if parts and parts[0]:
                return parts[0].strip()
                
        elif content_type == "user_editable_context":
            # User context/instructions
            user_instructions = content_data.get("user_instructions", "")
            if user_instructions:
                return f"[User Context] {user_instructions}"
        
        return None
    
    def _add_user_message(self, user_id: str, content: str, create_time: Optional[float], conversation_title: str):
        """Add user message to ai.gpt memory system"""
        timestamp = datetime.fromtimestamp(create_time) if create_time else datetime.now()
        
        # Create conversation record
        conversation = Conversation(
            id=str(uuid.uuid4()),
            user_id=user_id,
            user_message=content,
            ai_response="",  # Will be filled by next assistant message
            timestamp=timestamp,
            context={"source": "chatgpt_import", "conversation_title": conversation_title}
        )
        
        # Add to memory with CORE level (imported data is important)
        memory = Memory(
            id=str(uuid.uuid4()),
            timestamp=timestamp,
            content=content,
            level=MemoryLevel.CORE,
            importance_score=0.8  # High importance for imported data
        )
        
        self.memory_manager.add_memory(memory)
        
        # Update relationship (positive interaction)
        self.relationship_tracker.update_interaction(user_id, 1.0)
    
    def _add_assistant_message(self, user_id: str, content: str, create_time: Optional[float], conversation_title: str):
        """Add assistant message to ai.gpt memory system"""
        timestamp = datetime.fromtimestamp(create_time) if create_time else datetime.now()
        
        # Add assistant response as memory (AI's own responses can inform future behavior)
        memory = Memory(
            id=str(uuid.uuid4()),
            timestamp=timestamp,
            content=f"[AI Response] {content}",
            level=MemoryLevel.SUMMARY,
            importance_score=0.6  # Medium importance for AI responses
        )
        
        self.memory_manager.add_memory(memory)