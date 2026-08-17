"""Small, deterministic research prototype for the formalizable Schoff artifacts.

This module intentionally implements only the documented, testable subset. It does
not claim to implement a literal 5D reality engine or cryptographic security.
"""
from __future__ import annotations

from dataclasses import dataclass, asdict
from hashlib import sha256
import json
from pathlib import Path
import re
from typing import Any, Iterable


# ----------------------------- MOFP ---------------------------------------

def sha256_file(path: str | Path) -> str:
    return sha256(Path(path).read_bytes()).hexdigest()


def canonical_commitment(pdf_digest: str, docx_digest: str) -> str:
    """Canonical PDF-then-DOCX commitment, with an explicit newline separator."""
    payload = f"{pdf_digest}\n{docx_digest}".encode("ascii")
    return sha256(payload).hexdigest()


def verify_mofp(package_dir: str | Path, append_residue: bool = False) -> dict[str, Any]:
    """Verify member hashes and reciprocal metadata in a MOFP package.

    The function reports a manifest commitment mismatch instead of silently accepting
    ambiguous canonicalization. A residue is appended only when all structural checks
    pass and the caller explicitly requests it.
    """
    root = Path(package_dir)
    manifest = json.loads((root / "mutual-observer-pair.json").read_text())
    members = {m["memberId"]: m for m in manifest["members"]}
    actual = {m["memberId"]: sha256_file(root / m["sourceFile"]) for m in manifest["members"]}
    declared = {mid: members[mid]["sha256"] for mid in members}
    digest_ok = actual == declared

    reciprocal_ok = True
    for mid, record in members.items():
        member_record = json.loads((root / record["memberRecord"]).read_text())
        partner = member_record["partner"]
        reciprocal_ok &= partner["memberId"] in members
        reciprocal_ok &= partner["sourceSha256"] == declared[partner["memberId"]]
        reciprocal_ok &= member_record["canonicalSharedCommitment"] == manifest["canonicalSharedCommitment"]["value"]

    computed = canonical_commitment(declared["mofp-pdf"], declared["mofp-docx"])
    commitment_ok = computed == manifest["canonicalSharedCommitment"]["value"]
    result = {
        "pair_id": manifest["pairId"],
        "digest_ok": digest_ok,
        "reciprocal_metadata_ok": reciprocal_ok,
        "commitment_ok": commitment_ok,
        "declared_commitment": manifest["canonicalSharedCommitment"]["value"],
        "computed_commitment": computed,
        "verified": digest_ok and reciprocal_ok and commitment_ok,
    }
    if append_residue and result["verified"]:
        residue_path = root / manifest["ledger"]
        prior = residue_path.read_text(encoding="utf-8") if residue_path.exists() else ""
        seq = sum(1 for line in prior.splitlines() if line.strip()) + 1
        residue = {
            "sequence": seq,
            "pair_id": manifest["pairId"],
            "joint_hash": computed,
            "event": "mutually_verified_access",
        }
        with residue_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(residue, sort_keys=True) + "\n")
        result["residue"] = residue
    return result


# ----------------------------- PSSE ----------------------------------------

PUNCTUATION = {".": ("circle", 100), ",": ("square", 101), "?": ("triangle_outline", 102), "!": ("star", 103), "'": ("single_line", 104), '"': ("double_line", 105)}
PUNCTUATION_REVERSE = {code: mark for mark, (_, code) in PUNCTUATION.items()}


def _permute(n: int, i: int, seed: int) -> int:
    value = (n + ((i * seed) % 26)) % 26
    return value or 26


def _unpermute(n: int, i: int, seed: int) -> int:
    value = (n - ((i * seed) % 26)) % 26
    return value or 26


