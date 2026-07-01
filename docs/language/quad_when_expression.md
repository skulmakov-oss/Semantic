# `when` Expression For Quad Rule Selection

Status: language design note

## Purpose

This document specifies `when` as the compact expression-selection form used in
the quad design track.

It is a surface alias for explicit boolean selection.

## Shape

```semantic
when cond {
    expr_a
} else {
    expr_b
}
```

## Meaning

`when` lowers to the canonical nested expression-`if` shape.

Properties:

- the condition must be `bool`;
- the arm values must agree in type;
- the surface form is denser, but the control-flow model remains unchanged.

## Intended Use

`when` is useful where expression-style branching reads better than a verbose
`if`/`else` block.

It should not be used as a truthiness escape hatch.

## Non-Goals

This document does not approve:

- `quad` conditions;
- hidden coercions;
- branch-table lowering claims;
- verifier, VM, or optimizer changes.

## Related Docs

- [`docs/language/quad_predicate_vocabulary.md`](quad_predicate_vocabulary.md)
- [`docs/language/quad_surface_syntax_migration.md`](quad_surface_syntax_migration.md)
- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)
