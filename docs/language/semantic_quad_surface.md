# Semantic Quad Surface

Status: quad-surface design note
Scope type: documentation only

Related:

- `docs/roadmap/language_maturity/quad_language_design_roadmap.md`
- `docs/spec/types.md`
- `docs/spec/source_semantics.md`
- `docs/spec/syntax.md`
- `docs/spec/branch_condition_quad_rule.md`

## Purpose

This document collects the quad surface contract in one readable place.

It covers the lexical model, operation families, predicate vocabulary, surface
selection forms, lowering guidance, and migration advice for quad-heavy code.

## Lexical Model

Semantic quad code uses four visible values:

- `N` for unknown
- `F` for false
- `T` for true
- `S` for conflict

These are semantic states, not boolean aliases.

## Operation Families

Quad-related operations are grouped into three families:

- identity predicates: `==`, `!=`
- evidence operators: `&&`, `||`, `!`, `->`
- selection predicates: explicit compare or `match`

The families are distinct on purpose:

- identity predicates return `bool`
- evidence operators return `quad`
- selection predicates return `bool` or drive explicit quad dispatch

## Predicate Vocabulary

The current admitted branch vocabulary is explicit:

- `if q == T { ... }`
- `if q == F { ... }`
- `match q { N => ... F => ... T => ... S => ... _ => ... }`

Roadmap vocabulary candidates such as `when` and `is` should be treated as
design candidates, not as current admitted grammar.

## Surface Syntax

The compact quad surface should read as a progression:

1. declare or infer a quad local
2. compare explicitly when branching
3. use `match` when all four states matter
4. keep `else if` as nested `if` sugar when present

Examples:

```semantic
let boot:quad = T;
if boot==T { observe "ready"; } else { observe "pending"; }
```

```semantic
match boot {
    N=>{ observe "unknown"; }
    F=>{ observe "false"; }
    T=>{ observe "true"; }
    S=>{ observe "conflict"; }
    _=>{ observe "fallback"; }
}
```

## Canonical Lowering

Canonical lowering stays deterministic:

- `if q == T { ... }` lowers through the `bool` result of the comparison
- `if q == F { ... }` lowers the same way
- `else if` lowers as nested `if` in source order
- `match q` lowers as ordered dispatch over the quad literals
- `_` remains the explicit default arm

The lowering contract should never silently reintroduce truthiness.

## Conservative Local Inference

Quad-heavy code may use local inference when the initializer already fixes the
meaning:

```semantic
let q = T;
let ready = q == T;
```

When the source needs to communicate intent more directly, an explicit
annotation remains preferred:

```semantic
let q:quad = T;
```

The conservative rule is:

- infer when the initializer is already unambiguous
- annotate when the reader needs the semantic type to stay obvious

## Migration Guide

Use the following migration shape when moving from generic-looking code to
quad-heavy code:

- replace truthy-looking branch logic with explicit comparison
- replace scattered boolean branching with `match` when all four quad states
  matter
- keep conflict visible instead of collapsing it into fallback
- annotate locals when the surrounding context would otherwise hide the quad
  meaning

## Dense Examples

```semantic
if q==T { ... } else if q==F { ... } else { ... }
```

```semantic
match q {
    N=>{ ... }
    F=>{ ... }
    T=>{ ... }
    S=>{ ... }
    _=>{ ... }
}
```

```semantic
let boot:quad = T;
if boot==T { observe "ready"; }
```

## Non-Goals

This document does not:

- claim `when` or `is` are currently admitted parser forms
- collapse `quad` into boolean truthiness
- widen runtime behavior
- change verifier policy
- replace `docs/spec/*`

