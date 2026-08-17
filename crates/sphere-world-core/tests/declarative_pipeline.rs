use sphere_world_core::{
    compile_patch, compile_root_patches, cube_sphere_direction, direction_for_patch, is_walkable,
    locate_direction, observer_frame, radial_height, split_patch, structure_transform,
    trace_surface,
};
use sphere_world_schema::{accepted, validate_world, CubeFace, Layer, PatchId, SphereWorld, Vec3};

fn fixture() -> SphereWorld {
    serde_json::from_str(include_str!(
        "../../../examples/sphere-world-basic/world.sphereworld.json"
    ))
    .expect("checked sphere-world fixture must deserialize")
}

fn length_between(a: Vec3, b: Vec3) -> f64 {
    (a - b).length()
}

#[test]
fn canonical_fixture_has_no_validation_failures() {
    let world = fixture();
    let checks = validate_world(&world);
    assert!(accepted(&checks));
    assert!(checks.iter().all(|check| check.passed));
}

#[test]
fn every_root_face_center_round_trips_through_direction_location_and_trace() {
    let world = fixture();
    for face in CubeFace::ALL {
        let direction = cube_sphere_direction(face, 0.5, 0.5);
        let (located_face, u, v) = locate_direction(direction);
        assert_eq!(
            located_face, face,
            "face center must select its source face"
        );
        assert!((u - 0.5).abs() < 1e-12);
        assert!((v - 0.5).abs() < 1e-12);
        let trace = trace_surface(&world, direction, 4).expect("trace must succeed");
        assert_eq!(trace.patch_id.face, face);
        assert_eq!(trace.patch_id.level, 4);
        assert!(length_between(trace.direction, direction) < 1e-12);
        assert!(trace.patch_id.is_valid_for(&world.topology));
    }
}

#[test]
fn patch_slices_partition_the_selected_parent_direction_domain() {
    let parent = PatchId::root(CubeFace::PosZ);
    let children = parent.children();
    let expected = [
        PatchId {
            face: CubeFace::PosZ,
            level: 1,
            x: 0,
            y: 0,
        },
        PatchId {
            face: CubeFace::PosZ,
            level: 1,
            x: 1,
            y: 0,
        },
        PatchId {
            face: CubeFace::PosZ,
            level: 1,
            x: 0,
            y: 1,
        },
        PatchId {
            face: CubeFace::PosZ,
            level: 1,
            x: 1,
            y: 1,
        },
    ];
    assert_eq!(children, expected);

    let child_centers: Vec<_> = children
        .into_iter()
        .map(|patch| direction_for_patch(patch, 0.5, 0.5))
        .collect();
    assert!(child_centers.iter().all(|direction| direction.z > 0.5));
    assert!(child_centers
        .windows(2)
        .any(|pair| length_between(pair[0], pair[1]) > 0.1));
}

#[test]
fn cube_face_edges_map_to_identical_canonical_directions() {
    let z_to_x = cube_sphere_direction(CubeFace::PosZ, 1.0, 0.25);
    let x_to_z = cube_sphere_direction(CubeFace::PosX, 0.0, 0.25);
    let z_to_y = cube_sphere_direction(CubeFace::PosZ, 0.25, 1.0);
    let y_to_z = cube_sphere_direction(CubeFace::PosY, 0.25, 0.0);
    let z_to_neg_x = cube_sphere_direction(CubeFace::PosZ, 0.0, 0.75);
    let neg_x_to_z = cube_sphere_direction(CubeFace::NegX, 1.0, 0.75);
    assert!(length_between(z_to_x, x_to_z) < 1e-12);
    assert!(length_between(z_to_y, y_to_z) < 1e-12);
    assert!(length_between(z_to_neg_x, neg_x_to_z) < 1e-12);
}

#[test]
fn generated_mesh_topology_matches_declared_resolution_and_skirt_policy() {
    let world = fixture();
    let mesh = compile_patch(&world, PatchId::root(CubeFace::PosZ)).expect("compile root patch");
    let n = world.topology.patch_resolution as usize;
    let base_vertices = n * n;
    let base_triangles = 2 * (n - 1) * (n - 1);
    let wire_segments = 2 * n * (n - 1);
    let skirt_triangles = 8 * (n - 1);
    assert_eq!(mesh.wire_segments.len(), wire_segments);
    assert_eq!(mesh.skirt_triangle_count, skirt_triangles);
    assert_eq!(mesh.triangles.len(), base_triangles + skirt_triangles);
    assert_eq!(mesh.vertices.len(), base_vertices + 4 * n);

    let mut without_skirts = world.clone();
    without_skirts.lod.use_edge_skirts = false;
    let plain = compile_patch(&without_skirts, PatchId::root(CubeFace::PosZ))
        .expect("compile no-skirt patch");
    assert_eq!(plain.vertices.len(), base_vertices);
    assert_eq!(plain.triangles.len(), base_triangles);
    assert_eq!(plain.skirt_triangle_count, 0);
}

