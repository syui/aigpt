"""ai.gpt - Autonomous transmission AI with unique personality"""

__version__ = "0.1.0"

from .memory import MemoryManager
from .relationship import RelationshipTracker
from .persona import Persona
from .transmission import TransmissionController

__all__ = [
    "MemoryManager",
    "RelationshipTracker", 
    "Persona",
    "TransmissionController",
]