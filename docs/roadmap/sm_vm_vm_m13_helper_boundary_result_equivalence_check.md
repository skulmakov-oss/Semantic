# sm-vm VM-M13 Helper-Boundary Result-Equivalence Check Audit

## Status

VM-M13 specifies the evidence boundary for helper-boundary fixture-pair result equivalence.

This document does not approve runtime changes, lowering changes, fixture changes, SemCode changes, VM optimization, or helper inlining.

## Context

- VM-M9 first exposed the helper-boundary scalar-movement signal.
- VM-M11 G2 helper single-call and helper call-chain pairs kept and amplified that signal.
- VM-M12 audited helper-boundary source, lowering, and VM shape and identified argument/result/frame/local-slot staging as plausible mechanisms.
- VM-M13 now defines how result equivalence should be proven before any implementation candidate is selected.

## Problem

Fixture-local assertions show that each fixture reaches its own expected state, but they do not provide a shared harness-level comparison that helper and inline variants return the same observable result.

This does not invalidate VM-M12 evidence.

It only limits how strong the pair-equivalence claim can be.

## Current Evidence Boundary

- Fixture-local assertions are valid smoke and equivalence guards.
- Profile deltas are valid scalar-movement observations.
- Helper-vs-inline result equality is not yet proven by a shared Rust-level comparison.
- No result-inspection API should be introduced casually.
- No VM behavior change is approved by this audit.

## Candidate Equivalence Mechanisms

| Option | What it proves | What it does not prove | Implementation risk | Compatibility risk | Requires VM API changes | Changes fixtures | Acceptable before optimization selection |
|---|---|---|---|---|---|---|---|
| A. Shared returned-value harness comparison | The helper and inline fixtures return the same observable result through an existing test harness path. | It does not prove internal lowering is identical or that all intermediate state matches. | Low if an existing helper already exposes the return value; medium if new plumbing is needed. | Low to medium. | Maybe, depending on existing helpers. | No, if existing fixtures already expose a returned value; otherwise maybe. | Yes, this is the cleanest target if available without widening public APIs. |
| B. Shared final-state digest comparison | The two variants converge to the same final-state summary or digest. | It does not prove step-by-step execution equivalence or internal call/return staging identity. | Medium. | Medium. | Usually no public API change if a test-only digest path already exists. | Possibly, if fixtures must emit or expose digestable state. | Yes, if the digest can be derived from current test surfaces. |
| C. Shared event/trace digest comparison | The two variants produce the same high-level event or trace digest. | It does not prove exact register or local-slot equality, only summary-level equivalence. | Medium to high. | Medium to high. | Possibly, if trace capture is already available in tests. | Usually no fixture change if trace capture already exists. | Conditional. Useful only if existing harness support is already present. |
| D. Existing fixture-local assertions only | Each fixture meets its own expected end state. | It does not prove helper and inline variants are equivalent to each other at harness level. | Low. | Low. | No. | No. | No. This is acceptable as a smoke guard, but too weak for a strong pair-equivalence claim. |
| E. Golden output files | Both variants produce the same captured output artifact. | It does not prove internal state equality unless the output fully encodes the state. | Medium. | Medium to high. | Usually no, but it may require dedicated output capture plumbing. | Possibly. | Conditional. Acceptable only if it can be derived from current test output without adding public APIs. |

## Recommended VM-M14 Path

Preferred recommendation:

VM-M14 should first audit whether existing VM test helpers can compare returned values or execution summaries without adding public result-inspection APIs.

Fallback:

If no existing helper path is sufficient, VM-M14 should specify a test-only result observation boundary before any production API is introduced.

## Non-claims

This document does not claim:

- helper and inline fixtures are fully equivalent at harness level;
- VM performance improved;
- scalar optimization is approved;
- helper inlining is required;
- SemCode format should change;
- VM public APIs should widen;
- fixture changes are approved;
- P5-A/P5-B is reopened.

## Validation

- `git diff --check` should pass.
- `cargo fmt --check` should pass if no unrelated local formatting drift is present.
- `git status --short` should still show the unrelated untracked local artifacts that are outside VM-M13 scope.

If `cargo fmt --check` fails, the blocker must be recorded honestly and no unrelated files should be modified.
