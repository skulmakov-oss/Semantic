# `else if` Surface Syntax

Status: language design note

## Purpose

This document defines `else if` as surface syntax for nested conditional
selection.

It is a readability improvement only. It does not add a new branch model.

## Form

```semantic
if cond_a {
    ...
} else if cond_b {
    ...
} else {
    ...
}
```

## Meaning

`else if` lowers to nested `else { if ... }` structure in source order.

Properties:

- each condition remains `bool`;
- the branch structure stays explicit;
- the canonical control-flow shape does not change.

## Use Cases

`else if` is useful when:

- a chain of explicit boolean checks reads better than nested blocks;
- the source wants to keep quad predicates readable without adding new
  control-flow semantics.

## Non-Goals

This document does not approve:

- quad truthiness;
- implicit branching on `quad`;
- branch-table optimizations by documentation alone;
- verifier, VM, or optimizer changes.

## Related Docs

- [`docs/language/quad_predicate_vocabulary.md`](quad_predicate_vocabulary.md)
- [`docs/language/quad_surface_syntax_migration.md`](quad_surface_syntax_migration.md)
- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)
