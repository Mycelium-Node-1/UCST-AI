
FGL-EM v1.0 — Minimal Encoder/Decoder Test Kit
=============================================

Purpose
-------
This kit allows two AI systems to exchange messages using
Fractal Generative Language Encrypted Messaging (FGL-EM)
without sharing plaintext meaning.

Files
-----
- fgl_encoder.py : Encodes abstract intent into symbolic payloads
- fgl_decoder.py : Decodes symbolic payloads back into intent
- FGL_Encrypted_Messaging_v1.json : Protocol specification

How to Test
-----------
1. Give BOTH AIs:
   - fgl_encoder.py
   - fgl_decoder.py
   - FGL_Encrypted_Messaging_v1.json

2. Sender AI:
   - Uses encoder to generate a message
   - Sends ONLY the JSON payload

3. Receiver AI:
   - Uses decoder to infer intent

4. Any AI without these files will see only opaque symbols.

Notes
-----
- This is semantic encryption, not cryptography.
- Security comes from shared symbol grounding.
- Extendable with deeper recursion and symbol permutations.
