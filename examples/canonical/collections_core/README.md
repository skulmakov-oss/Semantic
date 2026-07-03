# Canonical example: collections_core

This canonical example demonstrates the current Practical Core collections
surface in Semantic.

## Purpose

The example shows `Sequence(T)` and `Map(K, V)` as practical admitted
collection forms.

It avoids text formatting, host ABI widening, map iteration, collection
serialization, and generic collection abstractions.

## What this example covers

- `Sequence(i32)`;
- sequence literals where admitted;
- `for` over a sequence;
- `len`;
- `is_empty`;
- `contains`;
- `push`;
- `prepend`;
- `pop`;
- `Map(i32, bool)`;
- `map_empty`;
- `map_set`;
- `map_get`;
- `map_contains`;
- `assert`-based self-checks.

## What this example does not cover

- `remove`;
- map iteration;
- iterator protocol;
- collection formatting;
- `to_text(collection)`;
- host-facing collection ABI widening;
- serialization;
- advanced generic collection framework.

## Expected behavior

The program should pass `smc check`.

The internal checks validate:

```text
sequence_score() == 10
map_score() == 10
```

## Validation

```bash
cargo run --bin smc -- check examples/canonical/collections_core/src/main.sm
```

This example should also be included in:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`
