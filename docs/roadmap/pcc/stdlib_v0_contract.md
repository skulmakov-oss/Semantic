# PCC Stdlib v0 Contract

## Status

PCC-STDLIB-1 contract draft.

This document defines the canonical-safe helper surface for the current
Practical Core Completion contour.

It is based on:

- `docs/roadmap/pcc/stdlib_v0_audit.md`
- Control Flow Core closeout
- Text Core closeout
- Collections Core closeout
- current canonical examples
- current negative diagnostics harnesses

## Purpose

Stdlib v0 is not a full standard library design.

This contract only defines which currently admitted helpers may be used by PCC
canonical examples as stable-enough practical surface.

## Canonical-Safe Helper Surface

The following helpers are canonical-safe for the current PCC contour:

### Core Helper

- `assert`

### Text Helpers

- `to_text(...)`
- `print(text)`

### Sequence Helpers

- `len`
- `is_empty`
- `contains`
- `push`
- `prepend`
- `pop`

### Map Helpers

- `map_empty`
- `map_set`
- `map_get`
- `map_contains`

## Helper Classification

| Helper | Classification | PCC status | Notes |
| --- | --- | --- | --- |
| `assert` | core helper / builtin candidate | canonical-safe | Used for self-checks. |
| `print(text)` | capability-bound helper | canonical-safe with boundary | Must not bypass host/capability policy. |
| `to_text(...)` | text helper / builtin candidate | canonical-safe for admitted scalars | Records/collections out of scope. |
| `len` | collection helper | canonical-safe | Current collection contour only. |
| `is_empty` | sequence helper | canonical-safe | Current `Sequence(T)` contour. |
| `contains` | sequence helper | canonical-safe | Element type must match. |
| `push` | sequence helper | canonical-safe | Current sequence helper. |
| `prepend` | sequence helper | canonical-safe | Current sequence helper. |
| `pop` | sequence helper | canonical-safe | Empty-pop behavior not finalized. |
| `map_empty` | map helper / constructor candidate | canonical-safe | Current map construction path. |
| `map_set` | map helper | canonical-safe | Key/value type checked. |
| `map_get` | map helper | canonical-safe | Missing-key behavior not finalized. |
| `map_contains` | map helper | canonical-safe | Current map query path. |

## Core Helper Boundary

### `assert`

`assert` is admitted as a canonical self-check mechanism.

It may be used in canonical examples to verify expected behavior.

It must not be treated as:

- a replacement for verifier admission;
- a replacement for type checking;
- a general testing framework;
- a host-side effect escape hatch.

## Text Helper Boundary

### `to_text(...)`

`to_text(...)` is canonical-safe for currently admitted scalar families.

Canonical examples may use it for scalar-to-text conversion where already
supported by the current admitted surface.

Out of scope:

- `to_text(record)`;
- `to_text(collection)`;
- custom formatting;
- implicit scalar-to-text concatenation.

### `print(text)`

`print(text)` is admitted as a practical output helper.

It is capability-bound in principle and must not be used to widen host-facing
ABI.

Canonical examples may use `print(text)` only for already admitted text
values.

Out of scope:

- `print(record)`;
- `print(collection)`;
- raw host output APIs;
- debug/logging framework.

## Collection Helper Boundary

### Sequence Helpers

Canonical examples may use:

- `len`;
- `is_empty`;
- `contains`;
- `push`;
- `prepend`;
- `pop`.

The current contract covers `Sequence(T)` practical usage only.

Out of scope:

- iterator protocol design;
- collection traits;
- advanced generic abstractions;
- collection formatting;
- finalized empty-pop trap semantics.

### Map Helpers

Canonical examples may use:

- `map_empty`;
- `map_set`;
- `map_get`;
- `map_contains`.

Out of scope:

- `map_remove`;
- map iteration;
- ordered maps;
- sets;
- finalized missing-key trap semantics;
- map formatting;
- map serialization.

## Module / Import Boundary

This contract does not define final stdlib module layout.

It does not decide whether helpers will eventually live under:

```text
core.*
text.*
seq.*
map.*
debug.*
```

or remain available through the current admitted helper surface.

Module/import policy is future work.

## Capability Boundary

Helpers that touch observable output, especially `print(text)`, must remain
compatible with the capability/effect boundary.

This contract does not widen:

- host ABI;
- debug output capabilities;
- PROMETHEUS effect policy;
- runtime host-call surface.

## Canonical Examples

Current canonical examples using the helper surface:

- `examples/canonical/text_core/src/main.sm`
- `examples/canonical/collections_core/src/main.sm`
- `examples/canonical/text_collections_toolbox/src/main.sm`
- `examples/canonical/stdlib_v0_helpers/src/main.sm`
- `examples/canonical/loop_control_flow/src/main.sm`
- `examples/canonical/match_control_flow/src/main.sm`
- `examples/canonical/option_result_control_flow/src/main.sm`

## Negative Diagnostics Coverage

Current negative harnesses:

- `tests/pcc_control_flow_negative.rs`
- `tests/pcc_text_negative.rs`
- `tests/pcc_collections_negative.rs`

## 7hell Coverage

Hell 6 currently runs:

```text
cargo test --test pcc_control_flow_negative
cargo test --test pcc_text_negative
cargo test --test pcc_collections_negative
```

## Out of Scope

Stdlib v0 does not include:

- full module system policy;
- import redesign;
- formatting API;
- debug/logging framework;
- host ABI widening;
- `to_text(record)`;
- `to_text(collection)`;
- `text + scalar`;
- map removal;
- map iteration;
- collection formatting;
- collection serialization;
- generic collection traits;
- async/concurrency helpers;
- IO/filesystem/network helpers.

## Open Questions

- Should `assert` remain a builtin or become `core.assert`?
- Should `print(text)` become capability-explicit in syntax or manifest?
- Should `to_text` become `text.to_text` / `core.to_text` / builtin?
- Should `len` be overloaded across text and collections?
- Should map missing-key behavior return `Option(V)` in a later contour?
- Should `pop` return a pair / option / updated sequence only?

## PCC Decision

For the current PCC contour:

```text
Keep the current admitted helper names.
Do not introduce module namespacing yet.
Do not widen helper behavior.
Treat these helpers as canonical-safe practical surface, not final stdlib architecture.
```

## Follow-Up

- PCC-STDLIB-2: canonical stdlib helper example
- PCC-STDLIB-3A: stdlib negative diagnostics corpus
- PCC-STDLIB-3B: stdlib negative diagnostics harness
- PCC-STDLIB-4: 7hell stdlib coverage
- PCC-STDLIB-5: stdlib v0 closeout

## Closeout

The completed contour summary lives in
[stdlib_v0_closeout.md](stdlib_v0_closeout.md).
