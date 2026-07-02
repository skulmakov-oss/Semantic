# PCC Stdlib v0 Admitted Surface Audit

Status: PCC-STDLIB-0 audit

This document records the currently admitted helper/builtin surface used by
Practical Core canonical examples.

It does not define the final standard library design.

## Goal

Separate the current practical helper surface into:

- true public stdlib candidates;
- current builtins;
- provisional helpers;
- capability-bound helpers;
- out-of-scope future helpers.

## Sources

This audit is based on the currently qualified PCC contours:

- Control Flow Core closeout;
- Text Core closeout;
- Collections Core closeout;
- canonical examples;
- smoke tests;
- negative diagnostics harnesses.

## Current Admitted Helper Surface

| Helper | Domain | Current evidence | Current classification | Notes |
| --- | --- | --- | --- | --- |
| `assert` | core/checking | canonical examples, `stdlib_v0_helpers` | core builtin candidate | Self-check mechanism. |
| `print` | output/text | `text_core`, `stdlib_v0_helpers` | capability-bound helper | Must not bypass host/capability policy. |
| `to_text` | text conversion | `text_core`, `text_collections_toolbox`, `stdlib_v0_helpers`, negative fixtures | text helper / builtin candidate | Currently scalar-only. |
| `len` | collections | `collections_core`, `text_collections_toolbox`, `stdlib_v0_helpers` | collection helper | Domain clarity needed. |
| `is_empty` | collections | `collections_core`, `stdlib_v0_helpers` | collection helper | Current Sequence contour. |
| `contains` | collections | `collections_core`, `text_collections_toolbox`, `stdlib_v0_helpers` | collection helper | Current Sequence contour. |
| `push` | collections | `collections_core`, `stdlib_v0_helpers` | collection helper | Current Sequence contour. |
| `prepend` | collections | `collections_core`, `stdlib_v0_helpers` | collection helper | Current Sequence contour. |
| `pop` | collections | `collections_core`, `stdlib_v0_helpers` | collection helper | Empty-pop semantics not finalized. |
| `map_empty` | collections/map | `collections_core`, `text_collections_toolbox`, `stdlib_v0_helpers` | map helper / constructor candidate | Constructor role needs policy. |
| `map_set` | collections/map | `collections_core`, `text_collections_toolbox`, `stdlib_v0_helpers` | map helper | Key/value type checked. |
| `map_get` | collections/map | `collections_core`, `text_collections_toolbox`, `stdlib_v0_helpers` | map helper | Missing-key behavior not finalized. |
| `map_contains` | collections/map | `collections_core`, `stdlib_v0_helpers` | map helper | Current Map query helper. |

## Classification Notes

### Core Builtins

Potential core builtins:

- `assert`

Open question:

- should `assert` be a language builtin, stdlib function, or verifier-visible helper?

### Text Helpers

Current admitted text helpers:

- `to_text`
- `print`

Boundaries:

- `to_text(record)` is out of scope;
- `to_text(collection)` is out of scope;
- formatting API is out of scope;
- implicit `text + scalar` is out of scope.

### Collection Helpers

Current admitted collection helpers:

- `len`
- `is_empty`
- `contains`
- `push`
- `prepend`
- `pop`
- `map_empty`
- `map_set`
- `map_get`
- `map_contains`

Boundaries:

- `remove` is out of scope;
- map iteration is out of scope;
- collection formatting is out of scope;
- missing-key and empty-pop trap semantics are not finalized.

### Capability-Bound Helpers

`print(text)` is admitted as practical surface, but it must remain capability-aware.

This audit does not widen host-facing ABI.

## Canonical Evidence

Canonical examples using helper surface:

- `examples/canonical/text_core/src/main.sm`
- `examples/canonical/collections_core/src/main.sm`
- `examples/canonical/text_collections_toolbox/src/main.sm`
- `examples/canonical/stdlib_v0_helpers/src/main.sm`
- `examples/canonical/loop_control_flow/src/main.sm`
- `examples/canonical/match_control_flow/src/main.sm`
- `examples/canonical/option_result_control_flow/src/main.sm`

## Current Negative Coverage

Text negative diagnostics:

- `tests/pcc_text_negative.rs`

Collections negative diagnostics:

- `tests/pcc_collections_negative.rs`

Control-flow negative diagnostics:

- `tests/pcc_control_flow_negative.rs`

## Current 7hell Coverage

Hell 6 currently runs:

```text
cargo test --test pcc_control_flow_negative
cargo test --test pcc_text_negative
cargo test --test pcc_collections_negative
```

## Out of Scope for Stdlib v0 Audit

This audit does not introduce:

- module system changes;
- import policy changes;
- formatting API;
- debug/logging framework;
- host ABI widening;
- `to_text(record)`;
- `to_text(collection)`;
- map removal;
- map iteration;
- generic collection traits;
- async/concurrency helpers.

## Boundary Questions

- `assert`: builtin or stdlib core?
- `print`: stdlib helper or capability-bound host call?
- `to_text`: builtin conversion or text module helper?
- `len`: overloaded helper or per-domain function?
- `map_empty`: constructor or stdlib function?
- `map_get` missing-key behavior: trap, default, or `Option`?
- stdlib modules: implicit or explicit imports?

## Follow-Up

- PCC-STDLIB-1: stdlib v0 contract
- PCC-STDLIB-2: canonical stdlib helper example
- PCC-STDLIB-3A: stdlib negative diagnostics corpus
- PCC-STDLIB-3B: stdlib negative diagnostics harness
- PCC-STDLIB-4: 7hell stdlib coverage
- PCC-STDLIB-5: stdlib v0 closeout

## Closeout

The completed contour summary lives in
[stdlib_v0_closeout.md](stdlib_v0_closeout.md).
