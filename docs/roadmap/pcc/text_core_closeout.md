# PCC Text Core Closeout

## 0. Status

Status:
  COMPLETE / CURRENT PCC TEXT CORE CONTOUR CLOSED

This document does **not** claim full language completion.
This document does **not** claim interpolation readiness.
This document does **not** claim multiline text readiness.
This document does **not** claim `text + scalar`, `to_text(record)`, or
host-facing text ABI widening.

## 1. Closed Scope

The qualified text contour is:

- one-line text literals;
- empty text literal `""`;
- `text == text`;
- `text != text`;
- bounded `text + text`;
- explicit `to_text(...)` for admitted scalar families;
- `print(text)`;
- `assert`-based text self-checks.

This closeout captures the currently admitted, practical-safe text surface that
is backed by examples, negative fixtures, and 7hell wiring.

## 2. Completed Slices

- TEXT-0: current admitted text surface audit
- TEXT-1: text core contract
- TEXT-2: standalone canonical `text_core` example
- TEXT-3A: text negative diagnostics fixture corpus
- TEXT-3B: text negative diagnostics harness
- TEXT-4: 7hell text diagnostics wiring

## 3. Practical-Safe Surface

The following text surface is qualified for the current PCC contour:

- `text` is a practical source type
- one-line text literals are admitted
- empty text `""` is admitted
- `text == text` and `text != text` are admitted
- current practical concatenation is bounded to `text + text`
- `to_text(...)` is admitted for the scalar families shown by the current
  fixtures
- `print(text)` is admitted as a practical helper

## 4. Canonical Examples

The canonical examples that cover this contour are:

- `examples/canonical/text_core/src/main.sm`
- `examples/canonical/text_collections_toolbox/src/main.sm`

These examples are part of the canonical pack and are wired into the public
example qualification surface.

## 5. Negative Diagnostics Coverage

The text negative corpus is covered by:

- `tests/pcc_text_negative.rs`

Fixtures:

- `tests/fixtures/pcc/text/fail/text_plus_i32.sm`
- `tests/fixtures/pcc/text/fail/text_plus_bool.sm`
- `tests/fixtures/pcc/text/fail/text_plus_quad.sm`
- `tests/fixtures/pcc/text/fail/to_text_record.sm`
- `tests/fixtures/pcc/text/fail/multiline_text.sm`
- `tests/fixtures/pcc/text/fail/text_ordering.sm`

## 6. 7hell Coverage

Hell 6 now runs:

```bash
cargo test --test pcc_text_negative
```

The runner remains:

- linear;
- hardcoded;
- fail-fast;
- without `--group` selector;
- without fixture registry.

## 7. Documented Current Boundaries

Still out of scope for the current text contour:

- interpolation;
- multiline text;
- raw strings;
- `text + scalar`;
- implicit scalar-to-text conversion;
- `to_text(record)`;
- collection formatting;
- host-facing text ABI widening;
- formatting API;
- Unicode normalization policy;
- locale-aware comparison;
- text ordering.

## 8. Current Observed Negative Markers

- `text + i32/bool/quad`:
  - `E0201`
  - `text concatenation currently admits only text + text operands`
- `to_text(record)`:
  - `E0201`
  - `builtin 'to_text' does not yet support record type`
- multiline text:
  - `E0000`
  - `unterminated string literal`
- text ordering:
  - `E0201`
  - `relational operators are currently admitted only for same-family i32 operands`

Exact diagnostic text and spans are intentionally not over-specified.

## 9. Validation

Passed:

```bash
cargo test -q --test pcc_text_negative
powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1
```

Also covered through:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`

## 10. Next PCC Contour

Recommended next practical contour:

```text
Collections v0
```

Reason:

Text Core is now qualified enough for current PCC. Collections are the next
practical axis needed for ordinary small programs.
