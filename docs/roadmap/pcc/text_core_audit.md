# PCC Text Core Audit

Status: working audit note for the practical text contour

## Purpose

This document records the current admitted text surface in Semantic as
observed through the source specs, canonical examples, and text-focused
fixtures.

It is intentionally conservative. It does not widen the language contract.
It captures the current practical contour so the next text issues can be
scoped cleanly.

## Executive Verdict

Text is already useful on the current admitted surface, but the contract is
split across layers:

- `docs/spec/syntax.md` still describes the text surface conservatively.
- `docs/spec/source_semantics.md` and the current fixtures show a practical
  contour with `text`, same-family equality, bounded concatenation, and
  explicit `to_text` helpers.

Observed practical anchors:

- [examples/canonical/text_collections_toolbox/](C:\Users\said3\Desktop\EXOcode\EXOcode\examples\canonical\text_collections_toolbox\README.md)
- [examples/canonical/text_core/](C:\Users\said3\Desktop\EXOcode\EXOcode\examples\canonical\text_core\README.md)
- `tests/fixtures/snake_benchmark/positive_text_to_text.sm`
- `tests/fixtures/snake_benchmark/positive_text_concat.sm`
- `tests/fixtures/snake_benchmark/positive_text_equality.sm`
- `tests/fixtures/snake_benchmark/positive_print.sm`

Current verdict:

- `text` is a practical source type on the current contour.
- double-quoted same-line text literals are admitted.
- empty text `""` is admitted.
- same-family `text == text` and `text != text` are admitted.
- bounded `text + text` concatenation is admitted on the current practical
  contour.
- `to_text` is the primary stdlib/helper bridge for admitted scalar families.
- host-facing text ABI widening is still out of scope.

## Observed Surface

### Text literals

Observed admitted literal forms:

- double-quoted same-line text, for example `"hello"`
- empty text `""`

Not qualified here:

- interpolation
- multiline text blocks
- raw text literals

### Text type

- current source examples use the lowercase `text` spelling
- the audit does not claim any alternate public spelling such as `Text`

### Equality and comparison

Observed admitted same-family operations:

- `text == text`
- `text != text`

Not qualified here:

- ordering comparisons on text
- lexical ordering guarantees
- collation policy

### Concatenation

Observed admitted practical form:

- `text + text`

Observed rejection:

- `text + scalar` is rejected by the current fixture surface

Not qualified here:

- mixed concatenation with `i32`, `u32`, `bool`, `quad`, or records
- formatting interpolation
- operator overloading beyond the current bounded contour

### Stdlib / helper bridge

Observed helper surface:

- `to_text(i32)`
- `to_text(u32)`
- `to_text(bool)`
- `to_text(quad)`
- `to_text("done")`

Observed boundary:

- `to_text(record)` is not admitted by the current fixture surface

Observed consumer:

- `print(text)` is admitted
- `print(i32)` is rejected

## Canonical Anchors

### Mixed text + collections anchor

`examples/canonical/text_collections_toolbox/src/main.sm`

This example shows:

- `text` as a return type
- `to_text(len(...))`
- `text + text` concatenation
- text equality in a runnable canonical example
- text used alongside `Sequence` and `Map`

### Standalone text anchor

`examples/canonical/text_core/src/main.sm`

This example shows:

- `text` as an input and return type without collections noise
- bounded `text + text` concatenation
- explicit `to_text(i32)` conversion
- text equality and inequality checks
- `print(text)` as a practical output helper

### Text helper fixtures

Positive fixtures:

- `tests/fixtures/snake_benchmark/positive_text_to_text.sm`
- `tests/fixtures/snake_benchmark/positive_text_concat.sm`
- `tests/fixtures/snake_benchmark/positive_text_equality.sm`
- `tests/fixtures/snake_benchmark/positive_print.sm`

Negative fixtures:

- `tests/fixtures/snake_benchmark/negative_text_plus_scalar.sm`
- `tests/fixtures/snake_benchmark/negative_to_text_record.sm`
- `tests/fixtures/snake_benchmark/negative_print_non_text.sm`

## Stable Quirks

- `docs/spec/syntax.md` still documents the text-literal surface
  conservatively and does not promote the full practical contour.
- current practical concatenation is bounded to `text + text`.
- interpolation and multiline text are not part of the current admitted
  surface.
- `to_text` is admitted only for the scalar families shown by the current
  fixtures.
- host ABI widening for text remains out of scope.

## Runtime / Stdlib Boundary

Current practical boundary:

- text literals are source-level surface
- `to_text` is the canonical conversion helper
- `print` is the canonical consumer helper for `text`
- collection helpers can compose with text, but that is a mixed contour and
  should not be confused with a text-only contract

## Not Yet Canonicalized Here

- text-only canonical example pack
- interpolation
- multiline text blocks
- raw string variants
- text formatting API
- general text utility library design
- host-facing text ABI widening

## Evidence Summary

Current text evidence is distributed across:

- source semantics documentation
- syntax documentation
- canonical mixed example
- positive and negative text fixtures

Practical summary:

- the admitted contour already supports ordinary text handling
- the contour is still intentionally bounded
- text should be treated as a practical core surface, not a fully open-ended
  string subsystem

## Follow-Up Issues

Recommended next issue pack:

- PCC-TEXT-1: specify text core contract
- PCC-TEXT-2: define text / stdlib boundary policy
- PCC-TEXT-3: add canonical text-only example
- PCC-TEXT-4: add negative diagnostics fixtures for text gaps
- PCC-TEXT-5: close out the text contour

## Closeout

See [`text_core_closeout.md`](text_core_closeout.md) for the closeout record
after the contour is fully wired.

## Non-Goals

- No language widening.
- No claim that interpolation is ready.
- No claim that multiline text is admitted.
- No claim that the text contour is release-stable.
- No canonical promotion of any new text operator until probe evidence exists.
