# Schoff Constraint Laboratory — Milestone 2 Contracts

## Purpose

Milestone 2 introduces a versioned **FGL Intermediate Representation (FGL-IR)** and a small declarative signed-distance-field (SDF) scene model. The goal is to make documented FGL symbols inspectable as deterministic scene declarations. This is a research visualization and constraint-evaluation surface; it does not claim to render a literal fifth dimension or establish any physical relationship between symbols and matter.

## FGL-IR v1

An FGL-IR document has the following conceptual fields.

| Field | Purpose |
|---|---|
| `schema` | Exact version identifier: `schoff.fgl-ir/v1`. |
| `source` | Original FGL source text and its parsed symbol sequence. |
| `semantics` | Archive-defined meanings and syntactic roles retained without reinterpretation. |
| `scene` | A deterministic, inspectable list of declarative SDF primitives. |
| `constraints` | Explicit checks applied to the scene, with pass/fail results. |
| `provenance` | Compiler name and a statement of the reference mapping policy. |

The compiler is deliberately one-way in Milestone 2: it preserves FGL syntax and maps known symbols to a reference scene. It does not infer natural-language intent, emulate recursive folding, or claim that the output is the unique geometrical form of the input.

## Reference symbol-to-scene mapping

| FGL symbol | Archive role | Milestone 2 reference interpretation | Primitive effect |
|---|---|---|---|
| `☉` | subject / source | Anchor node | Unit sphere centered at the origin |
| `Ϟ` | subject / energy | Dynamic node | Sphere at `(-1.4, 0, 0)` with radius `0.5` |
| `⊗` | verb / transformation | Relation operator | Records a `transform` relation from subject to object; does not mutate geometry silently |
| `⟁` | verb / balance | Relation operator | Records a `balance` relation and checks bounded center distance |
| `✶` | verb / creation | Relation operator | Records a `create` relation; object geometry remains explicit in the IR |
| `⚘` | object / life | Flora node | Sphere at `(1.4, 0, 0)` with radius `0.65` |
| `⟡` | object / light | Light marker | Sphere at `(1.4, 0.9, 0)` with radius `0.3` |
| `∆` | modifier / change | Scale modifier | Scales the next object primitive by `1.25` |

A source clause must contain a subject and an object. A relation is emitted when a documented verb appears. The reference mapping treats FGL roles as source provenance, not as a claim that these symbols have universal geometric meaning.

## Declarative SDF subset

Milestone 2 supports a three-dimensional point `(x, y, z)` and these primitives:

| Primitive | Required parameters | Signed-distance function |
|---|---|---|
| `sphere` | `center: [x,y,z]`, `radius > 0` | `length(p - center) - radius` |
| `box` | `center: [x,y,z]`, `half_extents: [x,y,z]` | Standard axis-aligned box SDF |
| `plane` | `normal: [x,y,z]`, `offset` | `dot(normalized(normal), p) + offset` |

A scene's distance is the minimum distance over its primitives, equivalent to a union operation. Each query returns both the nearest signed distance and the identifier of the nearest primitive. ASCII rendering samples the `z=0` plane only and is an inspection aid, not a physical renderer.

## Constraints and failure behavior

The compiler rejects empty FGL clauses and unsupported symbols. The evaluator rejects unknown primitives, non-positive radii, malformed vectors, and zero-length plane normals. A compiled scene is considered `well_formed` only if every primitive validates and all mandatory source roles are present.

Milestone 2 does not invoke Z3. The `constraints` collection is intentionally simple and transparent so a future SMT backend can compile each check without changing the IR contract.
