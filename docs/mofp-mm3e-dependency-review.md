# Mutual Observer File Pair and MM3E Backend Dependency Review

## Decision

The **MM3E renderer and SDF-native physics backend must not have a hard runtime dependency on the archived Mutual Observer File Pair (MOFP)**. MM3E can render, query distances, compute collision normals, and advance a bounded physics simulation from a valid HDGE scene without reading a MOFP package. The existing scene and TDM contracts likewise contain no MOFP field or access hook.

MOFP should instead be introduced as an **optional, transaction-level provenance and attestation layer** around accepted HDGE state transitions. It is appropriate for binding a canonical HDGE scene revision to its verified backend report or a physics-checkpoint record. It is not appropriate for binding every ray-march sample, render frame, collision query, or fixed physics timestep.

> Recommended model: **backend-independent execution; MOFP-gated attestation at accepted commit boundaries.**

This preserves the MOFP concept of coupled verification without making analytic rendering nondeterministic, slow, or dependent on an archival pair that currently does not fully verify.

## Source requirements

The MOFP paper defines four strong axioms. Its access-coupling rule says neither member may be read, written, or verified without concurrent reference to its partner. Its irreversible-residue rule says that every successful mutual verification appends a permanent joint update before content is released. Its staged construction makes the update sequence explicit: read partner commitment, verify it locally, append a monotonic sequence and joint hash, then release content.[1]

The archived package is a more limited implementation. It keeps the original PDF and DOCX immutable, uses reciprocal SHA-256 member records, and points to one `mutual-observer-residue.ndjson` ledger. Its README says that an external signed ledger or repository-protected append-only process would be needed to make runtime access cryptographically irreversible.[2] The operational MOFP verifier similarly considers a result fully verified only when source hashes, reciprocal metadata, and canonical commitment all pass.[3]

| MOFP concept | Current archived implementation | Consequence for MM3E integration |
|---|---|---|
| Reciprocal partners | PDF and DOCX reciprocal metadata records | Useful model for a **new** runtime provenance pair. |
| Access coupling | Manifest defines a preflight verification protocol | Appropriate before an attested scene/physics commit, not per frame. |
| Residue in both members | Paper’s A3 requires bilateral permanent update | Not directly implemented by the immutable PDF/DOCX archive package. |
| Append-only ledger | One external NDJSON ledger is specified | Needs a protected/signed persistence mechanism for stronger attestation. |
| Canonical commitment | Declared value does not match the documented straightforward derivation in the current archive package | Current archive pair cannot be a strict blocking runtime gate until repaired or superseded. |

## Why a per-frame or per-step MOFP dependency is incorrect

MM3E’s SDF renderer executes many world-field queries per pixel, ray, normal estimate, shadow, ambient-occlusion probe, and physics contact. Its SDF-native physics model similarly samples the field repeatedly during every fixed timestep. Requiring an MOFP verification and irreversible ledger append at each such event would conflate **observation** with **low-level numerical evaluation**.

That coupling would create four technical failures. First, it would make a pure field evaluation depend on mutable I/O, breaking simple CPU/GPU parity and reproducibility. Second, it would create enormous ledger volume and contention. Third, it would make GPU or multithreaded execution difficult to reason about because a frame contains many parallel evaluations. Fourth, it would not actually match the useful engineering boundary: a ray sample is not an accepted HDGE state transition.

The correct state-transition boundary is the end of a TDM `EXPAND` phase, after a candidate scene patch or physics checkpoint has passed validation. That is where a provenance record has semantic meaning and where a rollback decision can occur.

## Dependency classification

| Dependency | Classification | Required behavior |
|---|---|---|
| HDGE Scene IR → MM3E scene translation | **Hard** | Required for any backend render or physics execution. |
| HDGE TDM run record → backend report | **Hard** for accepted state changes | Required to explain which scene revision and staged command produced a result. |
| MOFP archive PDF/DOCX → MM3E renderer | **None** | The renderer must not load, mutate, or depend on the archival pair. |
| MOFP verification status → normal engineering render | **Soft / observational** | Record status if supplied; do not block ordinary smoke tests or parity runs. |
| Verified MOFP binding → an “attested” HDGE commit | **Hard within attested profile only** | Refuse attestation if pair verification or durable residue persistence fails. |
| MOFP residue → individual render sample / physics timestep | **Prohibited** | No per-sample, per-frame, or per-step residues. |
| MOFP residue → accepted static-world revision or physics checkpoint | **Recommended** | Append one durable, jointly attributable record at a defined commit boundary. |

## Recommended runtime pair model

The project should not reuse or mutate the archival PDF/DOCX pair as a backend control file. It should create a new, versioned **HDGE execution pair** under a separate schema.

