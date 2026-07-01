# Quad Lexical Model

Status: language design note

## Purpose

This document defines the lexical model for quad-heavy surface syntax.

It stays aligned with the canonical quad semantics already documented in the
spec:

- [`docs/spec/types.md`](../spec/types.md)
- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)

## Core Rule

```text
bool decides.
quad means.
```

## Quad Vocabulary

`quad` is a first-class semantic logic type with four literal values:

- `N`
- `F`
- `T`
- `S`

These literals are not interchangeable synonyms. They are distinct source
states with distinct semantics.

## Lexical Meaning

In the current design track:

- `N` represents unknown state;
- `F` represents false;
- `T` represents true;
- `S` represents conflict.

The lexical model is intentionally small and explicit. It does not introduce a
new general-purpose truthiness system.

## Control-Flow Boundary

`quad` remains a value domain, not a condition domain.

Current control-flow rules remain:

- `if` conditions must be `bool`;
- a bare `quad` does not control execution flow;
- explicit predicates such as `x == T` or `x is S` are admitted because they
  return `bool`.

## Surface Vocabulary

The lexical model is the base layer for the current quad surface vocabulary:

- `x is S`
- `known(x)`
- `unknown(x)`
- `conflict(x)`
- `match` over quad literals

These forms are source-density aids only. They do not relax canonical
semantics.

## Non-Goals

This document does not approve:

- implicit `quad -> bool`;
- hidden coercions;
- new literal states;
- runtime or verifier changes;
- lowering changes by documentation alone.

## Related Docs

- [`docs/language/quad_language_design.md`](quad_language_design.md)
- [`docs/language/quad_surface_syntax_migration.md`](quad_surface_syntax_migration.md)
- [`docs/roadmap/language_maturity/quad_language_design_roadmap.md`](../roadmap/language_maturity/quad_language_design_roadmap.md)
