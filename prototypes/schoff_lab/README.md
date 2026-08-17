# Schoff Constraint Laboratory — Reference Package

This package is the first executable milestone for the formalizable artifacts in `Archive (2026)`. It is a deterministic research reference, not a completed Holographic Declarative Game Engine and not a security, medical, or consciousness claim.

## Commands

Run commands from the repository root using Python 3.11 or later.

```bash
python3 -m prototypes.schoff_lab.cli psse-encode "Hi, life!" --seed 3
python3 -m prototypes.schoff_lab.cli psse-decode examples/psse_hi_life_seed3.json --seed 3
python3 -m prototypes.schoff_lab.cli fgl-parse "☉⊗⚘∎"
python3 -m prototypes.schoff_lab.cli tdm-simulate --payload alpha --payload beta --cycles 2
python3 -m prototypes.schoff_lab.cli fgl-compile "☉⊗⚘∎" --output examples/source_transforms_life.ir.json
python3 -m prototypes.schoff_lab.cli scene-sample examples/source_transforms_life.ir.json 0 0 0
python3 -m prototypes.schoff_lab.cli scene-render examples/source_transforms_life.ir.json --width 31 --height 13
python3 -m prototypes.schoff_lab.cli mofp-verify "Archive (2026)/Mutual Observer File Pair"
```

The last command currently exits with status `2` because the archived package's declared canonical commitment does not match the reference derivation, although the two source hashes and reciprocal member metadata do match. This is intentional fail-closed behavior; see [`docs/milestone-1-contracts.md`](../../docs/milestone-1-contracts.md).

## Supported surface

| Capability | Supported behavior |
|---|---|
| PSSE | ASCII letters, the archive's six punctuation marks, case flags, word boundaries, and an optional seeded permutation |
| FGL | Documented symbols `☉ ⊗ ⚘ ∆ ⟁ ⟡ ✶ Ϟ`, with preserved archive meanings and roles |
| TDM | Deterministic `ANCHOR`, `INSERT`, and `EXPAND` cycle events |
| MOFP | Hash, reciprocal metadata, and canonical-commitment verification; residue append only after fully valid verification |
| FGL-IR | Versioned, source-provenanced JSON document retaining parsed FGL symbols, reference scene primitives, explicit relations, and constraint results |
| SDF | CPU evaluation of sphere, box, and plane primitives; z=0 ASCII inspection slices for union scenes |

The schemas under `schemas/` define the reference outputs. The FGL-to-scene mapping is deliberately an inspectable visualization policy rather than a claim that symbols have one universal physical geometry; see [`docs/milestone-2-contracts.md`](../../docs/milestone-2-contracts.md). Unsupported inputs fail explicitly rather than being inferred.
