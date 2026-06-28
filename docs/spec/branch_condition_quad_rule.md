# Branch Condition Quad Rule

Status: draft v0

## Purpose

This document records the current source contract for `if` conditions when a
value is `quad`-typed.

The rule is intentionally narrow:

- bare `quad` conditions are rejected with a diagnostic;
- explicit comparisons against `T` or `F` are admitted;
- the contract does not change `quad` as a value domain.

## Current Rule

Current `if` semantics require a `bool` condition.

`quad` is not treated as an implicit condition type.

Therefore:

- `if quad_expr { ... } else { ... }` is rejected;
- `if state == T { ... } else { ... }` is accepted;
- `if state == F { ... } else { ... }` is accepted.

## Diagnostics

The source-facing diagnostic for a bare `quad` condition is expected to keep
the stable wording family:

- `if condition must be bool`
- `explicit compare is required for quad`

The exact rendered diagnostic may include additional source context, but the
comparison requirement must remain explicit.

## Examples

Accepted:

```text
let q: quad = T;
if q == T {
    return;
} else {
    return;
}
```

Accepted:

```text
let q: quad = F;
if q == F {
    return;
} else {
    return;
}
```

Rejected:

```text
let q: quad = T;
if q {
    return;
} else {
    return;
}
```

## Invariants

- `quad` remains a first-class value domain.
- `if` conditions remain `bool`-only.
- explicit compare semantics keep branch intent readable and non-implicit.
- this rule does not introduce a general truthiness model.
