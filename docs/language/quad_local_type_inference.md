# Conservative Local Type Inference

Status: language design note

## Purpose

This document specifies conservative local type inference for local `let`
bindings in the quad design track.

The goal is source reduction, not semantic widening.

## Scope

Inference is limited to local bindings such as:

```semantic
let value = expr;
```

## Meaning

Local inference removes obvious annotations when the initializer already
determines the binding type.

Properties:

- it stays local;
- it remains conservative;
- it does not introduce hidden coercions;
- it does not convert `quad` into `bool`.

## Intended Use

Use it when the explicit type would be redundant for the reader and the
initializer is unambiguous.

Keep explicit annotations when the type boundary matters for clarity.

## Non-Goals

This document does not approve:

- global inference;
- hidden type widening;
- implicit `quad -> bool`;
- inference-driven optimizer behavior;
- verifier or VM changes.

## Related Docs

- [`docs/language/quad_language_design.md`](quad_language_design.md)
- [`docs/language/quad_surface_syntax_migration.md`](quad_surface_syntax_migration.md)
- [`docs/spec/types.md`](../spec/types.md)
