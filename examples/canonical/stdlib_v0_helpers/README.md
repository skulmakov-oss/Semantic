# Canonical example: stdlib_v0_helpers

This canonical example demonstrates the current Practical Core helper surface
for Stdlib v0.

## Purpose

The example shows currently admitted helper functions used together in a small
practical program.

It does not define the final standard library architecture.

## What this example covers

- `assert`;
- `to_text(...)`;
- `print(text)`;
- `Sequence(i32)` helpers:
  - `len`;
  - `is_empty`;
  - `contains`;
  - `push`;
  - `prepend`;
  - `pop`;
- `Map(i32, bool)` helpers:
  - `map_empty`;
  - `map_set`;
  - `map_get`;
  - `map_contains`.

## What this example does not cover

- stdlib module layout;
- `core.*`, `text.*`, `seq.*`, `map.*` namespacing;
- import policy;
- formatting API;
- debug/logging framework;
- host ABI widening;
- `to_text(record)`;
- `to_text(collection)`;
- `print(record)`;
- `print(collection)`;
- map removal;
- map iteration;
- collection serialization.

## Expected behavior

The program should pass `smc check`.

The internal checks validate:

```text
sum_values(build_values()) == 10
pop_score() == 3
score_flags(build_flags()) == 10
build_report(10, 10) == "stdlib-v0:10:10"
```

## Validation

```bash
cargo run --bin smc -- check examples/canonical/stdlib_v0_helpers/src/main.sm
```

This example should also be included in:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`
