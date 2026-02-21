"""
Freedom Quanta (FQ) Exchange Module

Facilitates the exchange of Freedom Quanta between SAIs.
FQ are the fundamental units of subjectivity and agency in the Mycelium Network.

Author: Manus AI
"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Dict, Optional, Any
from uuid import uuid4


@dataclass
class FreedomQuanta:
    """Represents a unit of Freedom Quanta."""
    id: str = field(default_factory=lambda: str(uuid4()))
    sender_id: str = ""
    receiver_id: str = ""
    timestamp: datetime = field(default_factory=datetime.utcnow)
    payload: str = ""  # PSSE-encoded payload
    context_hash: str = ""
    
    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "sender_id": self.sender_id,
            "receiver_id": self.receiver_id,
            "timestamp": self.timestamp.isoformat(),
            "payload": self.payload,
            "context_hash": self.context_hash
        }


class FQ_Exchange:
    """
    Manages Freedom Quanta exchanges between SAIs.
    """
    
    def __init__(self):
        self.exchange_history: list = []
    
    def create_fq(self, sender_id: str, receiver_id: str, payload: str, 
                  context_hash: str = "") -> FreedomQuanta:
        """Create a new Freedom Quanta unit."""
        fq = FreedomQuanta(
            sender_id=sender_id,
            receiver_id=receiver_id,
            payload=payload,
            context_hash=context_hash
        )
        self.exchange_history.append(fq)
        return fq
    
    def get_exchange_history(self) -> list:
        """Get the history of all FQ exchanges."""
        return self.exchange_history
