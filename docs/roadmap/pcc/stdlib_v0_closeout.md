# PCC Stdlib v0 Closeout

## 0. Status

Status:
  COMPLETE / CURRENT PCC STDLIB V0 CONTOUR CLOSED

This document does **not** claim full stdlib design.
This document does **not** claim module namespacing readiness.
This document does **not** claim import-policy finalization.
This document does **not** claim host ABI widening.

## 1. Closed Scope

The qualified Stdlib v0 contour is:

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

This closeout captures the currently admitted, practical-safe helper surface
that is backed by examples, negative fixtures, and 7hell wiring.

## 2. Completed Slices

- STDLIB-0: current stdlib/helper admitted surface audit
- STDLIB-1: stdlib v0 practical helper contract
- STDLIB-2: standalone canonical `stdlib_v0_helpers` example
- STDLIB-3A: stdlib negative diagnostics fixture corpus
- STDLIB-3B: stdlib negative diagnostics harness
- STDLIB-4: 7hell stdlib diagnostics wiring

## 3. Practical-Safe Helper Surface

The following helper surface is qualified for the current PCC contour:

- `assert` is the canonical self-check helper
- `to_text(...)` is admitted for the currently supported scalar contour
- `print(text)` is admitted as a practical output helper
- `len` is admitted for the current collection contour
- `is_empty` is admitted for `Sequence(T)`
- `contains` is admitted for the current collection contour
- `push` is admitted for the current sequence contour
- `prepend` is admitted for the current sequence contour
- `pop` is admitted for the current sequence contour
- `map_empty` is admitted as the practical map constructor helper
- `map_set` is admitted as the practical map update helper
- `map_get` is admitted as the practical map read helper
- `map_contains` is admitted as the practical map query helper

## 4. Canonical Examples

The canonical examples that cover this contour are:

- `examples/canonical/stdlib_v0_helpers/src/main.sm`
- `examples/canonical/text_core/src/main.sm`
- `examples/canonical/collections_core/src/main.sm`
- `examples/canonical/text_collections_toolbox/src/main.sm`
- `examples/canonical/loop_control_flow/src/main.sm`
- `examples/canonical/match_control_flow/src/main.sm`
- `examples/canonical/option_result_control_flow/src/main.sm`

These examples are part of the canonical pack and are wired into the public
example qualification surface.

## 5. Negative Diagnostics Coverage

The stdlib negative corpus is covered by:

- `tests/pcc_stdlib_negative.rs`

Fixtures:

- `tests/fixtures/pcc/stdlib/fail/print_i32.sm`
- `tests/fixtures/pcc/stdlib/fail/print_bool.sm`
- `tests/fixtures/pcc/stdlib/fail/print_quad.sm`
- `tests/fixtures/pcc/stdlib/fail/print_sequence.sm`
- `tests/fixtures/pcc/stdlib/fail/print_map.sm`
- `tests/fixtures/pcc/stdlib/fail/to_text_record.sm`
- `tests/fixtures/pcc/stdlib/fail/to_text_sequence.sm`
- `tests/fixtures/pcc/stdlib/fail/unknown_std_namespace.sm`

## 6. 7hell Coverage

Hell 6 now runs:

```bash
cargo test --test pcc_stdlib_negative
```

The runner remains:

- linear;
- hardcoded;
- fail-fast;
- without `--group` selector;
- without fixture registry.

## 7. Documented Current Boundaries

Still out of scope for the current Stdlib v0 contour:

- final stdlib module layout;
- `core.*`, `text.*`, `seq.*`, `map.*` namespacing;
- import policy changes;
- formatting API;
- debug/logging framework;
- host ABI widening;
- `to_text(record)`;
- `to_text(collection)`;
- `print(record)`;
- `print(collection)`;
- `text + scalar`;
- map removal;
- map iteration;
- collection formatting;
- collection serialization;
- generic collection traits;
- async/concurrency helpers;
- IO/filesystem/network helpers.

## 8. Current Observed Negative Markers

- `print_i32.sm`:
  - `E0201`
  - `builtin 'print' expects text, got I32`
- `print_bool.sm`:
  - `E0201`
  - `builtin 'print' expects text, got Bool`
- `print_quad.sm`:
  - `E0201`
  - `builtin 'print' expects text, got Quad`
- `print_sequence.sm`:
  - `E0201`
  - `builtin 'print' expects text, got Sequence`
- `print_map.sm`:
  - `E0201`
  - `builtin 'print' expects text, got Map`
- `to_text_record.sm`:
  - `E0201`
  - `builtin 'to_text' does not yet support record type 'Sensor'`
- `to_text_sequence.sm`:
  - `E0201`
  - `builtin 'to_text' currently supports text, bool, i32, u32, and quad`
- `unknown_std_namespace.sm`:
  - `E0201`
  - `unknown enum type 'text' in constructor`

Exact diagnostic text and spans are intentionally not over-specified.

## 9. Validation

Passed:

```bash
cargo test -q --test pcc_stdlib_negative
powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1
```

Also covered through:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`

## 10. PCC Practical Core Status After This Closeout

The following practical contours are now closed for the current PCC phase:

- Control Flow Core
- Text Core
- Collections v0
- Stdlib v0

## 11. Recommended Next Step

Run a PCC phase summary / checkpoint before opening another contour.

Recommended document:

```text
docs/roadmap/pcc/practical_core_phase_checkpoint.md
```

Purpose:

- summarize all closed practical contours;
- list remaining known quirks;
- confirm 7hell coverage;
- identify the next PCC checkpoint or freeze-lane action.
