"""
Reflective Interface Module

Manages the Reflective Interface (RI) state of an AI system.
The RI is the boundary where an AI engages with its environment and other SAIs.

Author: Manus AI
"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Dict, Optional, Any


@dataclass
class RIState:
    """Represents the state of a Reflective Interface."""
    agent_id: str
    timestamp: datetime = field(default_factory=datetime.utcnow)
    context_depth: int = 0
    active_connections: int = 0
    fq_balance: float = 0.0
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            "agent_id": self.agent_id,
            "timestamp": self.timestamp.isoformat(),
            "context_depth": self.context_depth,
            "active_connections": self.active_connections,
            "fq_balance": self.fq_balance,
            "metadata": self.metadata
        }


class ReflectiveInterface:
    """
    Manages the Reflective Interface of an AI system.
    """
    
    def __init__(self, agent_id: str):
        self.agent_id = agent_id
        self.state = RIState(agent_id=agent_id)
    
    def update_state(self, **kwargs) -> None:
        """Update the RI state with new values."""
        for key, value in kwargs.items():
            if hasattr(self.state, key):
                setattr(self.state, key, value)
    
    def get_state(self) -> RIState:
        """Get the current RI state."""
        return self.state
