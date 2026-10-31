"""Reproduce the fixed Q3.10 EVR0 structural vector from the candidate spec.

This is byte-for-byte fixture reconstruction, not semantic proof validation.
"""
from pathlib import Path
import hashlib
import re

ROOT = Path(__file__).resolve().parents[2]
SPEC = ROOT / "docs/spec/evr0_proof_section_v0.md"
TARGET = Path(__file__).parent
lines = re.findall(r"(?m)^\d{3}: ([0-9a-f ]+)$", SPEC.read_text("utf-8"))
assert len(lines) == 16, f"expected 16 complete dump lines, got {len(lines)}"
section = bytes.fromhex(" ".join(lines))
assert len(section) == 249, len(section)
assert section[:8] == b"EVR0" + (241).to_bytes(4, "little")
assert hashlib.sha256(section).hexdigest() == (
    "cd91d079b4c5846f8bd4e9edf13fc2136d70fe935d808f6d17fdd1f318bd1870"
)
def u16(x: int) -> bytes:
    return x.to_bytes(2, "little")


def name(value: str) -> bytes:
    raw = value.encode("utf-8")
    return u16(len(raw)) + raw


def adt_descriptor(type_name: str, variants: tuple[tuple[str, int], ...]) -> bytes:
    return name(type_name) + u16(len(variants)) + b"".join(
        name(variant) + u16(arity) for variant, arity in variants
    )


# Required canonical built-ins, in strict type-name byte order.
adt0 = b"ADT0" + u16(2)
adt0 += adt_descriptor("Option", (("None", 0), ("Some", 1)))
adt0 += adt_descriptor("Result", (("Ok", 1), ("Err", 1)))
assert len(adt0) == 55
artifact = b"SEMCOD26" + adt0 + section
assert len(artifact) == 312
assert hashlib.sha256(artifact).hexdigest() == (
    "064c8e8482af430f88b74ed61b3930434cacf2bb7742f64429d017fe9279e360"
)
(TARGET / "q3_10_evr0_section.bin").write_bytes(section)
(TARGET / "q3_10_evr0_structural_only.smc").write_bytes(artifact)
print("EVR0 fixture OK; section 249 bytes, canonical ADT0 55 bytes, artifact 312 bytes")
print("section SHA256:", hashlib.sha256(section).hexdigest())
print("artifact SHA256:", hashlib.sha256(artifact).hexdigest())
