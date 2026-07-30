# Semantic Release Asset Smoke Matrix

Status: active stable asset validation baseline

This document records the minimum smoke validation required against downloaded release assets, not just locally built binaries.

## Current validated published tag:
- v1.1.1

## Pending prerelease candidate:
- v1.2.0-beta.1
- Local pre-publication asset smoke
- Published downloaded-asset smoke: pending publication

## Validated Assets

- `smc.exe`
- `svm.exe`
- `semantic-language-windows-x64-v1.1.1.zip`

## Current Smoke Matrix

| Scenario | Source | Validation | Expected Signal | Current Result |
| --- | --- | --- | --- | --- |
| Minimal compile-run-disasm | generated `smoke_minimal.sm` | `smc.exe compile`, `svm.exe run`, `svm.exe disasm` | `SEMCODE0`, `RET`, clean run | pass |
| Verified-path `f64` builtin pipeline | generated `smoke_builtin_f64.sm` | `smc.exe compile`, `svm.exe run`, `svm.exe disasm` | `SEMCODE1`, `SUB_F64`, builtin `CALL`, clean run | pass |
| Heavy semantic policy trace | `examples/semantic_policy_overdrive_trace.sm` | `smc.exe compile`, `svm.exe run`, `svm.exe disasm` | `SEMCODE1`, `fusion_consensus_state`, `policy_trace_guard`, `policy_trace_quality`, `policy_trace` | pass |

## Current Validation Notes

- downloaded stable `v1.1.1` assets were revalidated from the published
  release, not only from the local build tree
- release asset hashes are verified against GitHub release metadata before the
  smoke scenarios run
- the stable Windows zip contains exactly:
  - `smc.exe`
  - `svm.exe`
- the zip-contained `smc.exe` and `svm.exe` must hash-match the standalone
  downloaded executables for the same tag

## Reproducible Smoke Command

Preferred command when the release assets are already downloaded locally:

- `pwsh -File scripts/verify_release_assets.ps1 -Tag v1.1.1 -AssetsDirectory <downloaded-assets-dir>`

Fallback command when the local environment can fetch GitHub release assets
directly:

- `pwsh -File scripts/verify_release_assets.ps1 -Tag v1.1.1`

## Smoke Commands

Representative command pattern:

- `smc.exe compile <source>.sm -o <source>.smc`
- `svm.exe run <source>.smc`
- `svm.exe disasm <source>.smc`

## Release Rule

Every published beta or final release should repeat this smoke matrix against
the downloaded assets for that exact tag.

If a release asset fails this smoke matrix, the tag should be treated as packaging-invalid even if `cargo test --workspace` remains green in the repository.
