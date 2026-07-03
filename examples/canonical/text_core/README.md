# Canonical example: text_core

This canonical example demonstrates the current Practical Core `text` surface
in Semantic.

## Purpose

The example shows `text` as a practical admitted surface without mixing it with
collections, `Option`, `Result`, `match`, or loop behavior.

It qualifies the current PCC text contour around:

- one-line text literals;
- empty text literal `""`;
- `text == text`;
- `text != text`;
- bounded `text + text`;
- explicit scalar conversion through `to_text(...)`;
- `print(text)`;
- `assert`-based self-checks.

## What this example covers

- `text` function parameters;
- `text` return value;
- local `text` bindings;
- concatenation of text values;
- explicit `to_text(i32)`;
- equality and inequality checks;
- practical output through `print(text)`.

## What this example does not cover

- interpolation;
- multiline text;
- raw strings;
- implicit `text + scalar`;
- `to_text(record)`;
- collection formatting;
- host-facing text ABI widening;
- formatting API;
- Unicode normalization policy.

## Expected behavior

The program should pass `smc check`.

At runtime, the internal assertions check:

- `build_status("pressure", 42) == "sensor:pressure=42"`
- `message != ""`
- `check_status(message) == 1`

## Validation

```bash
cargo run --bin smc -- check examples/canonical/text_core/src/main.sm
```

This example should also be included in:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`
