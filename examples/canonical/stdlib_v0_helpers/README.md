# Canonical example: stdlib_v0_helpers

This canonical example demonstrates `semantic.foundation.std/0.1`, the bounded
Stable Foundation Standard Library v0 candidate.

## Purpose

The example shows currently admitted helper functions used together in a small
practical program.

The `std.*` family names are documentation identities. Foundation Source 1.1
uses canonical language-owned builtins and standard forms rather than
namespace-qualified imports.

## What this example covers

- `assert`;
- `qtruth_and`, `qtruth_or`, `qtruth_not`, `qtruth_impl`;
- `to_text(...)`;
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
- `Option::Some` / `Option::None` and match;
- `Result::Ok` / `Result::Err` and match;
- deterministic `random_seed` / `random_next_i32` replay.

## What this example does not cover

- importable `std.*` namespace facades;
- `std.math` APIs;
- `std.serde` APIs or encodings;
- `print(text)` or any other host effect;
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
seeded random values replay exactly after reseeding
```

## Validation

```bash
cargo run --bin smc -- run examples/canonical/stdlib_v0_helpers/src/main.sm
```

This example should also be included in:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`
