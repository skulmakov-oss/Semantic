# PCC Text Negative Diagnostics Plan

Status: PCC-TEXT-3 fixture corpus + diagnostics plan

## Goal

Define the first negative fixture corpus for Practical Core text diagnostics.

## Scope

This plan covers negative fixtures for:

- implicit `text + scalar`;
- `to_text(record)`;
- multiline text literal;
- text ordering comparison.

## Fixture Path

```text
tests/fixtures/pcc/text/fail/
```

## Fixture Matrix

| Fixture | Expected failure |
| --- | --- |
| `text_plus_i32.sm` | implicit scalar concat forbidden |
| `text_plus_bool.sm` | implicit scalar concat forbidden |
| `text_plus_quad.sm` | implicit scalar concat forbidden |
| `to_text_record.sm` | `to_text(record)` out of scope |
| `multiline_text.sm` | multiline text literal out of scope |
| `text_ordering.sm` | text ordering out of scope |

## Harness Policy

The harness should assert:

- the fixture fails;
- a broad diagnostic marker is present;
- no panic occurs.

It should not assert full diagnostic text or exact spans.

## Harness Status

`PCC-TEXT-3B` adds:

```text
tests/pcc_text_negative.rs
```

The harness asserts:

- each text-negative fixture fails;
- broad diagnostic markers are present;
- no panic occurs.

It intentionally does not assert exact spans or full diagnostic rendering.

## Expected Current Markers

Observed markers recorded after manual probe:

- `text_plus_i32.sm`: `E0201` - text concatenation currently admits only text + text operands
- `text_plus_bool.sm`: `E0201` - text concatenation currently admits only text + text operands
- `text_plus_quad.sm`: `E0201` - text concatenation currently admits only text + text operands
- `to_text_record.sm`: `E0201` - builtin 'to_text' does not yet support record type 'Sensor'
- `multiline_text.sm`: `E0000` - unterminated string literal
- `text_ordering.sm`: `E0201` - relational operators are currently admitted only for same-family i32 operands in the first application-completeness wave

## Follow-Up

- PCC-TEXT-4: wire text diagnostics into 7hell
- PCC-TEXT-5: text closeout

## Closeout

See [`text_core_closeout.md`](text_core_closeout.md) for the final contour
summary once the text qualification path is fully closed.

## Explicit Non-Goals

- No diagnostics redesign.
- No formatting API.
- No widening of `to_text(record)`.
- No allowance for `text + scalar`.
- No 7hell integration before a harness exists.
- No exact span assertions yet.
