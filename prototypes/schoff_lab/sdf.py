"""Small, deterministic CPU SDF evaluator for FGL-IR v1 scenes."""
from __future__ import annotations

from dataclasses import dataclass
from math import sqrt
from typing import Any, Iterable

Vector3 = tuple[float, float, float]


@dataclass(frozen=True)
class SDFSample:
    distance: float
    primitive_id: str | None


def _vector3(value: Iterable[float], name: str) -> Vector3:
    values = tuple(float(v) for v in value)
    if len(values) != 3:
        raise ValueError(f"{name} must contain exactly three numeric values.")
    return values  # type: ignore[return-value]


def _sub(a: Vector3, b: Vector3) -> Vector3:
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def _length(v: Vector3) -> float:
    return sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])


def _normalize(v: Vector3) -> Vector3:
    length = _length(v)
    if length == 0:
        raise ValueError("Plane normal must not be zero.")
    return (v[0] / length, v[1] / length, v[2] / length)


def sphere_distance(point: Vector3, parameters: dict[str, Any]) -> float:
    center = _vector3(parameters["center"], "sphere center")
    radius = float(parameters["radius"])
    if radius <= 0:
        raise ValueError("Sphere radius must be positive.")
    return _length(_sub(point, center)) - radius


def box_distance(point: Vector3, parameters: dict[str, Any]) -> float:
    center = _vector3(parameters["center"], "box center")
    extents = _vector3(parameters["half_extents"], "box half_extents")
    if min(extents) <= 0:
        raise ValueError("Box half_extents must be positive.")
    q = tuple(abs(v) - e for v, e in zip(_sub(point, center), extents))
    outside = _length((max(q[0], 0.0), max(q[1], 0.0), max(q[2], 0.0)))
    inside = min(max(q[0], q[1], q[2]), 0.0)
    return outside + inside


def plane_distance(point: Vector3, parameters: dict[str, Any]) -> float:
    normal = _normalize(_vector3(parameters["normal"], "plane normal"))
    offset = float(parameters.get("offset", 0.0))
    return normal[0] * point[0] + normal[1] * point[1] + normal[2] * point[2] + offset


def primitive_distance(point: Iterable[float], primitive: dict[str, Any]) -> float:
    p = _vector3(point, "point")
    kind = primitive.get("kind")
    parameters = primitive.get("parameters")
    if not isinstance(parameters, dict):
        raise ValueError("Primitive parameters must be an object.")
    if kind == "sphere":
        return sphere_distance(p, parameters)
    if kind == "box":
        return box_distance(p, parameters)
    if kind == "plane":
        return plane_distance(p, parameters)
    raise ValueError(f"Unsupported SDF primitive kind: {kind!r}")


def sample_scene(scene: dict[str, Any], point: Iterable[float]) -> SDFSample:
    if scene.get("dimension") != 3 or scene.get("composition") != "union":
        raise ValueError("Milestone 2 supports only dimension-3 union scenes.")
    primitives = scene.get("primitives")
    if not isinstance(primitives, list) or not primitives:
        raise ValueError("Scene must contain at least one primitive.")
    best: SDFSample | None = None
    for primitive in primitives:
        distance = primitive_distance(point, primitive)
        sample = SDFSample(distance, primitive.get("id"))
        if best is None or sample.distance < best.distance:
            best = sample
    assert best is not None
    return best


def validate_scene(scene: dict[str, Any]) -> list[dict[str, Any]]:
    """Return transparent structural checks for an IR scene."""
    results: list[dict[str, Any]] = []
    primitives = scene.get("primitives", [])
    ids = [primitive.get("id") for primitive in primitives if isinstance(primitive, dict)]
    results.append({"name": "non_empty_scene", "passed": bool(primitives), "detail": "Scene contains at least one primitive."})
    results.append({"name": "unique_primitive_ids", "passed": len(ids) == len(set(ids)), "detail": "Primitive identifiers are unique."})
    for primitive in primitives:
        try:
            primitive_distance((0.0, 0.0, 0.0), primitive)
        except (KeyError, TypeError, ValueError) as error:
            results.append({"name": f"valid_{primitive.get('id', 'unknown')}", "passed": False, "detail": str(error)})
        else:
            results.append({"name": f"valid_{primitive.get('id', 'unknown')}", "passed": True, "detail": "Primitive parameters are valid."})
    return results


def render_ascii(scene: dict[str, Any], width: int = 61, height: int = 25, span: float = 4.0) -> str:
    """Render a z=0 inspection slice; `#` is interior and `.` is exterior."""
    if width < 3 or height < 3 or span <= 0:
        raise ValueError("width and height must be at least 3 and span must be positive.")
    lines: list[str] = []
    for row in range(height):
        y = span - (2 * span * row / (height - 1))
        chars: list[str] = []
        for column in range(width):
            x = -span + (2 * span * column / (width - 1))
            distance = sample_scene(scene, (x, y, 0.0)).distance
            if distance < -0.05:
                chars.append("#")
            elif distance <= 0.05:
                chars.append("+")
            else:
                chars.append(".")
        lines.append("".join(chars))
    return "\n".join(lines)
