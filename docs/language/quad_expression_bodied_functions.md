# Expression-Bodied Functions

Status: language design note

## Purpose

This document specifies the expression-bodied function surface form used in the
quad design track.

It is a concise spelling for a function whose body is a single expression.

## Shape

```semantic
fn name(arg: T) -> U = expr;
```

## Meaning

The expression-bodied form lowers to an ordinary block function with a final
`return` of the body expression.

Properties:

- it keeps the return type explicit;
- it does not change evaluation order;
- it does not introduce hidden control flow.

## Intended Use

Use the shorter spelling when:

- the function body is a single value-producing expression;
- the expanded block would add noise without adding meaning.

## Non-Goals

This document does not approve:

- implicit return typing;
- multi-statement bodies hidden behind the short form;
- runtime or verifier changes;
- optimizer changes by documentation alone.

## Related Docs

- [`docs/language/quad_surface_syntax_migration.md`](quad_surface_syntax_migration.md)
- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)
