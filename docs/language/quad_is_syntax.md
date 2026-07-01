# `is` Syntax For Quad Predicates

Status: language design note

## Purpose

This document defines the `is` surface syntax for quad predicates.

It is a readable alias, not a new semantic operator family.

## Form

```semantic
x is S
```

## Meaning

`x is S` lowers to `x == S`.

The syntax is intentionally explicit:

- it returns `bool`;
- it does not convert `quad` into a condition type;
- it keeps the conflict state `S` visible in source.

## Use Cases

The form is intended for:

- branch conditions;
- predicate-heavy quad checks;
- compact readable comparisons in surface code.

## Non-Goals

This document does not approve:

- `quad` truthiness;
- hidden coercions;
- new predicate semantics;
- optimizer changes;
- runtime changes;
- verifier changes.

## Related Docs

- [`docs/language/quad_predicate_vocabulary.md`](quad_predicate_vocabulary.md)
- [`docs/language/quad_surface_syntax_migration.md`](quad_surface_syntax_migration.md)
- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)
