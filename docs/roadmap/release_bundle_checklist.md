# Semantic v1 Release Bundle Checklist

Status: active stable release baseline

Use this checklist before assembling or publishing a release-facing stable or
prerelease `v1` bundle.

## Local-first Release Readiness Policy

GitHub CI is not the authoritative release gate for the current stage.

Release/readiness admission is validated locally through:

- `pwsh scripts\admission_guard.ps1 -PRReady`
- `pwsh scripts\admission_guard.ps1 -Readiness`
- `pwsh scripts\admission_guard.ps1 -FullPreflight` when required or practical
- `pwsh scripts\verify_release_bundle.ps1 ...`
- `pwsh scripts\verify_release_assets.ps1 ...`

GitHub CI may remain a supplementary signal when available, but it must not be
described as required for the current local release-readiness baseline.

A human release decision is still required before tagging or publishing.

## Required Documentation Bundle

Verify the bundle includes:

- `docs/release_artifact_model.md`
- `docs/architecture/`
- `docs/spec/`
- `docs/roadmap/v1_readiness.md`
- `docs/roadmap/runtime_validation_policy.md`
- `docs/roadmap/compatibility_statement.md`
- `docs/roadmap/release_asset_smoke_matrix.md`
- `docs/roadmap/stable_release_policy.md`
- `docs/roadmap/private_custody_mode.md`, if the repository is private or if release preparation follows a private custody period
- published asset notes for `smc.exe`, `svm.exe`, and the Windows zip when a GitHub release is cut

Reproducible check command:

- `pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath <path>`

## Required Contract Surfaces

Verify the release documents the current state of:

- SemCode header family and verifier rule
- ParserProfile contract and hash semantics
- VM quota and verified-only execution rule
- PROMETHEUS ABI, capability, and gate boundaries
- semantic runtime orchestration, state, rules, and audit owner split

## Required Test Gates

Verify these are green locally before the bundle is considered releasable:

- `pwsh scripts\admission_guard.ps1 -PRReady`
- `pwsh scripts\admission_guard.ps1 -Readiness`
- `pwsh scripts\admission_guard.ps1 -FullPreflight` when required or practical
- `pwsh scripts\verify_release_bundle.ps1 -ManifestPath <path>`
- `pwsh scripts\verify_release_assets.ps1 -Tag <tag> -AssetsDirectory <downloaded-assets-dir>`
- `cargo test --workspace`
- `cargo test --test public_api_contracts`
- `cargo test --test golden_semcode`
- `cargo test --test prometheus_runtime_matrix`
- `cargo test --test prometheus_runtime_goldens`
- `cargo test --test prometheus_runtime_negative_goldens`
- `cargo test --test prometheus_runtime_compat_matrix`

The older CI-job names (`boundary-enforcement`, `public-api-guard`,
`runtime-release-gates`, `release-bundle-process`) are historical references or
supplementary signals only; they are not the authoritative gate for the current
local release-readiness baseline.

## Required Artifact Notes

Verify the release notes include:

- currently stabilized surfaces
- known limits that remain explicit non-commitments for the current narrow `v1`
- explicit snapshot regeneration rule
- compatibility-sensitive contract families
- which packaged assets were published for the current tag
- whether the release was prepared from a private custody period, without exposing private-only implementation details

## Required Asset Smoke

Verify published assets are checked against at least:

- one minimal compile-run-disasm source
- one verified-path `f64` builtin case
- one representative semantic policy example from `examples/`

Reproducible command:

- `pwsh -File scripts/verify_release_assets.ps1 -Tag <tag> -AssetsDirectory <downloaded-assets-dir>`

## Custody Transition Check

Before publishing anything derived from a private repository state, verify:

- public-facing documents still distinguish implemented, stable, planned, and experimental behavior;
- private-only architecture notes are not accidentally promoted into public claims;
- known limits remain visible in release-facing notes;
- repository visibility and access-control changes are treated as operational facts, not technical maturity evidence;
- the final release branch or tag points only to content intended for that release surface.

## Blocking Rule

Do not mark the bundle release-ready if:

- root inventory docs disagree with `tests/legacy_guards.rs`
- any known limit was silently dropped from the docs
- compatibility-sensitive tests were not run
- runtime snapshots were regenerated without review
- readiness and compatibility documents disagree with actual repository behavior
- private custody mode is used to bypass PR, CI, or release evidence discipline
