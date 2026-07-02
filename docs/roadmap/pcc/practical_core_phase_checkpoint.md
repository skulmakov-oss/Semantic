# PCC Practical Core Phase Checkpoint

## Status

Checkpoint after closing the first Practical Core contours.

This document summarizes the current state after closing:

- Control Flow Core
- Text Core
- Collections v0
- Stdlib v0

## Closed PCC Contours

| Contour | Status | Closeout |
| --- | ---: | --- |
| Control Flow Core | closed | `docs/roadmap/pcc/control_flow_core_closeout.md` |
| Text Core | closed | `docs/roadmap/pcc/text_core_closeout.md` |
| Collections v0 | closed | `docs/roadmap/pcc/collections_core_closeout.md` |
| Stdlib v0 | closed | `docs/roadmap/pcc/stdlib_v0_closeout.md` |

## Practical-Safe Surface Now Covered

### Control Flow

Qualified surface:

- `if / else`
- `match`
- `return`
- `assert`
- `fn main()`
- `match` over `quad`
- `match` over `Option(T)`
- `match` over `Result(T, E)`
- `while`
- `loop`
- `break;`
- `continue;`
- terminal return paths

### Text

Qualified surface:

- one-line text literals
- empty text literal `""`
- `text == text`
- `text != text`
- bounded `text + text`
- explicit `to_text(...)` for admitted scalar families
- `print(text)`
- `assert`-based text self-checks

### Collections

Qualified surface:

- `Sequence(T)`
- sequence literals where admitted
- indexing with admitted integer index type
- `for` over admitted sequence forms
- `len`
- `is_empty`
- `contains`
- `push`
- `prepend`
- `pop`
- `Map(K, V)`
- `map_empty`
- `map_set`
- `map_get`
- `map_contains`

### Stdlib v0 Helpers

Canonical-safe helper surface:

- `assert`
- `to_text(...)`
- `print(text)`
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

## Canonical Anchors

Current canonical practical anchors:

- `examples/canonical/match_control_flow/src/main.sm`
- `examples/canonical/option_result_control_flow/src/main.sm`
- `examples/canonical/loop_control_flow/src/main.sm`
- `examples/canonical/text_core/src/main.sm`
- `examples/canonical/collections_core/src/main.sm`
- `examples/canonical/text_collections_toolbox/src/main.sm`
- `examples/canonical/stdlib_v0_helpers/src/main.sm`

Supporting anchors:

- `examples/canonical/cli_batch_core/src/main.sm`

## Negative Diagnostics Harnesses

Current PCC negative harnesses:

- `tests/pcc_control_flow_negative.rs`
- `tests/pcc_text_negative.rs`
- `tests/pcc_collections_negative.rs`
- `tests/pcc_stdlib_negative.rs`

These harnesses assert:

- fixture fails;
- broad diagnostic markers are present;
- no `panicked`;
- no exact span lock-in;
- no full diagnostic rendering lock-in.

## 7hell Coverage

Hell 6 now includes:

```text
cargo test --test pcc_control_flow_negative
cargo test --test pcc_text_negative
cargo test --test pcc_collections_negative
cargo test --test pcc_stdlib_negative
```

The 7hell runner remains:

- linear;
- hardcoded;
- fail-fast;
- without `--group` selector;
- without fixture registry.

## Known Current Boundaries

The following are documented known boundaries, not hidden regressions:

### Control Flow

- canonical `fn main()` uses no return type;
- current `match` requires explicit `_` fallback arm;
- expression-valued `match` is out of scope;
- exhaustiveness checking is future work;
- `break expr`, labeled loops, iterators, and advanced loop forms are out of scope.

### Text

- interpolation is out of scope;
- multiline/raw strings are out of scope;
- `text + scalar` is out of scope;
- `to_text(record)` and `to_text(collection)` are out of scope;
- text ordering and locale-aware comparison are out of scope;
- host-facing text ABI widening is out of scope.

### Collections

- `map_remove` is out of scope;
- map iteration is out of scope;
- collection formatting is out of scope;
- generic collection traits are out of scope;
- missing-key, out-of-bounds, and empty-pop trap semantics are not fully finalized.

### Stdlib

- final stdlib module layout is out of scope;
- `core.*`, `text.*`, `seq.*`, `map.*` namespacing is future work;
- formatting API is out of scope;
- debug/logging framework is out of scope;
- host ABI widening is out of scope.

## Validation Status

Recently passed during contour closeouts:

```text
cargo test -q --test pcc_control_flow_negative
cargo test -q --test pcc_text_negative
cargo test -q --test pcc_collections_negative
cargo test -q --test pcc_stdlib_negative
powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1
```

This checkpoint is docs-only and does not claim a fresh test run unless
explicitly executed.

## Current PCC Conclusion

The first Practical Core phase is now structurally qualified.

Semantic can now demonstrate ordinary small-program capability across:

- control flow;
- text;
- collections;
- canonical helper surface;
- positive examples;
- negative diagnostics;
- 7hell qualification.

## Recommended Next Action

Do not immediately open another feature contour.

Recommended next step:

```text
PCC / CTF sync checkpoint
```

Planned document:

- [pcc_ctf_sync_checkpoint.md](pcc_ctf_sync_checkpoint.md)

Purpose:

- verify whether any Practical Core changes affect runtime value registry;
- check trap taxonomy alignment;
- check determinism matrix;
- check verifier expectations;
- check golden trace policy;
- decide whether to enter the next PCC contour or freeze/sync the current one first.
