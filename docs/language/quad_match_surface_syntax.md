# `match` For Compact Quad Selection

Status: language design note

## Purpose

This document specifies `match` as the compact selection form used in quad-heavy
surface code.

It is a source-density feature. It does not introduce implicit truthiness.

## Shape

```semantic
match value {
    N => { ... }
    F => { ... }
    T => { ... }
    S => { ... }
    _ => { ... }
}
```

## Meaning

`match` keeps selection explicit and deterministic.

Current rules:

- quad literal arms are allowed;
- the default arm `_` remains explicit;
- the lowered form remains canonical branching, not a hidden decision oracle.

## Intended Use

The form is intended for:

- compact quad-state selection;
- branch-heavy code that would otherwise become nested `if` chains;
- readable exhaustive handling of the four quad literals.

## Non-Goals

This document does not approve:

- implicit `quad -> bool`;
- hidden coercions;
- nondeterministic dispatch;
- verifier or VM changes;
- optimizer changes by documentation alone.

## Related Docs

- [`docs/language/quad_operation_families.md`](quad_operation_families.md)
- [`docs/language/quad_surface_syntax_migration.md`](quad_surface_syntax_migration.md)
- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)
