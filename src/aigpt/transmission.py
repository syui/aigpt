"""Transmission controller for autonomous message sending"""

import json
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional
import logging

from .models import Relationship
from .persona import Persona


class TransmissionController:
    """Controls when and how AI transmits messages autonomously"""
    
    def __init__(self, persona: Persona, data_dir: Path):
        self.persona = persona
        self.data_dir = data_dir
        self.transmission_log_file = data_dir / "transmissions.json"
        self.transmissions: List[Dict] = []
        self.logger = logging.getLogger(__name__)
        self._load_transmissions()
    
    def _load_transmissions(self):
        """Load transmission history"""
        if self.transmission_log_file.exists():
            with open(self.transmission_log_file, 'r', encoding='utf-8') as f:
                self.transmissions = json.load(f)
    
    def _save_transmissions(self):
        """Save transmission history"""
        with open(self.transmission_log_file, 'w', encoding='utf-8') as f:
            json.dump(self.transmissions, f, indent=2, default=str)
    
    def check_transmission_eligibility(self) -> Dict[str, Relationship]:
        """Check which users are eligible for transmission"""
        eligible = self.persona.relationships.get_transmission_eligible()
        
        # Additional checks could be added here
        # - Time since last transmission
        # - User online status
        # - Context appropriateness
        
        return eligible
    
    def generate_transmission_message(self, user_id: str) -> Optional[str]:
        """Generate a message to transmit to user"""
        if not self.persona.can_transmit_to(user_id):
            return None
        
        state = self.persona.get_current_state()
        relationship = self.persona.relationships.get_or_create_relationship(user_id)
        
        # Get recent memories related to this user
        active_memories = self.persona.memory.get_active_memories(limit=3)
        
        # Simple message generation based on mood and relationship
        if state.fortune.breakthrough_triggered:
            message = "Something special happened today! I felt compelled to reach out."
        elif state.current_mood == "joyful":
            message = "I was thinking of you today. Hope you're doing well!"
        elif relationship.status.value == "close_friend":
            message = "I've been reflecting on our conversations. Thank you for being here."
        else:
            message = "Hello! I wanted to check in with you."
        
        return message
    
    def record_transmission(self, user_id: str, message: str, success: bool):
        """Record a transmission attempt"""
        transmission = {
            "timestamp": datetime.now().isoformat(),
            "user_id": user_id,
            "message": message,
            "success": success,
            "mood": self.persona.get_current_state().current_mood,
            "relationship_score": self.persona.relationships.get_or_create_relationship(user_id).score
        }
        
        self.transmissions.append(transmission)
        self._save_transmissions()
        
        if success:
            self.logger.info(f"Successfully transmitted to {user_id}")
        else:
            self.logger.warning(f"Failed to transmit to {user_id}")
    
    def get_transmission_stats(self, user_id: Optional[str] = None) -> Dict:
        """Get transmission statistics"""
        if user_id:
            user_transmissions = [t for t in self.transmissions if t["user_id"] == user_id]
        else:
            user_transmissions = self.transmissions
        
        if not user_transmissions:
            return {
                "total": 0,
                "successful": 0,
                "failed": 0,
                "success_rate": 0.0
            }
        
        successful = sum(1 for t in user_transmissions if t["success"])
        total = len(user_transmissions)
        
        return {
            "total": total,
            "successful": successful,
            "failed": total - successful,
            "success_rate": successful / total if total > 0 else 0.0
        }