# Semantic Compatibility Statement

Status: release-facing compatibility reading

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

This document should be read together with:

- `docs/release_artifact_model.md`
- `docs/roadmap/v1_readiness.md`
- `reports/g1_release_scope_statement.md`

## Purpose

This statement defines compatibility posture honestly across three layers:

- a future published stable line (none currently evidenced)
- the currently qualified limited-release contour
- landed-on-`main` behavior that is not yet promoted

## Core Compatibility Rules

- standard `.smc` execution is verifier-first
- unknown or unsupported SemCode headers must reject explicitly
- VM execution must not silently reinterpret unsupported payloads
- profile and spec changes that alter meaning require explicit review
- public CLI ownership remains centered in `smc-cli`

## Published Stable Compatibility

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

Published-stable compatibility commitments apply only to what `v1.2.0` and its
released assets actually promise, within the limits above.

## Qualified Limited-Release Compatibility

The current Gate 1 verdict qualifies a narrow practical-programming contour.
Compatibility-sensitive reading at that layer applies only to the admitted
qualified contour documented in:

- `reports/g1_release_scope_statement.md`

That qualified contour is narrower than current `main`.

## Landed On `main`, Not Yet Promised

Current `main` contains widened behavior beyond the current qualified contour.

Those surfaces must be read as:

- landed on `main`, not yet promised

They are not erased, but they also do not inherit compatibility promises
automatically.

### SemCode `SEMCOD22` ADT descriptor migration (SSF-09 D2-2)

Recorded under `docs/spec/semcode.md` `## Backward Compatibility Rule`:

- SemCode `SEMCOD22` (revision `23`) introduces the mandatory, canonical
  `ADT0` descriptor section; the current compiler emits `SEMCOD22` for every
  artifact.
- This is an intentional verifier interpretation change: a `SEMCODE0` to
  `SEMCOD21` artifact that contains `MAKE_ADT`, `ADT_TAG`, or `ADT_GET` is no
  longer verifier-admissible (`AdtRequiresDescriptorHeader`), although it was
  admitted before.
- No descriptor authority is inferred, reconstructed, or synthesized for such
  an artifact. The supported migration is recompilation with the current
  toolchain.
- `SEMCODE0` to `SEMCOD21` artifacts remain structurally decodable, and those
  without these ADT opcodes keep their existing compatibility.

This statement does not claim that every `SEMCODE0` to `SEMCOD21` binary keeps
its previous verifier outcome.

### SSF-10 Compatibility, Migration, and Artifact Trust Contract (#1581)

Established by milestone SSF-10 and documented in `docs/architecture/artifact_identity_and_trust.md`:

- **Compatibility Dimensions**: Explicitly distinguishes between `Compatible`, `Deprecated`, `Incompatible`, and `Unsupported` states across canonical dimensions: source (`0.1.0`), manifest (`1`), diagnostics (`semantic.diagnostics`), stdlib (`semantic-stdlib-v1`), SemCode format (`SEMCOD22`), verifier gate (`verifier-canonical-v1`), and runtime model (`deterministic-v1`).
- **Canonical Artifact Identity**: Deterministic, zero-dependency SHA-256 (`sha256:<64-hex>`) computed over the raw `.smc` byte payload, transitively binding header, capabilities, signatures, instruction stream, ADT descriptors, and debug symbols.
- **Immutable Verifier Binding**: `VerifiedSemCode` and `VerifiedEntrySemCode` record the exact artifact SHA-256 digest. A verification result for artifact A cannot validate or admit artifact B.
- **Deterministic Digest-Based Staleness Detection**: Detects source modification, mtime spoofing, and content mismatches via cryptographic digest comparison against companion provenance, without relying on filesystem timestamps alone and without silent regeneration or silent acceptance.
- **Non-Destructive Migration Dry-Run**: `smc migrate <check|preview> <path> [--json] [--dry-run]` guarantees zero filesystem mutations while reporting intended changes and deprecations.
- **Explicit Release Trust**: Binaries and distribution archives are explicitly declared as `"unsigned"` (no simulated certificates or fake PKI). Asset integrity is grounded in SHA-256 digests following the algorithm defined by FIPS PUB 180-4.

## Explicit Non-Commitments

The repository does not currently claim final compatibility guarantees for:

- broader executable-module authoring beyond the admitted bare/selected helper
  slice
- full CLI application authoring with admitted argv/stdout/file IO
- UI beyond any later explicitly qualified contour
- broader generalized iterable dispatch
- any landed-on-`main` widening that has not yet been explicitly promoted by a
  later release or qualification decision

## Release Honesty Rule

Compatibility wording must stay aligned with:

- `docs/spec/`
- `docs/roadmap/v1_readiness.md`
- `reports/g1_release_scope_statement.md`

If a surface is only landed on current `main`, it must be described that way
rather than implied as stable or qualified.
