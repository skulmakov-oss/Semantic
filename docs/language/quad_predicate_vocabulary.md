# Quad Predicate Vocabulary

Status: language design note

## Purpose

This document defines the surface predicate vocabulary for quad values.

The vocabulary is intentionally narrow and stays on the `bool` side of the
`bool` / `quad` boundary.

## Predicate Forms

### `is`

`x is S` is a readable predicate alias for `x == S`.

Properties:

- returns `bool`;
- is only a surface spelling;
- does not change the underlying quad state model.

### `known`

`known(x)` is a predicate alias for `x != N`.

Properties:

- returns `bool`;
- means the value is not unknown;
- does not invent a new tri-state or truthiness rule.

### `unknown`

`unknown(x)` is a predicate alias for `x == N`.

Properties:

- returns `bool`;
- means the value is unknown;
- remains an explicit comparison-based predicate.

### `conflict`

`conflict(x)` is a predicate alias for `x == S`.

Properties:

- returns `bool`;
- means the value is in conflict;
- keeps `S` visible as a first-class quad state.

## Vocabulary Rule

The predicate vocabulary exists to make quad-heavy branching readable without
changing canonical semantics.

That means:

- predicates remain explicit;
- conditions remain `bool`;
- quad values do not become branch control by implication.

The current vocabulary is:

- `x is S` -> `x == S`
- `known(x)` -> `x != N`
- `unknown(x)` -> `x == N`
- `conflict(x)` -> `x == S`

## Non-Goals

This document does not approve:

- implicit `quad -> bool`;
- hidden coercions;
- new predicate meanings;
- runtime or verifier changes;
- lowering changes by documentation alone.

## Related Docs

- [`docs/language/quad_lexical_model.md`](quad_lexical_model.md)
- [`docs/language/quad_operation_families.md`](quad_operation_families.md)
- [`docs/language/quad_language_design.md`](quad_language_design.md)
- [`docs/spec/types.md`](../spec/types.md)
- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)
