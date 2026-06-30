# sm-vm VM-M15 Private Test-Only Result Observation Boundary

## Status

VM-M15 specifies the private test-only observation boundary required for helper-vs-inline result equivalence.

This document does not approve runtime changes, lowering changes, fixture changes, SemCode changes, VM optimization, or helper inlining.

## Context

- VM-M9 showed the helper-boundary scalar-movement signal.
- VM-M11 kept and amplified that signal with G2 helper single-call and call-chain pairs.
- VM-M12 audited helper-boundary source, lowering, and VM shape and identified argument/result/frame/local-slot staging as plausible mechanisms.
- VM-M13 defined the evidence boundary for helper-boundary result equivalence.
- VM-M14 inspected existing VM/test result surfaces and found that no general harness-level result surface is exposed today for ordinary verified execution.
- VM-M15 now specifies the private test-only observation boundary needed before any helper-vs-inline pair-equivalence harness is selected.

## Problem

Fixture-local assertions show that each fixture reaches its own expected state, but they do not provide a shared harness-level comparison that helper and inline variants return the same observable result.

This does not invalidate VM-M12 evidence.

It only limits how strong the pair-equivalence claim can be until a private test-only observation path exists.

## Approved Observation Boundary

- private test-only observation
- no public API widening
- no production VM result API
- no SemCode change
- no verifier change

The observation boundary may inspect test-only execution outcomes such as:

- return value snapshot
- final-state digest
- trap or runtime failure string
- deterministic test-only summary

The boundary must remain private to tests and must not become a release-facing execution surface.

## Explicitly Forbidden Boundary

- public VM result APIs
- production result surfaces
- fixture-local assertions as the only equivalence claim
- broad VM runtime changes
- verifier changes
- SemCode format changes
- lowering rewrites
- helper inlining as a required policy

## Minimal Observable Shape

The minimal acceptable observation shape is a private, deterministic, test-only snapshot that can distinguish:

- successful completion
- trap or runtime failure
- the final observable result state for the checked fixture pair

The shape should be as small as possible while still supporting helper-vs-inline pair comparison at the harness level.

## VM-M16 Implementation Constraints

- Keep the helper private or test-only.
- Do not add a public VM API.
- Do not widen runtime result contracts.
- Do not change production VM behavior.
- Do not require fixture rewrites unless no private observation shape can be derived otherwise.
- Prefer a final-state or return-value snapshot derived from existing execution state over new runtime plumbing.

## Non-Claims

This document does not claim:

- helper and inline fixtures are already harness-equivalent;
- VM performance improved;
- scalar optimization is approved;
- helper inlining is required;
- SemCode format should change;
- VM public APIs should widen;
- fixture changes are approved;
- P5-A/P5-B is reopened.

## Validation

- `git diff --check`
- `cargo fmt --check`
- `git status --short`

If `cargo fmt --check` fails, the blocker must be recorded honestly and unrelated files must not be modified.

