# Text Observation Contract

Status: PR-E1 scope document
Program: Semantic application-completeness / observation boundary
Implementation target: PR-E1
Scope type: docs-only contract

## Purpose

Define the minimal text surface needed for benchmark-class Semantic programs to
produce human-readable traces.

This document does not implement text operations.

It defines the allowed implementation boundary for PR-E1.

## Required First-Wave Surface

### 1. Text Concatenation

Mandatory operator:

```semantic
let msg: text = "score=" + "10";
```

Admitted only for:

```text
text + text -> text
```

No implicit scalar conversion through `+`.

These must be rejected unless explicit conversion exists:

```semantic
let a: text = "score=" + 10;
let b: text = 10 + "score";
```

### 2. Scalar-To-Text Conversion

PR-E1 may admit explicit conversion helpers.

Recommended first-wave helpers:

| Function | Signature | Notes |
| --- | --- | --- |
| `to_text` | `to_text(text) -> text` | identity |
| `to_text` | `to_text(bool) -> text` | `true` / `false` |
| `to_text` | `to_text(i32) -> text` | decimal |
| `to_text` | `to_text(u32) -> text` | decimal |
| `to_text` | `to_text(quad) -> text` | `N` / `F` / `T` / `S` |

Explicitly deferred:

- `f64`
- `fx`
- records
- ADTs
- sequences
- maps
- closures
- host objects

Reason: these require separate stable rendering contracts.

## User-Facing `to_text` vs Internal `debug_render`

This is the main boundary.

### `to_text`

`to_text` is a language/runtime contract.

It must be:

- deterministic;
- stable enough for user-visible traces;
- suitable for benchmark output;
- independent from diagnostics formatting;
- versioned as public behavior once admitted.

### `debug_render`

`debug_render` is internal/tooling-only.

It may be used for:

- diagnostics;
- VM debug dumps;
- disassembly helpers;
- developer tooling;
- test failure messages.

It must not be treated as user-facing language output.

Rule:

```text
debug_render != to_text
```

No PR may satisfy PR-E1 by exposing existing debug formatting as user-facing
text conversion.

## Non-Goals

PR-E1 must not introduce:

- stdout;
- file I/O;
- interpolation;
- templates;
- format strings;
- locale-aware formatting;
- padding/alignment;
- number base formatting;
- broad `Display`/`ToString` trait system;
- implicit scalar-to-text coercion;
- debug rendering as public output;
- sequence/map/record/ADT rendering.

## Future PR-E1 Implementation Requirements

The future implementation PR must include:

- typecheck support for `text + text`;
- lowering/IR/SemCode support as required;
- verifier support as required;
- VM support;
- positive fixture for concatenation;
- positive fixture for explicit scalar `to_text`;
- negative fixture for implicit `"x" + 1`;
- negative fixture for unsupported `to_text(record)` or equivalent;
- snake benchmark matrix update;
- ledger update.

## PR-E2 Separation

PR-E1 must not emit output.

It only creates text values.

Actual observation/output belongs to PR-E2:

```text
cli/runtime: admit narrow stdout experiment surface
```

Example target after PR-E1 + PR-E2:

```semantic
let line: text = "score=" + to_text(score);
print(line);
```

Where:

- `"score=" + to_text(score)` belongs to PR-E1;
- `print(line)` belongs to PR-E2.

## Acceptance Criteria For This Docs PR

This PR is complete when:

- `docs/roadmap/text_observation_contract.md` exists;
- ledger points PR-E1 to this scope contract;
- PR-E1 remains not implemented;
- no runtime/code files changed;
- `git diff --check` passes;
- CI green.
