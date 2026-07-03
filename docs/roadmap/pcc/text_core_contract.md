# PCC Text Core Contract

Status: PCC-TEXT-1 contract draft

This document defines the currently qualified Practical Core text surface for
Semantic.

It is based on:

- `docs/roadmap/pcc/text_core_audit.md`
- `docs/spec/source_semantics.md`
- current canonical examples and text fixtures

## Scope

This contract covers the current PCC practical text contour.

It does not claim that the full language spec has been finalized around text.

The current practical contour admits:

- one-line text literals;
- empty text literal `""`;
- `text == text`;
- `text != text`;
- bounded `text + text`;
- `to_text(...)` for admitted scalar families;
- `print(text)`.

## Current Canonical-Safe Forms

Examples may use:

```semantic
let label: text = "sensor";
let status: text = "ok";
let message: text = label + status;

assert(message == "sensorok");
assert(message != "");
print(message);
```

This contract allows text to participate in practical examples, but it keeps the
surface deliberately narrow and explicit.

## Scalar Conversion Boundary

`to_text(...)` is currently admitted for scalar families already covered by the
current source/runtime contour.

Canonical examples may use scalar conversion only for already admitted scalar
types.

This contract does not admit:

- `to_text(record)`;
- `to_text(collection)`;
- arbitrary user-defined formatting;
- implicit scalar-to-text concatenation.

## Operator Boundary

Allowed:

```semantic
let c: text = a + b;
```

where both `a` and `b` are `text`.

Not admitted as canonical-safe:

```semantic
let c: text = "value=" + 42;
```

Use explicit conversion instead:

```semantic
let c: text = "value=" + to_text(42);
```

## Equality Boundary

Allowed:

```semantic
if a == b {
    assert(true);
}

if a != "" {
    assert(true);
}
```

Out of scope:

- text ordering;
- locale-aware comparison;
- case folding;
- normalization.

## Print Boundary

`print(text)` is admitted as a practical public helper.

This contract does not widen host-facing text ABI beyond the current admitted
behavior.

## Spec Alignment Note

`docs/spec/syntax.md` remains more conservative than the current PCC practical
contour.

This document records the current admitted practical surface for PCC
qualification.

It should not be treated as a full replacement for the language spec.

## Qualification Fixtures

Positive fixtures:

- `tests/fixtures/snake_benchmark/positive_text_to_text.sm`
- `tests/fixtures/snake_benchmark/positive_text_concat.sm`
- `tests/fixtures/snake_benchmark/positive_text_equality.sm`
- `tests/fixtures/snake_benchmark/positive_print.sm`

Negative fixtures:

- `tests/fixtures/snake_benchmark/negative_text_plus_scalar.sm`
- `tests/fixtures/snake_benchmark/negative_to_text_record.sm`
- `tests/fixtures/snake_benchmark/negative_print_non_text.sm`

Canonical anchor:

- `examples/canonical/text_core/src/main.sm`
- `examples/canonical/text_collections_toolbox/src/main.sm`

## Follow-Up Issues

- PCC-TEXT-2: add canonical text-only example
- PCC-TEXT-3: add text negative diagnostics fixtures
- PCC-TEXT-4: wire text diagnostics into 7hell if needed
- PCC-TEXT-5: close out the text contour

## Closeout

See [`text_core_closeout.md`](text_core_closeout.md) for the closeout record
once the contour is fully qualified.

## Explicit Non-Goals

- No language widening.
- No claim that interpolation is ready.
- No claim that multiline text is admitted.
- No claim that the text contour is release-stable.
- No canonical promotion of any new text operator until probe evidence exists.
- No host-facing text ABI widening.
