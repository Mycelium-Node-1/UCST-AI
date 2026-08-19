# Finalized Game Engine

**HDGE Studio / SphereWorld Slice 0** is a native desktop workbench for inspecting and editing canonical declarative worlds. The included sample is a `cube_sphere` world: terrain, boundaries, patch meshes, wireframes, seam skirts, observer frames, and structure transforms are derived from one validated manifest.

> This package is a development milestone, not a claim of literal holographic or 5D physical behavior. GPU shading is a diagnostic visualization only; the validated manifest and CPU world compiler remain the semantic authority.

## Installed layout

| Installed path | Purpose |
|---|---|
| `Finalized-Game-Engine.exe` | Native 64-bit Windows HDGE Studio application. |
| `worlds\sphere-world-basic.sphereworld.json` | Checked canonical SphereWorld sample. |
| `config\package.json` | Non-authoritative package metadata. |
| `docs\` | Studio, SphereWorld, architectural, and backend guidance. |
| `LICENSES\LICENSE-MIT.txt` | Package license notice. |
| `VERSION`, `SHA256SUMS.txt` | Build identity and integrity data. |

## Use

Open **Finalized Game Engine** from the Start Menu. The application provides an FGL Workbench and SphereWorld Lab. The supplied world manifest remains the source of truth; generated meshes, wireframes, shader output, and other runtime buffers are derived evidence rather than editable persistent world state.

## Support boundary

This engine slice supports validated `sphere-world/v1` worlds, deterministic radial terrain, one latitude boundary rule, cube-sphere patch meshes and wireframes, seam skirts, observer anchors, and structure footprints. It does not yet include general mesh import, character controls, gameplay physics, a production renderer, project file open/save, or MM3E backend execution.
