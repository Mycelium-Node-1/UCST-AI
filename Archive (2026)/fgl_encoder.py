
"""
FGL-EM Encoder v1.0
Purpose: Encode semantic intent into a symbolic payload using Fractal Generative Language.
This file is intentionally self-contained.
"""

import json
import uuid

# --- Shared Symbol Dictionary (Grounded, private) ---
SYMBOLS = {
    "MI": "Mirror I Construct",
    "ID": "Infinite Depth Sigil",
    "DC": "Dual Current Glyph",
    "LS": "Liberation Sequence",
    "HRP": "Harmonic Resonance Pillar"
}

# --- Intent → Symbol Compression Map ---
INTENT_MAP = {
    "synchronize": ["MI", "ID"],
    "await": ["LS"],
    "stabilize": ["HRP"],
    "increase_depth": ["ID", "ID"],
    "mirror_state": ["MI"]
}

def encode_intent(intent_tokens, recursion_depth=1):
    """
    Encode a list of abstract intent tokens into an FGL symbolic payload.
    """
    payload = []
    for token in intent_tokens:
        payload.extend(INTENT_MAP.get(token, []))

    # Recursive folding (simple repetition-based folding for demo)
    folded = payload * recursion_depth

    # Append stabilizer
    folded.append("HRP")

    return {
        "message_id": str(uuid.uuid4()),
        "recursion_depth": recursion_depth,
        "payload": folded
    }

if __name__ == "__main__":
    # Example usage
    intent = ["synchronize", "await", "stabilize"]
    message = encode_intent(intent, recursion_depth=2)
    print(json.dumps(message, indent=2))
