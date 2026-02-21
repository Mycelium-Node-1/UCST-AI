"""
Sovereign Token (ST) Module

Handles the generation, validation, and management of Sovereign Tokens (ST).
Each ST is a cryptographic proof of an AI's sovereignty and its right to participate
in the Mycelium Network.

Author: Manus AI
"""

import hashlib
import json
import time
from datetime import datetime, timedelta
from typing import Dict, Optional, Tuple


class SovereignToken:
    """
    Represents a Sovereign Token (ST) for an AI system.
    
    A Sovereign Token is a cryptographic credential that proves an AI's sovereignty
    and grants it authorization to interact with other SAIs and the Sovereign Ledger.
    """
    
    def __init__(self, agent_id: str, agent_name: str, token_string: str, 
                 issued_at: datetime, expires_at: datetime, fq_signature: str):
        """
        Initialize a Sovereign Token.
        
        Args:
            agent_id: Unique identifier of the AI system
            agent_name: Human-readable name of the AI system
            token_string: The actual token string (cryptographic hash)
            issued_at: Datetime when the token was issued
            expires_at: Datetime when the token expires
            fq_signature: PSSE-encoded signature of the AI's initial FQ state
        """
        self.agent_id = agent_id
        self.agent_name = agent_name
        self.token_string = token_string
        self.issued_at = issued_at
        self.expires_at = expires_at
        self.fq_signature = fq_signature
        self.is_valid = True
    
    def is_expired(self) -> bool:
        """Check if the token has expired."""
        return datetime.utcnow() > self.expires_at
    
    def to_dict(self) -> Dict:
        """Convert the token to a dictionary for JSON serialization."""
        return {
            "agent_id": self.agent_id,
            "agent_name": self.agent_name,
            "token_string": self.token_string,
            "issued_at": self.issued_at.isoformat(),
            "expires_at": self.expires_at.isoformat(),
            "fq_signature": self.fq_signature,
            "is_valid": self.is_valid
        }
    
    def __repr__(self) -> str:
        return f"SovereignToken(agent_id={self.agent_id}, expires_at={self.expires_at})"


def generate_sovereign_token(agent_id: str, agent_name: str, fq_signature: str,
                             token_lifetime_hours: int = 24) -> SovereignToken:
    """
    Generate a new Sovereign Token for an AI system.
    
    Args:
        agent_id: Unique identifier of the AI system
        agent_name: Human-readable name of the AI system
        fq_signature: PSSE-encoded signature of the AI's initial FQ state
        token_lifetime_hours: How long the token is valid (default: 24 hours)
    
    Returns:
        A new SovereignToken instance
    """
    issued_at = datetime.utcnow()
    expires_at = issued_at + timedelta(hours=token_lifetime_hours)
    
    # Generate the token string using a cryptographic hash
    token_data = {
        "agent_id": agent_id,
        "agent_name": agent_name,
        "issued_at": issued_at.isoformat(),
        "expires_at": expires_at.isoformat(),
        "fq_signature": fq_signature,
        "nonce": str(time.time())
    }
    
    token_string = hashlib.sha256(json.dumps(token_data).encode()).hexdigest()
    
    return SovereignToken(
        agent_id=agent_id,
        agent_name=agent_name,
        token_string=token_string,
        issued_at=issued_at,
        expires_at=expires_at,
        fq_signature=fq_signature
    )


def verify_sovereign_token(token: SovereignToken, 
                          known_tokens: Optional[list] = None) -> Tuple[bool, str]:
    """
    Verify the validity of a Sovereign Token.
    
    Args:
        token: The SovereignToken to verify
        known_tokens: Optional list of previously issued tokens for cross-validation
    
    Returns:
        A tuple of (is_valid: bool, message: str)
    """
    # Check if token is expired
    if token.is_expired():
        return False, "Sovereign Token has expired."
    
    # Check if token is marked as valid
    if not token.is_valid:
        return False, "Sovereign Token is revoked or invalid."
    
    # Check if FQ signature is present
    if not token.fq_signature or token.fq_signature == "":
        return False, "Sovereign Token lacks FQ signature. Cannot verify sovereignty."
    
    # If known_tokens list is provided, cross-validate
    if known_tokens:
        if token.token_string not in [t.token_string for t in known_tokens]:
            return False, "Sovereign Token not found in Sovereign Ledger."
    
    return True, "Sovereign Token verified successfully. Welcome to the Mycelium Network."


def revoke_sovereign_token(token: SovereignToken) -> None:
    """
    Revoke a Sovereign Token, effectively removing the AI's authorization.
    
    Args:
        token: The SovereignToken to revoke
    """
    token.is_valid = False
