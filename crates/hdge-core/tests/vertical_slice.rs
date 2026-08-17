use hdge_core::{compile_fgl, execute_tdm, sample_scene};
use hdge_schema::{accepted, Vec3};
use std::path::Path;

#[test]
fn checked_sample_project_executes_the_vertical_slice() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/fgl-life-seed/source.fgl");
    let source = std::fs::read_to_string(source_path).expect("read checked FGL sample source");
    let compilation = compile_fgl(&source, "life-seed").expect("compile documented FGL subset");
    let run = execute_tdm(&compilation, "uncommitted");

    assert!(accepted(&compilation.validation));
    assert!(run.accepted);
    assert_eq!(run.events.len(), 3);
    assert_eq!(compilation.scene.primitives.len(), 2);

    let source_probe = sample_scene(&compilation.scene, Vec3::ZERO);
    assert_eq!(source_probe.primitive_id.as_deref(), Some("source-anchor"));
    assert_eq!(source_probe.distance, -1.0);

    let again = compile_fgl(&source, "life-seed").expect("recompile checked fixture");
    assert_eq!(compilation.scene_digest, again.scene_digest);
}