| Pair member | Canonical content | Purpose |
|---|---|---|
| Member A: `hdge-scene` | Canonical `hdge.scene/v1`, its digest, parent scene digest, declared constraints, and approved TDM-run reference | Represents the committed static world. |
| Member B: `backend-evidence` | Canonical `hdge.backend-report/v1` or `hdge.physics-run/v1`, backend revision, render/probe/physics evidence, and member-A digest | Represents the observed execution result for that exact world. |
| Shared commitment | Canonical hash over the ordered A and B member digests, using a published byte-level derivation and test vectors | Binds the declared world to observed backend behavior. |
| Residue ledger | Protected append-only log of accepted pair transitions | Records only accepted scene revisions and selected physics checkpoints. |

The relationship is intentionally asymmetric in function while symmetric in verification. The scene member declares what the world is. The evidence member records what the pinned backend did with that world. Each member contains the other’s digest and the same shared commitment. Neither substitute for the other.

## TDM transaction protocol with optional MOFP attestation

| Phase | Standard engineering profile | Attested profile |
|---|---|---|
| `ANCHOR` | Load committed scene digest, backend revision, previous backend report, and static physics world. | Also verify the previous execution pair and ledger head before treating the prior state as attested. |
| `INSERT` | Stage a scene patch, renderer setting request, or dynamic-body command. | Create candidate member records in a private staging area; do not append a residue. |
| `EXPAND` | Translate, run parity checks, render/probe, perform bounded physics validation, then accept or roll back. | On acceptance, derive new A/B member digests, verify reciprocal binding, append one protected residue, then atomically advance the current-attested pointer. |

An engineering run may complete even if no MOFP is supplied. It must then report `attestation_status: "not_requested"` or `"unverified"`. An attested run may complete only if all pairing and persistence gates pass. It must never report a run as mutually verified merely because a backend render was produced.

## Physics checkpoint policy

Static-world changes and dynamic physics are different kinds of state. The static HDGE world changes only after an accepted TDM transaction. Dynamic spheres, positions, velocities, and contact results may update at a fixed rate under that world.

The recommended MOFP boundary for physics is a **checkpoint**, not a timestep. A checkpoint can occur after a completed command, at an explicitly requested simulation time, on collision-set change, or at a fixed interval chosen for the experiment. A checkpoint evidence member should contain the prior checkpoint digest, static scene digest, time interval, timestep, integrator parameters, body-state digest, and validation summary. This allows replay and audit without turning the physics loop into a ledger writer.

## Required schema additions

| New schema | Required fields |
|---|---|
| `hdge.execution-pair/v1` | pair ID, ordered members, member digests, canonicalization profile ID, shared commitment, ledger reference, parent pair ID, verification status |
| `hdge.execution-member/v1` | member ID, role (`scene` or `evidence`), payload path/digest, partner ID/digest, shared commitment, parent member digest |
| `hdge.execution-residue/v1` | monotonic sequence, prior residue hash, pair ID, scene digest, evidence digest, shared commitment, TDM run digest, timestamp source, signer/persistence proof reference |
| `hdge.attestation-status/v1` | profile (`engineering` or `attested`), verification results, durable-ledger status, failure reason, report links |

The canonicalization profile must be explicit and testable. The existing archive package’s commitment discrepancy is strong evidence that the derivation cannot remain prose-only.

## Failure policy

| Condition | Engineering profile | Attested profile |
|---|---|---|
| No MOFP supplied | Continue; mark `not_requested`. | Reject before anchor. |
| Archived legacy pair supplied | Continue only as a recorded historical reference; mark `legacy_commitment_mismatch`. | Reject; it is not fully verified. |
| New execution pair hash mismatch | Continue only if the caller explicitly chooses non-attested mode; record failure. | Reject and roll back candidate pointer. |
| Ledger unavailable or not durable | Continue as non-attested; do not append local residue as proof of irreversibility. | Reject the attested commit. |
| MM3E render/parity/physics validation fails | Roll back candidate scene/state. | Roll back; append no acceptance residue. |

## Changes required to the Milestone 4 roadmap

The roadmap should be amended in four places. First, its backend topology needs an optional execution-pair attestation layer above the MM3E adapter, rather than an archive dependency inside the renderer. Second, the `ANCHOR` and `EXPAND` descriptions need two profiles: ordinary engineering execution and attested execution. Third, the deliverables need the execution-pair schemas and physics-checkpoint policy. Fourth, the bridge needs tests proving that MOFP failure never alters a normal renderer output and that a failed attested transaction never advances the current-attested pointer.

## Commit recommendation

**Do not commit the current Milestone 4 roadmap unchanged.** Commit it only after incorporating the transaction-level MOFP coupling model described here. The revision does not enlarge the initial MM3E rendering scope; it clarifies a necessary boundary and prevents an invalid dependency from entering the architecture.

## References

[1]: `Archive (2026)/Mutual Observer File Pair/Mutual_Observer_File_Pair.docx`, sections III–VI.
[2]: `Archive (2026)/Mutual Observer File Pair/README.md`.
[3]: `schemas/mofp-verification-v1.json`.
[4]: `docs/milestone-4-mm3e-backend-roadmap.md`.
