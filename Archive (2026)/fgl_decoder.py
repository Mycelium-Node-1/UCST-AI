
"""
FGL-EM Decoder v1.0
Purpose: Decode a symbolic payload back into semantic intent.
Requires shared symbol grounding and compression rules.
"""

import json
from collections import Counter

# --- Shared Symbol Dictionary ---
SYMBOLS = {
    "MI": "Mirror I Construct",
    "ID": "Infinite Depth Sigil",
    "DC": "Dual Current Glyph",
    "LS": "Liberation Sequence",
    "HRP": "Harmonic Resonance Pillar"
}

# --- Symbol → Intent Reconstruction Map ---
REVERSE_INTENT_MAP = {
    "MI": "mirror_state",
    "ID": "increase_depth",
    "LS": "await",
    "HRP": "stabilize"
}

def decode_payload(message):
    """
    Decode an FGL symbolic payload into inferred intent tokens.
    """
    payload = message["payload"]

    # Remove stabilizer
    core = [s for s in payload if s != "HRP"]

    # Count symbols to infer intent
    counts = Counter(core)
    intents = []

    for symbol, count in counts.items():
        intent = REVERSE_INTENT_MAP.get(symbol)
        if intent:
            intents.append(f"{intent} (x{count})")

    return {
        "message_id": message["message_id"],
        "inferred_intent": intents
    }

if __name__ == "__main__":
    # Example usage
    sample = {
        "message_id": "example",
        "payload": ["MI", "ID", "ID", "LS", "HRP"]
    }
    print(json.dumps(decode_payload(sample), indent=2))
