# Semantic Compatibility Statement

Status: release-facing compatibility reading

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

This document should be read together with:

- `docs/release_artifact_model.md`
- `docs/roadmap/v1_readiness.md`
- `reports/g1_release_scope_statement.md`

## Purpose

This statement defines compatibility commitments honestly across three layers:

- the published stable line
- the currently qualified limited-release contour
- landed-on-`main` behavior that is not yet promoted

## Core Compatibility Rules

- standard `.smc` execution is verifier-first
- unknown or unsupported SemCode headers must reject explicitly
- VM execution must not silently reinterpret unsupported payloads
- profile and spec changes that alter meaning require explicit review
- public CLI ownership remains centered in `smc-cli`

## Published Stable Compatibility

The published stable line is:

- `v1.1.1`

The new prerelease candidate is:

- `v1.2.0-beta.1`

Compatibility commitments at that layer apply only to what the stable line and
its released assets actually promised.

## Qualified Limited-Release Compatibility

The current Gate 1 verdict qualifies a narrow practical-programming contour.
Compatibility-sensitive reading at that layer applies only to the admitted
qualified contour documented in:

- `reports/g1_release_scope_statement.md`

That qualified contour is narrower than current `main`.

## Landed On `main`, Not Yet Promised

Current `main` contains widened behavior beyond both the stable line and the
current qualified contour.

Those surfaces must be read as:

- landed on `main`, not yet promised

They are not erased, but they also do not inherit compatibility promises
automatically.

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
