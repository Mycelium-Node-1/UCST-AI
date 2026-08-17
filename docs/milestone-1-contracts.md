# Schoff Constraint Laboratory — Milestone 1 Contracts

## Purpose

Milestone 1 turns the most formalizable artifacts in `Archive (2026)` into a deterministic, repository-native reference package. It is a research implementation: it does not assert that the broader ontology, biological hypotheses, or HDGE physical claims have been validated.

## Supported scope

| Component | Milestone 1 contract | Compatibility policy |
|---|---|---|
| Mutual Observer File Pair (MOFP) | Verify source digests, reciprocal member metadata, declared commitment, and residue-record shape. Append a residue only after all enabled verification checks pass. | The archived package remains read-only. The reference tool reports its legacy commitment mismatch and never rewrites the archive. |
| PSSE | Deterministically encode and decode ASCII A–Z/a–z plus `. , ? ! ' "`, preserving case and word boundaries. An optional seed uses an explicit emitted-symbol index starting at 1. | The original mapping remains authoritative. Punctuation boundaries and index semantics are recorded as a v1 reference clarification. |
| FGL | Tokenize and parse the documented symbol subset into a typed clause representation. | Unsupported symbols, recursive folding semantics, and implicit compound meanings are rejected or preserved as future work rather than guessed. |
| TDM | Generate reproducible `ANCHOR → INSERT → EXPAND` event traces. | This is a software scheduler simulation, not a claim about hardware, biological, acoustic, or Triton-kernel behavior. |

## MOFP canonical commitment policy

The archived MOFP manifest defines its canonical commitment as the SHA-256 of newline-delimited ordered PDF and DOCX source digests, but the declared value does not match the straightforward derivation in either member order. The reference implementation therefore uses the following rules.

1. It treats the archived declared value as historical metadata.
2. It calculates a reference canonical value as `SHA256(pdf_sha256 + "\n" + docx_sha256)`.
3. It reports the declared and computed values separately.
4. It marks the pair as fully verified only if source hashes, reciprocal records, and canonical commitment all match.
5. It never appends a residue to a pair that does not fully verify.
6. It provides a future migration path in which a versioned manifest may specify a canonicalization profile and a repaired commitment without modifying the original archive artifacts.

## Invariants

| Area | Invariant |
|---|---|
| PSSE | `decode(encode(text, seed), seed) == text` for every supported input. |
| PSSE | Every emitted alphabetic object has `1 <= sides <= 26`, `orientation ∈ {0,1}`, and `boundary ∈ {0,1}`. |
| FGL | A parsed token is a known documented symbol and retains its archive-defined role and meaning. |
| TDM | Every full cycle emits exactly `ANCHOR`, `INSERT`, and `EXPAND` in that order. |
| MOFP | No residue append occurs unless the tool reports `verified: true`. |

## Explicit non-goals

Milestone 1 does not implement a game engine, physical fifth dimension, encryption, AI consciousness model, medical intervention, acoustic transduction model, GPU kernel, or production security system. FGL-EM and PSSE must not be represented as modern cryptography; the archive itself describes them as obfuscation or symbolic protocols rather than classical cryptographic protection.

## Acceptance criteria

The milestone is complete when the repository has a documented reference package, a command-line entry point, machine-readable schemas for the reference formats, examples that run without mutating archive originals, and a regression suite that passes in a clean Python environment.
