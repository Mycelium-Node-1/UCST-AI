//! Deterministic SphereWorld Slice 0 compiler.
//!
//! This crate derives meshes and observation frames from a canonical sphere-world
//! manifest. It intentionally keeps generated geometry out of persistent state.

use serde::{Deserialize, Serialize};
use sphere_world_schema::{
    accepted, digest_hex, validate_world, CubeFace, Layer, ObserverAnchor, PatchId, SphereWorld,
    StructureFootprint, ValidationCheck, Vec3,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error("SphereWorld manifest validation failed")]
    InvalidWorld,
    #[error("patch {0:?} is outside the topology limits")]
    InvalidPatch(PatchId),
    #[error("canonical contract error: {0}")]
    Contract(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceBasis {
    pub normal: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub direction: Vec3,
    pub walkable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchMesh {
    pub patch_id: PatchId,
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<[u32; 3]>,
    pub wire_segments: Vec<[u32; 2]>,
    pub skirt_triangle_count: usize,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurfaceFrame {
    pub origin: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub forward: Vec3,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureTransform {
    pub id: String,
    pub origin: Vec3,
    pub right: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub width_m: f64,
    pub depth_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldDiagnostics {
    pub world_digest: String,
    pub validation: Vec<ValidationCheck>,
    pub root_patch_count: usize,
    pub patch_resolution: u16,
    pub active_patch_count: usize,
}

pub fn face_basis(face: CubeFace) -> FaceBasis {
    match face {
        CubeFace::PosX => FaceBasis {
            normal: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            right: Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            up: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        },
        CubeFace::NegX => FaceBasis {
            normal: Vec3 {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            },
            right: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            up: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        },
        CubeFace::PosY => FaceBasis {
            normal: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            right: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            up: Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
        },
        CubeFace::NegY => FaceBasis {
            normal: Vec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            right: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            up: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        },
        CubeFace::PosZ => FaceBasis {
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            right: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            up: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        },
        CubeFace::NegZ => FaceBasis {
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            right: Vec3 {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            },
            up: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        },
    }
}

/// Map a face-local coordinate in [0, 1]^2 to the canonical unit sphere.
pub fn cube_sphere_direction(face: CubeFace, u: f64, v: f64) -> Vec3 {
    let basis = face_basis(face);
    let a = u * 2.0 - 1.0;
    let b = v * 2.0 - 1.0;
    (basis.normal + basis.right * a + basis.up * b).normalized()
}

fn patch_uv(patch_id: PatchId, local_u: f64, local_v: f64) -> (f64, f64) {
    let subdivisions = 1_u32 << patch_id.level;
    let u = (patch_id.x as f64 + local_u) / subdivisions as f64;
    let v = (patch_id.y as f64 + local_v) / subdivisions as f64;
    (u, v)
}

pub fn direction_for_patch(patch_id: PatchId, local_u: f64, local_v: f64) -> Vec3 {
    let (u, v) = patch_uv(patch_id, local_u, local_v);
    cube_sphere_direction(patch_id.face, u, v)
}

fn noise(direction: Vec3, seed: u64, frequency: f64) -> f64 {
    // Deterministic analytic value noise. The declared world seed is the only
    // variable input; no mutable RNG state or patch-order dependency exists.
    let phase = seed as f64 * 0.000_000_119_209_289_550_781_25;
    let a = (direction.x * frequency * 977.0 + phase).sin();
    let b = (direction.y * frequency * 1_387.0 + phase * 1.7).cos();
    let c = (direction.z * frequency * 1_879.0 - phase * 0.7).sin();
    (a + b + c) / 3.0
}

pub fn radial_height(world: &SphereWorld, direction: Vec3) -> f64 {
    world.layers.iter().fold(0.0, |height, layer| match layer {
        Layer::RadialNoise {
            amplitude_m,
            frequency,
            seed_offset,
            ..
        } => {
            height
                + noise(direction, world.seed.wrapping_add(*seed_offset), *frequency) * amplitude_m
        }
        Layer::LatitudeBand { .. } => height,
    })
}

pub fn is_walkable(world: &SphereWorld, direction: Vec3) -> bool {
    let latitude = direction.y.asin().to_degrees();
    world.layers.iter().all(|layer| match layer {
        Layer::LatitudeBand {
            min_degrees,
            max_degrees,
            behavior,
            ..
        } => !(*min_degrees <= latitude && latitude <= *max_degrees) || behavior != "blocked",
        Layer::RadialNoise { .. } => true,
    })
}

pub fn surface_position(world: &SphereWorld, direction: Vec3) -> Vec3 {
    direction * (world.radius_m + radial_height(world, direction))
}

fn surface_frame_at(
    world: &SphereWorld,
    face: CubeFace,
    u: f64,
    v: f64,
    altitude_m: f64,
    heading_degrees: f64,
) -> SurfaceFrame {
    let direction = cube_sphere_direction(face, u, v);
    let origin = surface_position(world, direction) + direction * altitude_m;
    let reference = if direction.y.abs() < 0.95 {
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    } else {
        Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    };
    let right = reference.cross(direction).normalized();
    let base_forward = direction.cross(right).normalized();
    let radians = heading_degrees.to_radians();
    let forward = (base_forward * radians.cos() + right * radians.sin()).normalized();
    let rotated_right = forward.cross(direction).normalized();
    SurfaceFrame {
        origin,
        up: direction,
        right: rotated_right,
        forward,
    }
}

pub fn observer_frame(world: &SphereWorld, anchor: &ObserverAnchor) -> SurfaceFrame {
    surface_frame_at(
        world,
        anchor.face,
        anchor.u,
        anchor.v,
        anchor.altitude_m,
        anchor.heading_degrees,
    )
}

pub fn structure_transform(
    world: &SphereWorld,
    footprint: &StructureFootprint,
) -> StructureTransform {
    let frame = surface_frame_at(
        world,
        footprint.face,
        footprint.u,
        footprint.v,
        0.0,
        footprint.heading_degrees,
    );
    StructureTransform {
        id: footprint.id.clone(),
        origin: frame.origin,
        right: frame.right,
        forward: frame.forward,
        up: frame.up,
        width_m: footprint.width_m,
        depth_m: footprint.depth_m,
    }
}

fn validate_or_error(world: &SphereWorld) -> Result<(), WorldError> {
    if accepted(&validate_world(world)) {
        Ok(())
    } else {
        Err(WorldError::InvalidWorld)
    }
}

fn append_skirt(
    vertices: &mut Vec<Vertex>,
    triangles: &mut Vec<[u32; 3]>,
    border: &[u32],
    depth: f64,
) -> usize {
    if border.len() < 2 {
        return 0;
    }
    let mut bottoms = Vec::with_capacity(border.len());
    for &top_index in border {
        let top = vertices[top_index as usize].clone();
        let bottom = Vertex {
            position: top.position - top.direction * depth,
            normal: top.normal,
            direction: top.direction,
            walkable: top.walkable,
        };
        bottoms.push(vertices.len() as u32);
        vertices.push(bottom);
    }
    let start_triangle_count = triangles.len();
    for index in 0..border.len() - 1 {
        let top_a = border[index];
        let top_b = border[index + 1];
        let bottom_a = bottoms[index];
        let bottom_b = bottoms[index + 1];
        triangles.push([top_a, top_b, bottom_a]);
        triangles.push([top_b, bottom_b, bottom_a]);
    }
    triangles.len() - start_triangle_count
}

pub fn compile_patch(world: &SphereWorld, patch_id: PatchId) -> Result<PatchMesh, WorldError> {
    validate_or_error(world)?;
    if !patch_id.is_valid_for(&world.topology) {
        return Err(WorldError::InvalidPatch(patch_id));
    }
    let resolution = world.topology.patch_resolution as usize;
    let mut vertices = Vec::with_capacity(resolution * resolution + resolution * 4);
    let mut triangles =
        Vec::with_capacity((resolution - 1) * (resolution - 1) * 2 + resolution * 8);
    let mut wire_segments = Vec::with_capacity(resolution * (resolution - 1) * 2);

    for row in 0..resolution {
        let local_v = row as f64 / (resolution - 1) as f64;
        for column in 0..resolution {
            let local_u = column as f64 / (resolution - 1) as f64;
            let direction = direction_for_patch(patch_id, local_u, local_v);
            vertices.push(Vertex {
                position: surface_position(world, direction),
                normal: direction,
                direction,
                walkable: is_walkable(world, direction),
            });
        }
    }

    for row in 0..resolution - 1 {
        for column in 0..resolution - 1 {
            let a = (row * resolution + column) as u32;
            let b = a + 1;
            let c = a + resolution as u32;
            let d = c + 1;
            triangles.push([a, b, c]);
            triangles.push([b, d, c]);
        }
    }

    for row in 0..resolution {
        for column in 0..resolution {
            let index = (row * resolution + column) as u32;
            if column + 1 < resolution {
                wire_segments.push([index, index + 1]);
            }
            if row + 1 < resolution {
                wire_segments.push([index, index + resolution as u32]);
            }
        }
    }

    let mut skirt_triangle_count = 0;
    if world.lod.use_edge_skirts {
        let bottom: Vec<u32> = (0..resolution as u32).collect();
        let top: Vec<u32> = ((resolution - 1) * resolution..resolution * resolution)
            .map(|index| index as u32)
            .collect();
        let left: Vec<u32> = (0..resolution)
            .map(|row| (row * resolution) as u32)
            .collect();
        let right: Vec<u32> = (0..resolution)
            .map(|row| (row * resolution + resolution - 1) as u32)
            .collect();
        let depth = (world.radius_m * 0.0025).max(0.25);
        skirt_triangle_count += append_skirt(&mut vertices, &mut triangles, &bottom, depth);
        skirt_triangle_count += append_skirt(&mut vertices, &mut triangles, &top, depth);
        skirt_triangle_count += append_skirt(&mut vertices, &mut triangles, &left, depth);
        skirt_triangle_count += append_skirt(&mut vertices, &mut triangles, &right, depth);
    }

    let mut mesh = PatchMesh {
        patch_id,
        vertices,
        triangles,
        wire_segments,
        skirt_triangle_count,
        digest: String::new(),
    };
    mesh.digest = digest_hex(&mesh).map_err(|error| WorldError::Contract(error.to_string()))?;
    Ok(mesh)
}

pub fn compile_root_patches(world: &SphereWorld) -> Result<Vec<PatchMesh>, WorldError> {
    CubeFace::ALL
        .into_iter()
        .map(|face| compile_patch(world, PatchId::root(face)))
        .collect()
}

pub fn split_patch(world: &SphereWorld, patch_id: PatchId) -> Result<Vec<PatchMesh>, WorldError> {
    validate_or_error(world)?;
    if patch_id.level >= world.topology.max_level {
        return Err(WorldError::InvalidPatch(patch_id));
    }
    patch_id
        .children()
        .into_iter()
        .map(|child| compile_patch(world, child))
        .collect()
}

pub fn diagnostics(
    world: &SphereWorld,
    active_patch_count: usize,
) -> Result<WorldDiagnostics, WorldError> {
    let validation = validate_world(world);
    if !accepted(&validation) {
        return Err(WorldError::InvalidWorld);
    }
    Ok(WorldDiagnostics {
        world_digest: digest_hex(world).map_err(|error| WorldError::Contract(error.to_string()))?,
        validation,
        root_patch_count: 6,
        patch_resolution: world.topology.patch_resolution,
        active_patch_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sphere_world_schema::{
        DerivedOutputs, LodPolicy, ProjectionKind, Topology, TopologyKind, WORLD_SCHEMA,
    };

    fn world(layers: Vec<Layer>) -> SphereWorld {
        SphereWorld {
            schema: WORLD_SCHEMA.into(),
            world_id: "test-world".into(),
            seed: 42,
            radius_m: 1000.0,
            topology: Topology {
                kind: TopologyKind::CubeSphere,
                projection: ProjectionKind::NormalizedCube,
                max_level: 3,
                patch_resolution: 5,
            },
            lod: LodPolicy {
                max_neighbor_level_delta: 1,
                use_edge_skirts: true,
            },
            layers,
            anchors: vec![],
            structures: vec![],
            derived_outputs: DerivedOutputs {
                render_mesh: true,
                wireframe: true,
                collision_mesh: true,
            },
        }
    }

    #[test]
    fn root_faces_generate_mesh_and_wireframe() {
        let meshes = compile_root_patches(&world(vec![])).unwrap();
        assert_eq!(meshes.len(), 6);
        assert!(meshes
            .iter()
            .all(|mesh| !mesh.wire_segments.is_empty() && mesh.skirt_triangle_count > 0));
    }

    #[test]
    fn unlayered_vertices_are_on_the_canonical_radius() {
        let mesh = compile_patch(&world(vec![]), PatchId::root(CubeFace::PosZ)).unwrap();
        for vertex in mesh.vertices.iter().take(25) {
            assert!((vertex.position.length() - 1000.0).abs() < 1e-9);
        }
    }

    #[test]
    fn matching_face_edge_maps_to_matching_direction() {
        let z_edge = cube_sphere_direction(CubeFace::PosZ, 1.0, 0.36);
        let x_edge = cube_sphere_direction(CubeFace::PosX, 0.0, 0.36);
        assert!((z_edge - x_edge).length() < 1e-12);
    }

    #[test]
    fn same_world_and_patch_produce_same_mesh_digest() {
        let world = world(vec![Layer::RadialNoise {
            id: "relief".into(),
            amplitude_m: 30.0,
            frequency: 0.008,
            seed_offset: 0,
        }]);
        let first = compile_patch(&world, PatchId::root(CubeFace::PosZ)).unwrap();
        let second = compile_patch(&world, PatchId::root(CubeFace::PosZ)).unwrap();
        assert_eq!(first.digest, second.digest);
        assert!(first
            .vertices
            .iter()
            .take(25)
            .any(|vertex| (vertex.position.length() - 1000.0).abs() > 1e-6));
    }

    #[test]
    fn observer_frame_is_orthonormal() {
        let world = world(vec![]);
        let anchor = ObserverAnchor {
            id: "camera".into(),
            kind: "camera_observer".into(),
            face: CubeFace::PosZ,
            level: 0,
            u: 0.5,
            v: 0.5,
            altitude_m: 2.0,
            heading_degrees: 0.0,
            pitch_degrees: 0.0,
        };
        let frame = observer_frame(&world, &anchor);
        assert!((frame.up.length() - 1.0).abs() < 1e-12);
        assert!(frame.up.dot(frame.forward).abs() < 1e-12);
        assert!(frame.right.dot(frame.forward).abs() < 1e-12);
    }

    #[test]
    fn parent_patch_splits_into_four_valid_child_meshes() {
        let world = world(vec![]);
        let children = split_patch(&world, PatchId::root(CubeFace::PosZ)).unwrap();
        assert_eq!(children.len(), 4);
        assert!(children.iter().all(|child| child.patch_id.level == 1));
    }
}
