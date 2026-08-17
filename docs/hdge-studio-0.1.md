# HDGE Studio 0.1

## Purpose

**HDGE Studio 0.1** is the first native desktop workbench for the Holographic Declarative Game Engine research program. It implements a narrow, inspectable software path: a documented FGL clause is parsed into a canonical HDGE scene, validated, passed through a three-phase TDM run, and visualized as a small analytic SDF reference scene.

> HDGE Studio is a declarative SDF-world authoring and validation prototype. It does **not** claim to implement literal five-dimensional physics, a physical hologram, a general-purpose AAA engine, or production-ready security.

## Implemented capabilities

| Surface | Current behavior |
|---|---|
| FGL editor | Edits the documented symbol subset `☉ Ϟ ⊗ ⚘ ⟡ ∆ ⟁ ✶`. |
| Compiler | Maps a source and object symbol to an explicit sphere-based reference scene. The `∆` modifier scales the object node. |
| Scene contract | Writes the in-memory `hdge.scene/v1` structure and stable SHA-256 digest. |
| Validation | Checks schema version, scene non-emptiness, unique IDs, finite parameters, positive radii, and camera values. |
| TDM runtime | Records deterministic `ANCHOR`, `INSERT`, and `EXPAND` events and reports eligibility for commit. |
| Viewport | Renders an analytic CPU-reference top-down inspection view with a distance-probe mode. |
| Evidence UI | Displays source symbols, validation checks, scene digest, TDM timeline, and canonical Scene IR. |
| Sample project | Includes `examples/fgl-life-seed/source.fgl` with the checked fixture `☉⊗∆⚘∎`. |

## Workspace layout

```text
Cargo.toml                         Rust workspace
crates/hdge-schema/                Versioned contracts, validation, canonical JSON, digests
crates/hdge-core/                  FGL compiler, TDM runner, CPU SDF probe evaluator
apps/hdge-studio/                  Native desktop editor shell
examples/fgl-life-seed/            Checked project fixture
schemas/hdge.*-v1.json             Machine-readable contract definitions
prototypes/schoff_lab/             Existing Python reference/oracle package
```

## Build and run

The development environment requires a current stable Rust toolchain with Cargo. On Linux, native windowing support is provided by the selected `eframe` feature set; use a graphical session to open the desktop window.

```sh
cd UCST-AI
cargo test --workspace
cargo run -p hdge-studio
```

The application opens the **FGL Life Seed** sample by default. Modify the FGL source or scene identifier and select **Compile + run TDM**. A successful supported source produces a new Scene IR and evidence record. Unknown symbols, missing subject/object symbols, and invalid scene parameters are reported without accepting a candidate revision.

## Development rules

The Rust core is the candidate application implementation. The existing Python package remains a reference oracle while the supported FGL subset and SDF evaluation are brought to parity. New semantic behavior must therefore add fixed fixtures and tests in both layers or document a deliberate versioned divergence.

The desktop interface must never become the source of truth. Project files, canonical contracts, digests, validation results, and backend reports are authoritative. A future MM3E bridge consumes `hdge.scene/v1` and emits a pinned-backend report; it does not own FGL semantics or TDM commit policy.[1] [2]

## Next engineering increments

The next increment is **HDGE Core completion**: file-based project loading/saving, `hdge.tdm-run/v1` and `hdge.backend-report/v1` artifacts on disk, scene-operation graphs, and CLI commands. The next backend increment is the version-pinned MM3E CPU bridge for the sphere/box/plane/union subset. Once CPU parity tests pass, the Studio viewport can swap its reference visualization for an MM3E frame and expose AOV artifacts.

MOFP remains optional at transaction boundaries. It must not be invoked for every renderer sample or physics timestep. Engineering mode may run normally without attestation; a future attested mode must require a separately valid HDGE execution pair and durable residue persistence before advancing an attested scene or physics-checkpoint pointer.[3]

## References

[1]: [HDGE Integration Blueprint](milestone-3-hdge-blueprint.md)
[2]: [MM3E Backend Roadmap](milestone-4-mm3e-backend-roadmap.md)
[3]: [MOFP–MM3E Dependency Review](mofp-mm3e-dependency-review.md)
