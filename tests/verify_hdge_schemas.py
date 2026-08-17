#!/usr/bin/env python3
"""Validate HDGE Studio's JSON Schema documents and checked fixtures."""
from __future__ import annotations

import json
from pathlib import Path

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
Draft202012Validator(sphere_schema).validate(sphere_fixture)
print("valid fixture: examples/sphere-world-basic/world.sphereworld.json")