#[test]
fn mesh_generation_is_deterministic_and_seed_sensitive() {
    let world = fixture();
    let patch = PatchId {
        face: CubeFace::PosZ,
        level: 2,
        x: 1,
        y: 2,
    };
    let first = compile_patch(&world, patch).expect("first mesh");
    let second = compile_patch(&world, patch).expect("second mesh");
    assert_eq!(first.digest, second.digest);

    let mut changed_seed = world.clone();
    changed_seed.seed += 1;
    let different = compile_patch(&changed_seed, patch).expect("changed seed mesh");
    assert_ne!(first.digest, different.digest);
}

#[test]
fn terrain_layer_amplitude_controls_radial_displacement_without_changing_topology() {
    let world = fixture();
    let direction = cube_sphere_direction(CubeFace::PosZ, 0.63, 0.41);
    let source_height = radial_height(&world, direction);
    assert!(source_height.abs() <= 35.0);

    let mut flat = world.clone();
    if let Layer::RadialNoise { amplitude_m, .. } = &mut flat.layers[0] {
        *amplitude_m = 0.0;
    }
    assert_eq!(radial_height(&flat, direction), 0.0);
    let world_mesh = compile_patch(&world, PatchId::root(CubeFace::PosZ)).expect("terrain mesh");
    let flat_mesh = compile_patch(&flat, PatchId::root(CubeFace::PosZ)).expect("flat mesh");
    assert_eq!(world_mesh.wire_segments, flat_mesh.wire_segments);
    assert_ne!(world_mesh.digest, flat_mesh.digest);
}

#[test]
fn declared_polar_boundary_classifies_traces_and_mesh_vertices() {
    let world = fixture();
    let polar = trace_surface(
        &world,
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        2,
    )
    .expect("polar trace");
    let equatorial = trace_surface(
        &world,
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        2,
    )
    .expect("equatorial trace");
    assert!(!polar.walkable);
    assert!(equatorial.walkable);
    assert!(!is_walkable(
        &world,
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0
        }
    ));

    let top_mesh = compile_patch(&world, PatchId::root(CubeFace::PosY)).expect("top face mesh");
    assert!(top_mesh
        .vertices
        .iter()
        .take((world.topology.patch_resolution as usize).pow(2))
        .any(|vertex| !vertex.walkable));
}

#[test]
fn observer_and_structure_anchors_remain_tangent_to_the_derived_surface() {
    let world = fixture();
    let frame = observer_frame(&world, &world.anchors[0]);
    assert!((frame.up.length() - 1.0).abs() < 1e-12);
    assert!(frame.up.dot(frame.right).abs() < 1e-12);
    assert!(frame.up.dot(frame.forward).abs() < 1e-12);
    assert!(frame.right.dot(frame.forward).abs() < 1e-12);

    let structure = structure_transform(&world, &world.structures[0]);
    assert!(structure.up.dot(structure.right).abs() < 1e-12);
    assert!(structure.up.dot(structure.forward).abs() < 1e-12);
    assert!(structure.width_m > 0.0 && structure.depth_m > 0.0);
}

#[test]
fn six_root_patches_and_four_child_patches_compile_from_same_manifest() {
    let world = fixture();
    let roots = compile_root_patches(&world).expect("six roots");
    let children = split_patch(&world, PatchId::root(CubeFace::PosZ)).expect("four child patches");
    assert_eq!(roots.len(), 6);
    assert_eq!(children.len(), 4);
    assert!(children.iter().all(|mesh| mesh.patch_id.level == 1));
    assert!(roots.iter().all(|mesh| mesh.patch_id.level == 0));
}

#[test]
fn invalid_inputs_fail_closed_before_geometry_is_derived() {
    let mut invalid_world = fixture();
    invalid_world.radius_m = f64::NAN;
    assert!(compile_patch(&invalid_world, PatchId::root(CubeFace::PosZ)).is_err());

    let world = fixture();
    assert!(trace_surface(&world, Vec3::ZERO, 0).is_err());
    assert!(trace_surface(
        &world,
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0
        },
        world.topology.max_level + 1
    )
    .is_err());
    assert!(compile_patch(
        &world,
        PatchId {
            face: CubeFace::PosZ,
            level: 2,
            x: 4,
            y: 0
        }
    )
    .is_err());
}
