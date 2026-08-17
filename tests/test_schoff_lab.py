import json
from pathlib import Path
import shutil
import subprocess
import sys

import pytest

sys.path.insert(0, str(Path(__file__).parents[1]))
from prototypes.schoff_lab.core import (
    canonical_commitment,
    parse_fgl,
    psse_decode,
    psse_encode,
    tdm_cycle,
    verify_mofp,
)

ROOT = Path(__file__).parents[1]
MOFP = ROOT / "Archive (2026)" / "Mutual Observer File Pair"


def test_psse_round_trip_without_permutation():
    text = "Hi, life!"
    assert psse_decode(psse_encode(text)) == text


def test_psse_round_trip_with_permutation():
    text = "Source transforms life"
    assert psse_decode(psse_encode(text, seed=3), seed=3) == text


def test_psse_rejects_unsupported_unicode():
    with pytest.raises(ValueError, match="Unsupported PSSE character"):
        psse_encode("café")


def test_fgl_documented_example():
    clause = parse_fgl("☉⊗⚘∎")
    assert clause.meanings == ("source", "transformation", "life")
    assert clause.roles == ("subject", "verb", "object")


def test_fgl_rejects_unknown_symbol():
    with pytest.raises(ValueError, match="Unknown FGL symbol"):
        parse_fgl("☉Ω")


def test_tdm_three_phase_order():
    events = tdm_cycle(["new data"], cycles=2)
    assert [e.phase for e in events] == ["ANCHOR", "INSERT", "EXPAND", "ANCHOR", "INSERT", "EXPAND"]


def test_tdm_without_payload_still_has_insertion_window():
    events = tdm_cycle([], cycles=1)
    assert [e.phase for e in events] == ["ANCHOR", "INSERT", "EXPAND"]
    assert events[1].payload["value"] is None


def test_mofp_member_digests_and_explicit_commitment_status():
    result = verify_mofp(MOFP)
    assert result["digest_ok"] is True
    assert result["reciprocal_metadata_ok"] is True
    # The archive's declared commitment currently differs from the protocol's
    # straightforward PDF-then-DOCX derivation; preserve this as a visible fixture.
    assert result["commitment_ok"] is False
    assert result["verified"] is False


def test_mofp_bad_commitment_never_appends_residue(tmp_path):
    package = tmp_path / "pair"
    shutil.copytree(MOFP, package)
    ledger = package / "mutual-observer-residue.ndjson"
    before = ledger.read_text(encoding="utf-8")
    result = verify_mofp(package, append_residue=True)
    assert result["verified"] is False
    assert "residue" not in result
    assert ledger.read_text(encoding="utf-8") == before


def test_mofp_repaired_manifest_appends_residue(tmp_path):
    package = tmp_path / "pair"
    shutil.copytree(MOFP, package)
    manifest_path = package / "mutual-observer-pair.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    members = {m["memberId"]: m for m in manifest["members"]}
    repaired = canonical_commitment(members["mofp-pdf"]["sha256"], members["mofp-docx"]["sha256"])
    manifest["canonicalSharedCommitment"]["value"] = repaired
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    for record_name in ["mofp-pdf.member.json", "mofp-docx.member.json"]:
        record_path = package / record_name
        record = json.loads(record_path.read_text(encoding="utf-8"))
        record["canonicalSharedCommitment"] = repaired
        record_path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    result = verify_mofp(package, append_residue=True)
    assert result["verified"] is True
    residue = result["residue"]
    entries = [json.loads(line) for line in (package / "mutual-observer-residue.ndjson").read_text().splitlines() if line]
    assert entries[-1] == residue


def test_cli_psse_and_fgl_workflows():
    command = [sys.executable, "-m", "prototypes.schoff_lab.cli"]
    encoded = subprocess.run(command + ["psse-encode", "Hi, life!", "--seed", "3"], cwd=ROOT, check=True, capture_output=True, text=True)
    decoded = subprocess.run(command + ["psse-decode", encoded.stdout, "--seed", "3"], cwd=ROOT, check=True, capture_output=True, text=True)
    parsed = subprocess.run(command + ["fgl-parse", "☉⊗⚘∎"], cwd=ROOT, check=True, capture_output=True, text=True)
    assert json.loads(decoded.stdout)["text"] == "Hi, life!"
    assert json.loads(parsed.stdout)["roles"] == ["subject", "verb", "object"]


def test_commitment_is_deterministic():
    a = "a" * 64
    b = "b" * 64
    assert canonical_commitment(a, b) == canonical_commitment(a, b)
    assert canonical_commitment(a, b) != canonical_commitment(b, a)
