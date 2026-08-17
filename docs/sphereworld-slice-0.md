# SphereWorld Slice 0

## Scope

**SphereWorld Slice 0** is a standalone Rust subsystem that turns one canonical declarative sphere-world manifest into deterministic cube-sphere patches, terrain samples, wireframes, triangle meshes, seam skirts, surface-anchor frames, and patch-local structure transforms. It is intentionally separate from the existing FGL/SDF core so that the sphere-first model can be tested before any HDGE Studio user-interface integration.

> The source of truth is `sphere-world/v1`. Generated vertices, triangle indices, wire segments, collision candidates, camera transforms, and debug data are derived artifacts and can be rebuilt from the same world declaration.

## Implemented capabilities

| Capability | Implementation | Evidence |
|---|---|---|
| Canonical world contract | `sphere-world-schema` contains the versioned manifest, validation checks, patch identifiers, and deterministic digests. | Validates positive radii, bounded topology, seam policy, layer parameters, unique anchors, and structure footprints. |
| Sphere topology | `sphere-world-core` implements six normalized cube faces with stable `(face, level, x, y)` patch IDs. | Unit test confirms a shared `pos_z` / `pos_x` edge maps to the same canonical direction. |
| Derived mesh data | A patch compiles an odd-resolution grid into vertices, triangles, and wire segments from one sampling pipeline. | Root faces generate both filled triangles and wire data. |
| Terrain | The declared `radial_noise` layer evaluates as a pure analytic function of direction, world seed, and parameters. | Same world and patch yield the same mesh digest; relief changes the radial surface. |
| Boundaries | `latitude_band` can classify a spherical region as blocked without changing canonical topology. | The checked fixture blocks its polar band and leaves an equatorial sample walkable. |
| Seam management | Every generated patch can add a downward radial skirt along its four borders. | Slice test verifies skirt triangles exist; one root patch can split into four child patches. |
| Observation anchor | A surface anchor produces an altitude-adjusted, orthonormal local frame. | Test verifies radial up and orthogonal forward/right directions. |
| Architecture anchor | A declarative footprint yields a local tangent-frame transform. | Checked fixture derives the `observation-platform` transform from the canonical sphere. |

## Source layout

```text
crates/sphere-world-schema/  Manifest types, validation, vector math, patch IDs, digest
crates/sphere-world-core/    Projection, layers, patch compilation, skirts, anchors, structures
schemas/sphere-world-v1.json Machine-readable JSON Schema
examples/sphere-world-basic/ Checked SphereWorld manifest
```

The sample world starts with a 1000-metre sphere, fixed seed `42`, a radial terrain layer, a blocked polar boundary band, a surface camera anchor, and an anchored observation platform. It is deliberately compact so the compiler has a stable regression fixture.

## Run the checks

```sh
cd UCST-AI
cargo test -p sphere-world-schema -p sphere-world-core
python3 tests/verify_hdge_schemas.py
```

The core test suite checks canonical root radius, cube-face edge continuity, deterministic terrain/mesh digests, orthonormal observer frames, root-face mesh/wireframe generation, and one four-child patch split. The integration test loads `examples/sphere-world-basic/world.sphereworld.json` exactly as a future application will.

## Current limits

This is **not** yet a full planet renderer, character controller, physics engine, mesh importer, or HDGE Studio editing panel. It uses mesh-derived terrain and collision candidates; it does not assert that radial terrain is an exact signed-distance field. MM3E integration therefore remains a later backend task requiring either a validated terrain field representation or a conservative mesh-to-field conversion.

## Next implementation sequence

The immediate next slice should be a `sphere-world-lab` native debug application or an HDGE Studio panel that loads the checked manifest, renders the six base patches, toggles wireframe, displays the active anchor frame, and visualizes the one split `pos_z` patch. Once that visual proof is stable, add manifest load/save, camera-controlled LOD selection, explicit neighbor mapping with one-level seam stitching, and separate render/collision patch budgets.

The later HDGE Studio integration should persist only a `sphere_world` component referencing the validated manifest digest. An HDGE TDM transaction stages a candidate manifest, compiles candidate patches, validates topology/edges/bounds/digests, and only then accepts or rolls back the declaration.
