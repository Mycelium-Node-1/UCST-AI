#!/usr/bin/env python3
"""Validate HDGE Studio's JSON Schema documents and checked fixtures."""
from __future__ import annotations

import copy
import json
from pathlib import Path

from jsonschema import ValidationError
from jsonschema.validators import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = [
    ROOT / "schemas" / "hdge.scene-v1.json",
    ROOT / "schemas" / "hdge.tdm-run-v1.json",
    ROOT / "schemas" / "hdge.backend-report-v1.json",
    ROOT / "schemas" / "sphere-world-v1.json",
]

for path in SCHEMAS:
    schema = json.loads(path.read_text(encoding="utf-8"))
    Draft202012Validator.check_schema(schema)
    print(f"valid schema: {path.relative_to(ROOT)}")

sphere_schema = json.loads((ROOT / "schemas" / "sphere-world-v1.json").read_text(encoding="utf-8"))
sphere_fixture = json.loads(
    (ROOT / "examples" / "sphere-world-basic" / "world.sphereworld.json").read_text(encoding="utf-8")
)
validator = Draft202012Validator(sphere_schema)
validator.validate(sphere_fixture)
print("valid fixture: examples/sphere-world-basic/world.sphereworld.json")

negative_cases = {
    "non_positive_radius": lambda candidate: candidate.update(radius_m=0),
    "unsupported_topology": lambda candidate: candidate["topology"].update(kind="icosphere"),
    "out_of_range_anchor_coordinate": lambda candidate: candidate["anchors"][0].update(u=1.1),
    "undeclared_manifest_property": lambda candidate: candidate.update(baked_mesh="not canonical world state"),
}
for name, mutate in negative_cases.items():
    candidate = copy.deepcopy(sphere_fixture)
    mutate(candidate)
    try:
        validator.validate(candidate)
    except ValidationError:
        print(f"rejected invalid fixture: {name}")
    else:
        raise AssertionError(f"invalid SphereWorld fixture was accepted: {name}")
