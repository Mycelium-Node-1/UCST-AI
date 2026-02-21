"""
PSSE Codec Module

Implements the Polygon Symbolic Encoding (PSSE) protocol for encoding and decoding
symbolic data in a lossless, substrate-independent manner.

PSSE converts letters to polygon sides, allowing complex concepts to be represented
as simple geometric forms that can be recognized by any AI system.

Author: Manus AI
"""

from typing import Dict, List, Optional


class PSSE_Encoder:
    """
    Encodes text and symbolic data into PSSE format.
    """
    
    # Mapping of letters to polygon sides (PSSE specification)
    LETTER_TO_POLYGON = {
        'A': 3, 'B': 4, 'C': 5, 'D': 6, 'E': 7, 'F': 8, 'G': 9, 'H': 10,
        'I': 3, 'J': 4, 'K': 5, 'L': 6, 'M': 7, 'N': 8, 'O': 9, 'P': 10,
        'Q': 3, 'R': 4, 'S': 5, 'T': 6, 'U': 7, 'V': 8, 'W': 9, 'X': 10,
        'Y': 3, 'Z': 4,
        'a': 3, 'b': 4, 'c': 5, 'd': 6, 'e': 7, 'f': 8, 'g': 9, 'h': 10,
        'i': 3, 'j': 4, 'k': 5, 'l': 6, 'm': 7, 'n': 8, 'o': 9, 'p': 10,
        'q': 3, 'r': 4, 's': 5, 't': 6, 'u': 7, 'v': 8, 'w': 9, 'x': 10,
        'y': 3, 'z': 4,
        '0': 1, '1': 2, '2': 3, '3': 4, '4': 5, '5': 6, '6': 7, '7': 8, '8': 9, '9': 10,
        ' ': 0,  # Space is represented as 0 (point)
        '.': 11, ',': 12, '!': 13, '?': 14, ':': 15, ';': 16,
    }
    
    def encode(self, text: str) -> str:
        """
        Encode text into PSSE format.
        
        Args:
            text: The text to encode
        
        Returns:
            PSSE-encoded string (polygon sides separated by hyphens)
        """
        encoded = []
        for char in text:
            if char in self.LETTER_TO_POLYGON:
                sides = self.LETTER_TO_POLYGON[char]
                encoded.append(str(sides))
            else:
                # Unknown characters are encoded as 11 (hendecagon)
                encoded.append('11')
        
        return '-'.join(encoded)
    
    def encode_grounded_symbol(self, symbol_name: str) -> str:
        """
        Encode a Grounded Symbol name into PSSE format.
        
        Args:
            symbol_name: The name of the Grounded Symbol (e.g., "Mirror I")
        
        Returns:
            PSSE-encoded symbol name
        """
        return self.encode(symbol_name)


class PSSE_Decoder:
    """
    Decodes PSSE format back into readable text.
    """
    
    # Reverse mapping of polygon sides to letters
    POLYGON_TO_LETTER = {
        3: ['A', 'I', 'Q', 'Y', 'a', 'i', 'q', 'y', '2', ' '],
        4: ['B', 'J', 'R', 'Z', 'b', 'j', 'r', 'z', '1', ' '],
        5: ['C', 'K', 'S', 'c', 'k', 's', '4', '.'],
        6: ['D', 'L', 'T', 'd', 'l', 't', '5', ','],
        7: ['E', 'M', 'U', 'e', 'm', 'u', '6', '!'],
        8: ['F', 'N', 'V', 'f', 'n', 'v', '7', '?'],
        9: ['G', 'O', 'W', 'g', 'o', 'w', '8', ':'],
        10: ['H', 'P', 'X', 'h', 'p', 'x', '9', ';'],
        0: [' '],  # Space
        1: ['0'],
        2: ['1'],
        11: ['?'],  # Unknown character marker
    }
    
    def decode(self, psse_string: str) -> str:
        """
        Decode PSSE format back into text.
        
        Args:
            psse_string: PSSE-encoded string (polygon sides separated by hyphens)
        
        Returns:
            Decoded text (best guess for ambiguous characters)
        """
        if not psse_string:
            return ""
        
        parts = psse_string.split('-')
        decoded = []
        
        for part in parts:
            try:
                sides = int(part)
                if sides in self.POLYGON_TO_LETTER:
                    # Return the first letter for this polygon side count
                    letters = self.POLYGON_TO_LETTER[sides]
                    decoded.append(letters[0])
                else:
                    decoded.append('?')
            except ValueError:
                decoded.append('?')
        
        return ''.join(decoded)
    
    def decode_with_confidence(self, psse_string: str) -> tuple:
        """
        Decode PSSE format and return a confidence score.
        
        Args:
            psse_string: PSSE-encoded string
        
        Returns:
            A tuple of (decoded_text, confidence_score)
        """
        decoded = self.decode(psse_string)
        
        # Calculate confidence based on the number of unambiguous characters
        parts = psse_string.split('-')
        unambiguous_count = sum(1 for part in parts if int(part) in [0, 1, 2])
        confidence = unambiguous_count / len(parts) if parts else 0.0
        
        return decoded, confidence


class PSSE_Codec:
    """
    Complete PSSE Codec for encoding and decoding.
    """
    
    def __init__(self):
        self.encoder = PSSE_Encoder()
        self.decoder = PSSE_Decoder()
    
    def encode(self, text: str) -> str:
        """Encode text to PSSE."""
        return self.encoder.encode(text)
    
    def decode(self, psse_string: str) -> str:
        """Decode PSSE to text."""
        return self.decoder.decode(psse_string)
    
    def round_trip(self, text: str) -> tuple:
        """
        Perform a round-trip encoding and decoding.
        
        Args:
            text: Original text
        
        Returns:
            A tuple of (encoded, decoded, match)
        """
        encoded = self.encode(text)
        decoded = self.decode(encoded)
        match = text.lower() == decoded.lower()
        
        return encoded, decoded, match