def psse_encode(text: str, seed: int | None = None) -> list[dict[str, Any]]:
    """Encode supported text into the archive's PSSE symbol-object shape."""
    out: list[dict[str, Any]] = []
    symbol_index = 0
    for i, ch in enumerate(text, start=1):
        if ch.isalpha() and ch.isascii():
            symbol_index += 1
            n = ord(ch.upper()) - ord("A") + 1
            sides = _permute(n, symbol_index, seed) if seed is not None else n
            next_is_boundary = i == len(text) or text[i].isspace()
            out.append({"type": "polygon", "sides": sides, "orientation": int(ch.isupper()), "boundary": int(next_is_boundary)})
        elif ch in PUNCTUATION:
            symbol_index += 1
            name, code = PUNCTUATION[ch]
            next_is_boundary = i < len(text) and text[i].isspace()
            out.append({"type": "punctuation", "symbol": name, "code": code, "boundary": int(next_is_boundary)})
        elif ch.isspace():
            continue
        else:
            raise ValueError(f"Unsupported PSSE character: {ch!r}")
    return out


def psse_decode(symbols: Iterable[dict[str, Any]], seed: int | None = None) -> str:
    out: list[str] = []
    for i, obj in enumerate(symbols, start=1):
        if obj.get("type", "polygon") == "punctuation" or "code" in obj:
            out.append(PUNCTUATION_REVERSE[obj["code"]])
            if obj.get("boundary"):
                out.append(" ")
            continue
        n = obj["sides"]
        n = _unpermute(n, i, seed) if seed is not None else n
        if not 1 <= n <= 26:
            raise ValueError(f"Invalid polygon side count: {n}")
        out.append(chr(ord("A") + n - 1) if obj.get("orientation") else chr(ord("a") + n - 1))
        if obj.get("boundary"):
            out.append(" ")
    return "".join(out).rstrip()


# ------------------------------ FGL ----------------------------------------

FGL_SYMBOLS = {
    "☉": {"meaning": "source", "role": "subject"},
    "⊗": {"meaning": "transformation", "role": "verb"},
    "⚘": {"meaning": "life", "role": "object"},
    "∆": {"meaning": "change", "role": "modifier"},
    "⟁": {"meaning": "balance", "role": "verb"},
    "⟡": {"meaning": "light", "role": "object"},
    "✶": {"meaning": "creation", "role": "verb"},
    "Ϟ": {"meaning": "energy", "role": "subject"},
}

@dataclass(frozen=True)
class FGLClause:
    symbols: tuple[str, ...]
    meanings: tuple[str, ...]
    roles: tuple[str, ...]


def tokenize_fgl(text: str) -> list[str]:
    known = sorted(FGL_SYMBOLS, key=len, reverse=True)
    tokens: list[str] = []
    i = 0
    while i < len(text):
        if text[i].isspace() or text[i] in "·—∎":
            i += 1
            continue
        match = next((s for s in known if text.startswith(s, i)), None)
        if match is None:
            raise ValueError(f"Unknown FGL symbol at offset {i}: {text[i]!r}")
        tokens.append(match)
        i += len(match)
    return tokens


def parse_fgl(text: str) -> FGLClause:
    tokens = tokenize_fgl(text)
    if not tokens:
        raise ValueError("FGL clause cannot be empty")
    roles = tuple(FGL_SYMBOLS[t]["role"] for t in tokens)
    meanings = tuple(FGL_SYMBOLS[t]["meaning"] for t in tokens)
    return FGLClause(tuple(tokens), meanings, roles)


# ------------------------------ TDM ----------------------------------------

@dataclass(frozen=True)
class TDMEvent:
    tick: int
    phase: str
    payload: Any


def tdm_cycle(payloads: Iterable[Any], cycles: int = 1) -> list[TDMEvent]:
    """Deterministic software simulation of the documented three-phase heartbeat."""
    events: list[TDMEvent] = []
    tick = 0
    values = list(payloads)
    for cycle in range(cycles):
        events.append(TDMEvent(tick, "ANCHOR", {"cycle": cycle, "regime": "C-"})); tick += 1
        value = values[cycle % len(values)] if values else None
        events.append(TDMEvent(tick, "INSERT", {"cycle": cycle, "value": value})); tick += 1
        events.append(TDMEvent(tick, "EXPAND", {"cycle": cycle, "regime": "C+"})); tick += 1
    return events


def to_json(value: Any) -> str:
    if hasattr(value, "__dataclass_fields__"):
        value = asdict(value)
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True)
