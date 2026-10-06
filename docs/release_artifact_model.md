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

## Published Stable Artifact Set (`v1.2.0`)

`v1.2.0` is the published stable release: **Semantic v1.2.0 — Stable
Foundation Release**, published on 2026-10-04 from qualified candidate C1
`89641da8237f4fcefb50cf1958a50e4d4003aea7` (`v1.2.0^{}` dereferences to C1).

- Foundation Oracle Verdict: `ORACLE QUALIFIED WITH EXPLICIT LIMITS`
- Stable Foundation Promotion Decision: `PROMOTE WITH EXPLICIT LIMITS`
- release contour: the Stable Foundation contour in `reports/semantic_stable_foundation_final_verdict.md` (section 6)
- platform: Windows x64 (`x86_64-pc-windows-msvc`) only; Linux and macOS
  release binaries are outside the v1.2.0 contour
- artifact trust: assets are explicitly unsigned; trust is bound to the
  published SHA-256 digests
- explicit limits: R1 (Quad helper predicates `known`/`unknown`/`conflict`
  are rejected), R2 (string literals do not process escapes), R3 (generic
  rejection has no stable diagnostic code), R4 (host paths appear in
  diagnostics), and `std.serde` (G-08) is not applicable / excluded

Current `main` is development beyond C1 (including the later Phase-B and
residual hardening). It is not the v1.2.0 release and is not qualified as any
new release; nothing landed after C1 widens the v1.2.0 promise.

History: the earlier `v1.1.1` tag remains an unresolved stable-tag checkpoint
(no GitHub Release), and `v1.2.0-beta.1` was the prerelease that preceded the
Stable Foundation release.

The published `v1.2.0` assets are (digests from `reports/semantic_stable_foundation_final_verdict.md`, sections 26-27):

| Artifact | Kind | Platform scope | SHA-256 |
| --- | --- | --- | --- |
| `smc.exe` | standalone executable (compiler / CLI) | Windows x64 | `e2e81709b5b050d06b81bf88d26d6472f6228f0ab1d26e82afdea3a25b82c75a` |
| `svm.exe` | standalone executable (SemCode VM / disassembler) | Windows x64 | `9d871a6ff2654d00fe8b5102c9aad426d2ffbd1beeea44e7cbf9fa982e12b4bf` |
| `semantic-language-windows-x64-v1.2.0.zip` | packaged archive of the tool pair | Windows x64 | `97c964c9665343cf0378fd672accca99eae38a7adbb1df0333447781871e4a17` |
| `semantic-language-windows-x64-v1.2.0-candidate.zip` | qualification-candidate archive (same digest) | Windows x64 | `97c964c9665343cf0378fd672accca99eae38a7adbb1df0333447781871e4a17` |

## Historical `v1.1.1` Checkpoint Artifact Set

The following describes the earlier `v1.1.1` stable-tag checkpoint, kept as
history. It was never a published stable release. The downloadable artifacts
prepared for that checkpoint were:

| Artifact | Kind | Platform scope | Role | Validation basis |
| --- | --- | --- | --- | --- |
| `smc.exe` | standalone executable | Windows x64 | packaged Semantic compiler / CLI entrypoint prepared for the `v1.1.1` checkpoint | included in release bundle verification and downloaded-asset smoke validation |
| `svm.exe` | standalone executable | Windows x64 | packaged SemCode VM / disassembler prepared for the `v1.1.1` checkpoint | included in release bundle verification and downloaded-asset smoke validation |
| `semantic-language-windows-x64-v1.1.1.zip` | packaged archive | Windows x64 | convenience bundle containing the `smc.exe` and `svm.exe` pair prepared for the checkpoint | zip contents and hashes are verified against the standalone assets |

## Supported Platform Scope

The published `v1.2.0` downloadable artifact promise is scoped to:

- Windows x64 only (`x86_64-pc-windows-msvc`)

This document does not promise:

- Linux release binaries
- macOS release binaries
- parity for unreleased local builds on other host platforms

Source checkout and local compilation on other hosts may work, but that is not
the same thing as a published binary-artifact promise.

## What A User Downloads

For the published `v1.2.0` release, download the assets listed in the
published stable artifact set above and verify them against their SHA-256
digests.

For the historical `v1.1.1` stable-tag checkpoint (not published stable):

- download `smc.exe` when the standalone compiler / CLI entrypoint is needed
- download `svm.exe` when the standalone VM / disassembler is needed
- download `semantic-language-windows-x64-v1.1.1.zip` when the packaged tool
  pair is preferred

The release-facing meaning of those assets comes from the repository docs named
above, not from the binary filenames alone.

## What These Artifacts Currently Represent

The `v1.1.1` checkpoint artifact set currently represents:

- the unresolved stable-tag checkpoint `v1.1.1`, not a published stable line
- the Windows x64 packaged `smc.exe` / `svm.exe` tool pair prepared for that
  checkpoint
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

That smoke baseline proves the checkpoint asset pair is packaging-valid; it
does not by itself make `v1.1.1` a published stable line.

## Validation Artifacts Versus User-Facing Artifacts

The release process also produces validation artifacts.

Those are not separate user runtime downloads; they are release-governance
evidence for the published assets.

Current release-governance artifacts include:

- release bundle manifests emitted by `scripts/verify_release_bundle.ps1`
- release asset smoke reports emitted by `scripts/verify_release_assets.ps1`
- the checklist and smoke matrix docs that define what those scripts must prove

These artifacts exist to validate the checkpoint assets, not to widen any
stable promise.

## Release Artifact Trust & Signing State (SSF-10)

Release artifacts (`smc.exe`, `svm.exe`, and release zip archives) follow an explicit trust contract established in SSF-10 (#1581):

- **Explicit Signing State**: `unsigned`. The repository deliberately does not sign binaries with code-signing certificates and does not simulate public-key infrastructure. All release smoke validation (`scripts/verify_release_assets.ps1`) and CLI tooling (`smc version`, `smc artifact inspect`) explicitly report `"signing": "unsigned"`.
- **Exact Cryptographic Checksums**: Every release asset publishes an exact SHA-256 digest (computed following the algorithm defined by FIPS PUB 180-4). Smoke verification downloads and verifies assets strictly against these digests.
- **Deterministic Toolchain Identity**: Artifact inspection exposes the exact compiler source fingerprint, package version, enabled features, and SemCode format revision (`SEMCOD22`).

## What Is Not Yet Promised

The following must not be inferred from the published `v1.2.0` artifacts or
the historical `v1.1.1` checkpoint artifacts:

- any promise beyond the `v1.2.0` Stable Foundation contour and its explicit
  limits
- landed-on-`main` widenings that are not explicitly promoted
- broader practical-programming scope beyond the current qualified contour
- broader executable-module authoring beyond the currently qualified slice
- package, schema, UI, or other post-stable waves merely because related code
  exists on current `main`
- Workbench beta packaging or beta smoke evidence as part of the core stable
  artifact set

Current-`main` behavior and the current checkpoint artifacts are related, but
they are not the same promise surface.

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
