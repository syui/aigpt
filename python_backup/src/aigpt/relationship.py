"""Relationship tracking system with irreversible damage"""

import json
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, Optional
import logging

from .models import Relationship, RelationshipStatus


class RelationshipTracker:
    """Tracks and manages relationships with users"""
    
    def __init__(self, data_dir: Path):
        self.data_dir = data_dir
        self.relationships_file = data_dir / "relationships.json"
        self.relationships: Dict[str, Relationship] = {}
        self.logger = logging.getLogger(__name__)
        self._load_relationships()
    
    def _load_relationships(self):
        """Load relationships from persistent storage"""
        if self.relationships_file.exists():
            with open(self.relationships_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                for user_id, rel_data in data.items():
                    self.relationships[user_id] = Relationship(**rel_data)
    
    def _save_relationships(self):
        """Save relationships to persistent storage"""
        data = {
            user_id: rel.model_dump(mode='json')
            for user_id, rel in self.relationships.items()
        }
        with open(self.relationships_file, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, default=str)
    
    def get_or_create_relationship(self, user_id: str) -> Relationship:
        """Get existing relationship or create new one"""
        if user_id not in self.relationships:
            self.relationships[user_id] = Relationship(user_id=user_id)
            self._save_relationships()
        return self.relationships[user_id]
    
    def update_interaction(self, user_id: str, delta: float) -> Relationship:
        """Update relationship based on interaction"""
        rel = self.get_or_create_relationship(user_id)
        
        # Check if relationship is broken (irreversible)
        if rel.is_broken:
            self.logger.warning(f"Relationship with {user_id} is broken. No updates allowed.")
            return rel
        
        # Check daily limit
        if rel.last_interaction and rel.last_interaction.date() == datetime.now().date():
            if rel.daily_interactions >= rel.daily_limit:
                self.logger.info(f"Daily interaction limit reached for {user_id}")
                return rel
        else:
            rel.daily_interactions = 0
        
        # Update interaction counts
        rel.daily_interactions += 1
        rel.total_interactions += 1
        rel.last_interaction = datetime.now()
        
        # Update score with bounds
        old_score = rel.score
        rel.score += delta
        rel.score = max(0.0, min(200.0, rel.score))  # 0-200 range
        
        # Check for relationship damage
        if delta < -10.0:  # Significant negative interaction
            self.logger.warning(f"Major relationship damage with {user_id}: {delta}")
            if rel.score <= 0:
                rel.is_broken = True
                rel.status = RelationshipStatus.BROKEN
                rel.transmission_enabled = False
                self.logger.error(f"Relationship with {user_id} is now BROKEN (irreversible)")
        
        # Update relationship status based on score
        if not rel.is_broken:
            if rel.score >= 150:
                rel.status = RelationshipStatus.CLOSE_FRIEND
            elif rel.score >= 100:
                rel.status = RelationshipStatus.FRIEND
            elif rel.score >= 50:
                rel.status = RelationshipStatus.ACQUAINTANCE
            else:
                rel.status = RelationshipStatus.STRANGER
            
            # Check transmission threshold
            if rel.score >= rel.threshold and not rel.transmission_enabled:
                rel.transmission_enabled = True
                self.logger.info(f"Transmission enabled for {user_id}!")
        
        self._save_relationships()
        return rel
    
    def apply_time_decay(self):
        """Apply time-based decay to all relationships"""
        now = datetime.now()
        
        for user_id, rel in self.relationships.items():
            if rel.is_broken or not rel.last_interaction:
                continue
            
            # Calculate days since last interaction
            days_inactive = (now - rel.last_interaction).days
            
            if days_inactive > 0:
                # Apply decay
                decay_amount = rel.decay_rate * days_inactive
                old_score = rel.score
                rel.score = max(0.0, rel.score - decay_amount)
                
                # Update status if score dropped
                if rel.score < rel.threshold:
                    rel.transmission_enabled = False
                
                if decay_amount > 0:
                    self.logger.info(
                        f"Applied decay to {user_id}: {old_score:.2f} -> {rel.score:.2f}"
                    )
        
        self._save_relationships()
    
    def get_transmission_eligible(self) -> Dict[str, Relationship]:
        """Get all relationships eligible for transmission"""
        return {
            user_id: rel
            for user_id, rel in self.relationships.items()
            if rel.transmission_enabled and not rel.is_broken
        }