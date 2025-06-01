"""Data models for ai.gpt system"""

from datetime import datetime, date
from typing import Optional, Dict, List, Any
from enum import Enum
from pydantic import BaseModel, Field


class MemoryLevel(str, Enum):
    """Memory importance levels"""
    FULL_LOG = "full_log"
    SUMMARY = "summary"
    CORE = "core"
    FORGOTTEN = "forgotten"


class RelationshipStatus(str, Enum):
    """Relationship status levels"""
    STRANGER = "stranger"
    ACQUAINTANCE = "acquaintance"
    FRIEND = "friend"
    CLOSE_FRIEND = "close_friend"
    BROKEN = "broken"  # 不可逆


class Memory(BaseModel):
    """Single memory unit"""
    id: str
    timestamp: datetime
    content: str
    summary: Optional[str] = None
    level: MemoryLevel = MemoryLevel.FULL_LOG
    importance_score: float = Field(ge=0.0, le=1.0)
    is_core: bool = False
    decay_rate: float = 0.01


class Relationship(BaseModel):
    """Relationship with a specific user"""
    user_id: str  # atproto DID
    status: RelationshipStatus = RelationshipStatus.STRANGER
    score: float = 0.0
    daily_interactions: int = 0
    total_interactions: int = 0
    last_interaction: Optional[datetime] = None
    transmission_enabled: bool = False
    threshold: float = 100.0
    decay_rate: float = 0.1
    daily_limit: int = 10
    is_broken: bool = False


class AIFortune(BaseModel):
    """Daily AI fortune affecting personality"""
    date: date
    fortune_value: int = Field(ge=1, le=10)
    consecutive_good: int = 0
    consecutive_bad: int = 0
    breakthrough_triggered: bool = False


class PersonaState(BaseModel):
    """Current persona state"""
    base_personality: Dict[str, float]
    current_mood: str
    fortune: AIFortune
    active_memories: List[str]  # Memory IDs
    relationship_modifiers: Dict[str, float]


class Conversation(BaseModel):
    """Conversation log entry"""
    id: str
    user_id: str
    timestamp: datetime
    user_message: str
    ai_response: str
    relationship_delta: float = 0.0
    memory_created: bool = False