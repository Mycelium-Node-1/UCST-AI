"""Command-line entry point for the Schoff Constraint Laboratory reference package."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

from .core import parse_fgl, psse_decode, psse_encode, tdm_cycle, to_json, verify_mofp
from .scene_ir import compile_fgl_to_ir
from .sdf import render_ascii, sample_scene, validate_scene


def _read_json_argument(value: str) -> object:
    stripped = value.lstrip()
    if stripped.startswith("[") or stripped.startswith("{"):
        return json.loads(value)
    path = Path(value)
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8"))
    return json.loads(value)


def _write(value: object, output: str | None) -> None:
    payload = to_json(value) + "\n"
    if output:
        Path(output).write_text(payload, encoding="utf-8")
    else:
        sys.stdout.write(payload)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="schoff-lab",
        description="Deterministic reference tools for MOFP, PSSE, FGL, and TDM artifacts.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    verify = sub.add_parser("mofp-verify", help="Verify a Mutual Observer File Pair package.")
    verify.add_argument("package_dir", help="Directory holding mutual-observer-pair.json.")
    verify.add_argument("--append-residue", action="store_true", help="Append a residue only after full verification succeeds.")
    verify.add_argument("--output", help="Write JSON result to this path.")

    encode = sub.add_parser("psse-encode", help="Encode supported text as PSSE symbol objects.")
    encode.add_argument("text")
    encode.add_argument("--seed", type=int)
    encode.add_argument("--output")

    decode = sub.add_parser("psse-decode", help="Decode PSSE JSON supplied inline or from a file.")
    decode.add_argument("symbols", help="JSON array or path to a JSON file.")
    decode.add_argument("--seed", type=int)
    decode.add_argument("--output")

    fgl = sub.add_parser("fgl-parse", help="Tokenize and parse documented FGL symbols.")
    fgl.add_argument("text")
    fgl.add_argument("--output")

    tdm = sub.add_parser("tdm-simulate", help="Generate deterministic three-phase TDM events.")
    tdm.add_argument("--payload", action="append", default=[], help="Payload value; repeat for more than one value.")
    tdm.add_argument("--cycles", type=int, default=1)
    tdm.add_argument("--output")

    compile_ir = sub.add_parser("fgl-compile", help="Compile documented FGL symbols into FGL-IR v1.")
    compile_ir.add_argument("text")
    compile_ir.add_argument("--output")

    sample = sub.add_parser("scene-sample", help="Sample a FGL-IR v1 scene at x, y, z.")
    sample.add_argument("scene", help="Path to an FGL-IR JSON file.")
    sample.add_argument("x", type=float)
    sample.add_argument("y", type=float)
    sample.add_argument("z", type=float)
    sample.add_argument("--output")

    render = sub.add_parser("scene-render", help="Render an ASCII z=0 inspection slice of a FGL-IR v1 scene.")
    render.add_argument("scene", help="Path to an FGL-IR JSON file.")
    render.add_argument("--width", type=int, default=61)
    render.add_argument("--height", type=int, default=25)
    render.add_argument("--span", type=float, default=4.0)
    render.add_argument("--output")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "mofp-verify":
        result = verify_mofp(args.package_dir, append_residue=args.append_residue)
        _write(result, args.output)
        return 0 if result["verified"] else 2
    if args.command == "psse-encode":
        _write(psse_encode(args.text, seed=args.seed), args.output)
        return 0
    if args.command == "psse-decode":
        symbols = _read_json_argument(args.symbols)
        if not isinstance(symbols, list):
            raise ValueError("PSSE symbols must be a JSON array.")
        result = {"text": psse_decode(symbols, seed=args.seed)}
        _write(result, args.output)
        return 0
    if args.command == "fgl-parse":
        _write(parse_fgl(args.text), args.output)
        return 0
    if args.command == "tdm-simulate":
        if args.cycles < 1:
            raise ValueError("--cycles must be at least 1")
        _write([json.loads(to_json(event)) for event in tdm_cycle(args.payload, cycles=args.cycles)], args.output)
        return 0
    if args.command == "fgl-compile":
        _write(compile_fgl_to_ir(args.text), args.output)
        return 0
    if args.command == "scene-sample":
        document = json.loads(Path(args.scene).read_text(encoding="utf-8"))
        scene = document["scene"]
        checks = validate_scene(scene)
        result = sample_scene(scene, (args.x, args.y, args.z))
        _write({"distance": result.distance, "primitive_id": result.primitive_id, "validation": checks}, args.output)
        return 0
    if args.command == "scene-render":
        document = json.loads(Path(args.scene).read_text(encoding="utf-8"))
        art = render_ascii(document["scene"], width=args.width, height=args.height, span=args.span)
        if args.output:
            Path(args.output).write_text(art + "\n", encoding="utf-8")
        else:
            sys.stdout.write(art + "\n")
        return 0
    raise AssertionError(f"Unhandled command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
