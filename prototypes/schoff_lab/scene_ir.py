"""Milestone 2 reference compiler from documented FGL symbols to FGL-IR v1.

The mapping is intentionally explicit and inspectable. It is a reference visualization
policy, not a claim that FGL symbols possess uniquely determined physical geometry.
"""
from __future__ import annotations

from math import dist
from typing import Any

from .core import FGL_SYMBOLS, parse_fgl

SCHEMA = "schoff.fgl-ir/v1"
COMPILER = "schoff-lab-m2-reference"
MAPPING_POLICY = "Milestone 2 documented-subset reference mapping; inspectable research visualization only."

NODE_MAP: dict[str, dict[str, Any]] = {
    "☉": {"id": "source-anchor", "kind": "sphere", "parameters": {"center": [0.0, 0.0, 0.0], "radius": 1.0}},
    "Ϟ": {"id": "energy-node", "kind": "sphere", "parameters": {"center": [-1.4, 0.0, 0.0], "radius": 0.5}},
    "⚘": {"id": "life-node", "kind": "sphere", "parameters": {"center": [1.4, 0.0, 0.0], "radius": 0.65}},
    "⟡": {"id": "light-node", "kind": "sphere", "parameters": {"center": [1.4, 0.9, 0.0], "radius": 0.3}},
}
RELATION_MAP = {"⊗": "transform", "⟁": "balance", "✶": "create"}


def _primitive_for(symbol: str, scale: float = 1.0) -> dict[str, Any]:
    base = NODE_MAP[symbol]
    parameters = {key: list(value) if isinstance(value, list) else value for key, value in base["parameters"].items()}
    if "radius" in parameters:
        parameters["radius"] = round(parameters["radius"] * scale, 8)
    return {"id": base["id"], "kind": base["kind"], "source_symbol": symbol, "parameters": parameters}


def _find_first(tokens: list[str], role: str) -> str | None:
    for token in tokens:
        if FGL_SYMBOLS[token]["role"] == role:
            return token
    return None


def _find_last(tokens: list[str], role: str) -> str | None:
    for token in reversed(tokens):
        if FGL_SYMBOLS[token]["role"] == role:
            return token
    return None


def compile_fgl_to_ir(text: str) -> dict[str, Any]:
    """Compile one documented FGL clause into FGL-IR v1.

    The source must include an anchor-capable subject and object. Unsupported syntax
    fails explicitly, which keeps the reference compiler deterministic and auditable.
    """
    clause = parse_fgl(text)
    tokens = list(clause.symbols)
    subject = _find_first(tokens, "subject")
    obj = _find_last(tokens, "object")
    if subject is None:
        raise ValueError("FGL-IR compilation requires a documented subject symbol.")
    if obj is None:
        raise ValueError("FGL-IR compilation requires a documented object symbol.")
    if subject not in NODE_MAP:
        raise ValueError(f"FGL subject {subject!r} has no Milestone 2 reference primitive.")
    if obj not in NODE_MAP:
        raise ValueError(f"FGL object {obj!r} has no Milestone 2 reference primitive.")

    modifiers = [token for token in tokens if FGL_SYMBOLS[token]["role"] == "modifier"]
    object_scale = 1.25 if "∆" in modifiers else 1.0
    primitives = [_primitive_for(subject), _primitive_for(obj, object_scale)]
    primitive_ids = [primitive["id"] for primitive in primitives]

    relations = []
    for token in tokens:
        relation_kind = RELATION_MAP.get(token)
        if relation_kind:
            relations.append({"kind": relation_kind, "from": primitives[0]["id"], "to": primitives[-1]["id"], "source_symbol": token})

    centers = [primitive["parameters"]["center"] for primitive in primitives]
    center_distance = dist(centers[0], centers[-1])
    constraints = [
        {"name": "subject_present", "passed": True, "detail": f"Subject {subject} maps to {primitives[0]['id']}."},
        {"name": "object_present", "passed": True, "detail": f"Object {obj} maps to {primitives[-1]['id']}."},
        {"name": "unique_primitive_ids", "passed": len(primitive_ids) == len(set(primitive_ids)), "detail": "Primitive identifiers must be unique within a scene."},
        {"name": "positive_radius", "passed": all(primitive["parameters"].get("radius", 0) > 0 for primitive in primitives), "detail": "Reference sphere radii must be positive."},
    ]
    for relation in relations:
        if relation["kind"] == "balance":
            constraints.append({"name": "balance_distance_bound", "passed": center_distance <= 3.0, "detail": f"Reference center distance is {center_distance:.3f}; bound is 3.000."})

    return {
        "schema": SCHEMA,
        "source": {"text": text, "symbols": tokens},
        "semantics": [
            {"symbol": token, "meaning": FGL_SYMBOLS[token]["meaning"], "role": FGL_SYMBOLS[token]["role"]}
            for token in tokens
        ],
        "scene": {"dimension": 3, "composition": "union", "primitives": primitives, "relations": relations},
        "constraints": constraints,
        "provenance": {"compiler": COMPILER, "mapping_policy": MAPPING_POLICY},
    }
