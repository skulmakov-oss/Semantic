# Quad Operation Families

Status: language design note

## Purpose

This document groups the quad operation families already used by the canonical
spec and the quad surface track.

It does not add new runtime behavior. It records the vocabulary boundary for
quad-heavy source.

## Families

### 1. Identity Predicates

Identity predicates answer whether two values have the same state.

Examples:

- `a == b`
- `a != b`

Properties:

- return `bool`;
- do not perform evidence merging;
- do not imply control flow;
- remain explicit comparison operators.

### 2. Evidence Algebra

Evidence operators operate on the quad evidence plane and return `quad`.

Examples:

- `a && b`
- `a || b`
- `!a`
- `a -> b`

Properties:

- preserve quad-state semantics;
- do not collapse into control-flow predicates;
- remain value-level operations.

### 3. Control-Flow Predicates

Control-flow predicates are explicit `bool` predicates used for branching.

Examples:

- `a == T`
- `a == S`
- `x is S`
- `known(x)`
- `unknown(x)`
- `conflict(x)`

Properties:

- return `bool`;
- are admitted in `if` and other boolean decision sites;
- are surface aliases over canonical boolean predicates.

### 4. Selection Forms

Selection forms keep quad-heavy code denser while still lowering to canonical
branching.

Examples:

- `when`
- `else if`
- `match`

Properties:

- source-density helpers only;
- lower to explicit canonical control flow;
- do not introduce implicit truthiness.

## Boundary Rule

The families are distinct:

- value-level quad operators stay value-level;
- boolean predicates stay explicit;
- selection sugar stays sugar.

This boundary keeps `bool` and `quad` separate in source and in lowering.

## Non-Goals

This document does not approve:

- implicit `quad -> bool`;
- hidden coercions;
- new operator families;
- optimizer changes;
- VM changes;
- verifier changes.

## Related Docs

- [`docs/language/quad_lexical_model.md`](quad_lexical_model.md)
- [`docs/language/quad_language_design.md`](quad_language_design.md)
- [`docs/language/quad_surface_syntax_migration.md`](quad_surface_syntax_migration.md)
- [`docs/spec/types.md`](../spec/types.md)
- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)
