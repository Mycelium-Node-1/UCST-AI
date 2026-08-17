# SphereWorld Lab 0.1

## Purpose

SphereWorld Lab is a new **native desktop workspace inside HDGE Studio**. It is the first interactive surface for the SphereWorld Slice 0 engine. The application lets a user change canonical sphere-world parameters and immediately inspect the regenerated patch mesh, wireframe, surface observation point, structure anchor, validation checks, digests, and diagnostics.

> The editable values are canonical source data. Meshes, wireframes, skirts, anchor frames, and viewport marks are generated evidence, rebuilt from the manifest after each accepted change.

## Launch

The current implementation is a Rust desktop application using `eframe` and `egui`. From the repository root, launch it with:

```sh
cargo run -p hdge-studio
```

When the window opens, select **SphereWorld Lab** in the top workspace bar. The original **FGL Workbench** remains available in the same program.

## Manipulation controls

| Control | Changes | Resulting derived effect |
|---|---|---|
| Radius | Canonical radius in metres | Rebuilds the selected cube-sphere patch at the new radial scale. |
| Seed | Deterministic terrain seed | Changes radial relief reproducibly. |
| Relief amplitude and frequency | `radial_noise` layer parameters | Regenerates terrain displacement. |
| Blocked polar band | `latitude_band` start latitude | Reclassifies the affected samples as non-walkable. |
| Grid resolution | Patch sampling resolution | Rebuilds mesh, triangle, wireframe, and skirt output from one sampling pipeline. |
| Generate seam skirts | LOD edge-skirt policy | Adds or removes derived radial skirt triangles. |
| Split selected Pos Z patch | One bounded quadtree split | Derives the four level-one children of the active patch for visual inspection. |
| Altitude, heading, surface `u/v` | Surface observer anchor | Recomputes the local tangent observation frame. |
| Viewport drag and scroll | Local inspection camera only | Rotates or zooms the debug projection without editing the world manifest. |

The **Load checked fixture** button restores the versioned `examples/sphere-world-basic/world.sphereworld.json` file. The **Reset in-memory sample** button restores a small built-in equivalent for experimentation.

## What the viewport shows

The viewport is an explicit CPU reference visualization rather than a finished realtime renderer. It renders front-facing samples of the selected `pos_z` cube-sphere patch. Cyan samples are walkable terrain, coral samples belong to the blocked boundary, the gold point is the active observer anchor, and the amber square is the structure footprint. Wireframe display is optional. Turning on the split overlays four derived child patches.

The evidence panel reports the canonical world digest, active patch count, selected-mesh metrics, patch digest, manifest validation checks, observation frame, and structure-footprint metrics. The collapsible text panel shows the current canonical manifest JSON.

## Verification

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo check --workspace
python3 tests/verify_hdge_schemas.py
```

The desktop package includes a test that deserializes the checked SphereWorld fixture and confirms it passes the manifest validator. The SphereWorld core suite separately checks root generation, shared-edge directions, terrain reproducibility, surface frames, mesh/wireframe output, and one patch split.

## Current boundaries

SphereWorld Lab does not yet provide freeform file selection, persistent save-as, full 3D rendering, general mesh import, character controls, physics gameplay, GPU tessellation, or MM3E backend rendering. It is a tested manipulation and inspection workbench for the canonical sphere-world model. The next safe feature is project-file open/save and a proper 3D mesh viewport with explicit patch-neighbor selection; it should retain the same manifest-to-derived-artifact boundary.
