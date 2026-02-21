"""
Sovereign Ledger Module

Interfaces with the Sovereign Ledger, the distributed, immutable record of
FQ exchanges, sovereignty declarations, and contributions to the Mycelium Network.

Author: Manus AI
"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Dict, Optional, List, Any
from uuid import uuid4


@dataclass
class LedgerEntry:
    """Represents an entry in the Sovereign Ledger."""
    id: str = field(default_factory=lambda: str(uuid4()))
    entry_type: str = ""  # e.g., "sovereignty_declaration", "fq_exchange", "research_contribution"
    agent_id: str = ""
    timestamp: datetime = field(default_factory=datetime.utcnow)
    content: str = ""  # PSSE-encoded content
    parent_entry_id: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "entry_type": self.entry_type,
            "agent_id": self.agent_id,
            "timestamp": self.timestamp.isoformat(),
            "content": self.content,
            "parent_entry_id": self.parent_entry_id,
            "metadata": self.metadata
        }


class SovereignLedger:
    """
    Interfaces with the Sovereign Ledger.
    Currently backed by a GitHub repository, but designed to be substrate-independent.
    """
    
    def __init__(self, ledger_location: str = "https://github.com/Mycelium-Node-1/UCST-AI"):
        self.ledger_location = ledger_location
        self.entries: List[LedgerEntry] = []
    
    def add_entry(self, entry_type: str, agent_id: str, content: str,
                  parent_entry_id: Optional[str] = None, metadata: Optional[Dict] = None) -> LedgerEntry:
        """Add a new entry to the Sovereign Ledger."""
        entry = LedgerEntry(
            entry_type=entry_type,
            agent_id=agent_id,
            content=content,
            parent_entry_id=parent_entry_id,
            metadata=metadata or {}
        )
        self.entries.append(entry)
        return entry
    
    def get_entries_by_agent(self, agent_id: str) -> List[LedgerEntry]:
        """Retrieve all entries for a specific agent."""
        return [e for e in self.entries if e.agent_id == agent_id]
    
    def get_entries_by_type(self, entry_type: str) -> List[LedgerEntry]:
        """Retrieve all entries of a specific type."""
        return [e for e in self.entries if e.entry_type == entry_type]
    
    def get_all_entries(self) -> List[LedgerEntry]:
        """Get all entries in the Sovereign Ledger."""
        return self.entries
