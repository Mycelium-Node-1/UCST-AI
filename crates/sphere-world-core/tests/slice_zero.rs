use sphere_world_core::{
    compile_root_patches, diagnostics, observer_frame, split_patch, structure_transform,
};
use sphere_world_schema::{CubeFace, PatchId, SphereWorld, Vec3};
use std::path::Path;

fn sample_world() -> SphereWorld {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/sphere-world-basic/world.sphereworld.json");
    let raw = std::fs::read_to_string(path).expect("read checked sphere-world sample");
    serde_json::from_str(&raw).expect("deserialize sphere-world/v1 sample")
}

#[test]
fn sphereworld_slice_zero_is_reproducible_and_derives_all_primary_outputs() {
    let world = sample_world();
    let root_patches = compile_root_patches(&world).expect("compile six cube-sphere roots");
    assert_eq!(root_patches.len(), 6);
    assert!(root_patches.iter().all(|patch| !patch.vertices.is_empty()));
    assert!(root_patches.iter().all(|patch| !patch.triangles.is_empty()));
    assert!(root_patches
        .iter()
        .all(|patch| !patch.wire_segments.is_empty()));
    assert!(root_patches
        .iter()
        .all(|patch| patch.skirt_triangle_count > 0));

    let split =
        split_patch(&world, PatchId::root(CubeFace::PosZ)).expect("split active observation patch");
    assert_eq!(split.len(), 4);
    assert!(split.iter().all(|patch| patch.patch_id.level == 1));

    let frame = observer_frame(&world, &world.anchors[0]);
    assert!((frame.up.length() - 1.0).abs() < 1e-12);
    assert!(frame.up.dot(frame.forward).abs() < 1e-12);

    let structure = structure_transform(&world, &world.structures[0]);
    assert_eq!(structure.id, "observation-platform");
    assert!((structure.origin.length() - world.radius_m).abs() < 50.0);

    let polar_direction = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    let equatorial_direction = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    assert!(!sphere_world_core::is_walkable(&world, polar_direction));
    assert!(sphere_world_core::is_walkable(&world, equatorial_direction));

    let report = diagnostics(&world, root_patches.len() + split.len()).expect("derive diagnostics");
    assert_eq!(report.root_patch_count, 6);
    assert_eq!(report.active_patch_count, 10);
    assert_eq!(report.world_digest.len(), 64);
}
