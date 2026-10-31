# Q3.10 D6-R4 / B2-1 — Narrow Owner Implementation Decision

Status: **IMPLEMENTATION CANDIDATE; qualification is recorded separately**.
Base `main`: `3dd5ddc982d38aae32b5afa9d0c0ce388827bb56`.
Branch: `feat/q3-10-b2-1-decoder`.
Owner GO: 2026-10-31 "Сделай за один проход" immediately after the
D6-R4/B2-1 plan; does not waive Harness or authorize merge.

This record closes only the *structural* D6-R4 decisions (wire schema,
resource envelope, deterministic rejection and rev27 fail-closed posture).
The normative, limited technical contract is
`docs/spec/evr0_b2_1_structural_v1.md`.

## Boundaries

- Implementation owner `sm-format`: revision-gated decoder and bounded
  canonical table processing, no trusted semantic claims.
- `sm-verify`: reject any rev27 artifact (structural validity cannot create
  `VerifiedSemCode`), deterministic malformed-structure diagnostic.
- `sm-vm`: no rev27 raw execution, no new runtime capability and no issuer.
- `smc-cli`: adapt public structural destructure without turning it into
  a trusted proof reader.
- No Cargo, workflows, Harness, bootstrap, release, tag, merge or unrelated
  functional refactor.
- Explicitly preserve the existing legacy content inventory; do not
  widen named-compatibility allowlists.

## Qualification gate

A PASS requires: golden 249-byte section decoded inside full SemCode;
negative section/malformed counts and legacy downgrade; exact known
revision-27 verifier rejection; VM raw rejection; public API lock and
no_std compatibility; full R3/CI and review of memory/work bound.
Until tests and CI pass, this record is **not** a completion claim.

D6/B2-1 is **not** D6 semantic issuer freeze, B2-2 trusted admission or
B2-3 `VerifiedEvidence`. In particular, any serialized
`authorized_operation` stays inert.

Codex CLI on owner's Windows host was unusable because of Windows
sandbox ACL setup failure and a previously incompatible CLI-model
version; implementation and tests were performed through authorized
direct tooling with independently observed command exit codes.
