import json
from pathlib import Path
import subprocess
import sys

import pytest
from jsonschema import Draft202012Validator

sys.path.insert(0, str(Path(__file__).parents[1]))
from prototypes.schoff_lab.scene_ir import compile_fgl_to_ir
from prototypes.schoff_lab.sdf import render_ascii, sample_scene, validate_scene

ROOT = Path(__file__).parents[1]


def test_compile_documented_example_into_versioned_ir():
    document = compile_fgl_to_ir("☉⊗⚘∎")
    assert document["schema"] == "schoff.fgl-ir/v1"
    assert document["source"]["symbols"] == ["☉", "⊗", "⚘"]
    assert [p["id"] for p in document["scene"]["primitives"]] == ["source-anchor", "life-node"]
    assert document["scene"]["relations"] == [{"kind": "transform", "from": "source-anchor", "to": "life-node", "source_symbol": "⊗"}]
    assert all(check["passed"] for check in document["constraints"])


def test_change_modifier_scales_next_object_reference_node():
    plain = compile_fgl_to_ir("☉⊗⚘")
    changed = compile_fgl_to_ir("☉∆⊗⚘")
    assert plain["scene"]["primitives"][1]["parameters"]["radius"] == 0.65
    assert changed["scene"]["primitives"][1]["parameters"]["radius"] == 0.8125


def test_balance_relation_emits_explicit_constraint():
    document = compile_fgl_to_ir("☉⟁⚘")
    assert document["scene"]["relations"][0]["kind"] == "balance"
    bound = next(check for check in document["constraints"] if check["name"] == "balance_distance_bound")
    assert bound["passed"] is True


def test_compile_rejects_clause_without_reference_subject():
    with pytest.raises(ValueError, match="requires a documented subject"):
        compile_fgl_to_ir("∆⟁⟡")


def test_sdf_sample_selects_nearest_primitive():
    scene = compile_fgl_to_ir("☉⊗⚘")["scene"]
    at_origin = sample_scene(scene, (0, 0, 0))
    at_life = sample_scene(scene, (1.4, 0, 0))
    assert at_origin.primitive_id == "source-anchor"
    assert at_origin.distance == -1.0
    assert at_life.primitive_id == "life-node"
    assert at_life.distance == -0.65


def test_sdf_validation_and_ascii_rendering():
    scene = compile_fgl_to_ir("☉⊗⚘")["scene"]
    checks = validate_scene(scene)
    assert all(check["passed"] for check in checks)
    art = render_ascii(scene, width=31, height=13, span=3)
    assert len(art.splitlines()) == 13
    assert all(len(line) == 31 for line in art.splitlines())
    assert "#" in art


def test_sdf_rejects_unknown_primitive():
    scene = {"dimension": 3, "composition": "union", "primitives": [{"id": "bad", "kind": "torus", "parameters": {}}]}
    with pytest.raises(ValueError, match="Unsupported SDF primitive"):
        sample_scene(scene, (0, 0, 0))


def test_cli_compile_sample_and_render_workflows(tmp_path):
    command = [sys.executable, "-m", "prototypes.schoff_lab.cli"]
    ir_path = tmp_path / "scene.json"
    subprocess.run(command + ["fgl-compile", "☉⊗⚘", "--output", str(ir_path)], cwd=ROOT, check=True)
    sample = subprocess.run(command + ["scene-sample", str(ir_path), "0", "0", "0"], cwd=ROOT, check=True, capture_output=True, text=True)
    render = subprocess.run(command + ["scene-render", str(ir_path), "--width", "11", "--height", "7"], cwd=ROOT, check=True, capture_output=True, text=True)
    assert json.loads(sample.stdout)["primitive_id"] == "source-anchor"
    assert len(render.stdout.splitlines()) == 7


def test_schemas_are_valid_json_documents():
    for name in ["fgl-ir-v1.json", "fgl-clause-v1.json", "mofp-verification-v1.json", "psse-v1.json", "tdm-event-v1.json"]:
        document = json.loads((ROOT / "schemas" / name).read_text(encoding="utf-8"))
        assert document["$schema"] == "https://json-schema.org/draft/2020-12/schema"
        Draft202012Validator.check_schema(document)


def test_compiled_ir_validates_against_fgl_ir_schema():
    schema = json.loads((ROOT / "schemas" / "fgl-ir-v1.json").read_text(encoding="utf-8"))
    document = compile_fgl_to_ir("☉∆⊗⚘")
    Draft202012Validator(schema).validate(document)
