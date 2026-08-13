# Mutual Observer File Pair

This package preserves the submitted PDF and DOCX as a **paired archival record** for *The Mutual Observer File Pair: A Minimal Constraint Geometry for Proto-Living Digital Objects*.

The original source documents are not modified. `mutual-observer-pair.json` describes the pair, while `mofp-pdf.member.json` and `mofp-docx.member.json` reciprocally bind the fixed-layout PDF and editable DOCX through SHA-256 commitments. `mutual-observer-residue.ndjson` starts an append-only ledger intended to record verified joint accesses.

To treat the two originals as a mutually verified access pair, verify both file digests, verify the reciprocal member records and shared commitment, then append an immutable jointly attributable residue record. Git history provides versioned archival persistence, but an external signed ledger or repository-protected append-only process would be required to make every runtime access cryptographically irreversible.

