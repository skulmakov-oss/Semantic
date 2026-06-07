# Semantic v0.1.0 — Baseline Readiness

## Release Type
- **Type:** Controlled engineering baseline
- **Status:** Not v1 final, not Cargo publish
- **Planned Tag:** `v0.1.0-baseline`

## Summary
This baseline captures the verified execution foundation. It demonstrates that the core architecture invariants hold under automated validation. The system correctly enforces verifier-first admission, deterministic VM execution, and policy-level token-first verified execution. Compatibility byte-shims have been retained for historical coverage, and raw/diagnostic paths have been preserved as unverified diagnostic endpoints. CI and admission gates are fully stabilized around this contract.

## Included in this baseline
- SemCode verification/admission
- Canonical token VM execution path
- Public API snapshot guards
- `no_std` compatibility check for core crates
- Release bundle verification (`scripts/verify_release_bundle.ps1`)
- CLI smoke coverage including the `smc 7hell` diagnostic matrix
- PROMETHEUS capability boundary tests
- Compatibility byte-shim coverage
- Raw/diagnostic VM execution paths

## Not included / Not claimed
- Not v1 final
- Not a complete user-facing SDK
- Not Semantic Studio
- Not Andromeda/PROMETHEUS release
- Not Cargo publish
- Not production security certification
- Not a guarantee that all tests use token execution (byte-shims are selectively retained)

## Verification baseline
This release cut requires the following gates to pass:
- `PRReady` (fmt, clippy, test)
- `Readiness` (smoke, snapshots, release bundle)
- `FullPreflight` (clean tree, matrix execution)
- GitHub CI parity
- `verify_release_bundle.ps1`

## Tag Policy Note
- The target tag for this release is **`v0.1.0-baseline`**.
- The plain `v0.1.0` tag is explicitly not used for this cut because it already exists historically in the repository (created 2026-02-14).
- No tag is created by the PR introducing these notes.

## Explicit No-Go List
For this specific baseline cut PR:
- No version bump in this PR
- No tag created in this PR
- No GitHub Release creation in this PR
- No code or test behavior changes
- No CI workflow modifications
- No compatibility shim removal
