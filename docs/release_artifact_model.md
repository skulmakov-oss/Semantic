# Semantic Release Artifact Model

Status: release-facing artifact authority for published stable assets

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Read this document together with:

- `docs/roadmap/v1_readiness.md`
- `docs/roadmap/stable_release_policy.md`
- `docs/roadmap/compatibility_statement.md`
- `docs/roadmap/release_bundle_checklist.md`
- `docs/roadmap/release_asset_smoke_matrix.md`

## Purpose

This document answers four release-facing questions explicitly:

- what do I download
- what platform is actually supported by the published artifacts
- what is promised by those artifacts
- what is not yet promised even if it exists on current `main`

## Current Published Stable Artifact Set

The current published stable line is:

- `v1.1.1`

The new prerelease candidate is:

- `v1.2.0-beta.1`

The currently published downloadable stable artifacts are:

| Artifact | Kind | Platform scope | Role | Validation basis |
| --- | --- | --- | --- | --- |
| `smc.exe` | standalone executable | Windows x64 | packaged Semantic compiler / CLI entrypoint for the published stable line | included in release bundle verification and downloaded-asset smoke validation |
| `svm.exe` | standalone executable | Windows x64 | packaged SemCode VM / disassembler for the published stable line | included in release bundle verification and downloaded-asset smoke validation |
| `semantic-language-windows-x64-v1.1.1.zip` | packaged archive | Windows x64 | convenience bundle containing the published `smc.exe` and `svm.exe` pair | zip contents and hashes are verified against the standalone assets |

## Supported Platform Scope

The published downloadable artifact promise is currently:

- Windows x64 only

This document does not promise:

- Linux release binaries
- macOS release binaries
- parity for unreleased local builds on other host platforms

Source checkout and local compilation on other hosts may work, but that is not
the same thing as a published binary-artifact promise.

## What A User Downloads

For the published stable line today:

- download `smc.exe` when the standalone compiler / CLI entrypoint is needed
- download `svm.exe` when the standalone VM / disassembler is needed
- download `semantic-language-windows-x64-v1.1.1.zip` when the packaged tool
  pair is preferred

The release-facing meaning of those assets comes from the repository docs named
above, not from the binary filenames alone.

## What These Artifacts Currently Promise

The published stable artifact set currently promises:

- the exact tagged stable line `v1.1.1`
- the Windows x64 packaged `smc.exe` / `svm.exe` tool pair
- release bundle verification through `scripts/verify_release_bundle.ps1`
- downloaded-asset smoke validation through `scripts/verify_release_assets.ps1`
- the current release-facing reading in:
  - `docs/roadmap/v1_readiness.md`
  - `docs/roadmap/stable_release_policy.md`
  - `docs/roadmap/compatibility_statement.md`
  - `docs/roadmap/release_asset_smoke_matrix.md`

The currently validated downloaded-asset smoke path is explicitly grounded in:

- `smc.exe compile <source>.sm -o <source>.smc`
- `svm.exe run <source>.smc`
- `svm.exe disasm <source>.smc`

That smoke baseline proves the published asset pair is packaging-valid for the
current stable line.

## Validation Artifacts Versus User-Facing Artifacts

The release process also produces validation artifacts.

Those are not separate user runtime downloads; they are release-governance
evidence for the published assets.

Current release-governance artifacts include:

- release bundle manifests emitted by `scripts/verify_release_bundle.ps1`
- release asset smoke reports emitted by `scripts/verify_release_assets.ps1`
- the checklist and smoke matrix docs that define what those scripts must prove

These artifacts exist to validate the published assets, not to widen the
stable promise.

## What Is Not Yet Promised

The following must not be inferred from the current published stable artifacts:

- landed-on-`main` widenings that are not explicitly promoted
- broader practical-programming scope beyond the current qualified contour
- broader executable-module authoring beyond the currently qualified slice
- package, schema, UI, or other post-stable waves merely because related code
  exists on current `main`
- Workbench beta packaging or beta smoke evidence as part of the core stable
  artifact set

Current-`main` behavior and current stable artifacts are related, but they are
not the same promise surface.

## Authority And Drift Rule

This document stays truthful only if it remains aligned with:

- `docs/roadmap/v1_readiness.md`
- `docs/roadmap/stable_release_policy.md`
- `docs/roadmap/compatibility_statement.md`
- `docs/roadmap/release_bundle_checklist.md`
- `docs/roadmap/release_asset_smoke_matrix.md`
- `scripts/verify_release_bundle.ps1`
- `scripts/verify_release_assets.ps1`

If any of those drift from the real published asset set or supported platform
scope, the release-facing reading is no longer honest and must be corrected
before the next release-facing decision.
