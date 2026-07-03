# Quad Surface Syntax Migration Guide

Status: proposed migration guide

## Purpose

This guide helps migrate current quad-heavy Semantic code from canonical
core-form syntax to the proposed quad surface syntax tracked by QLD.

The guide is documentation only. It does not claim the surface forms are fully
implemented unless the relevant parser, typechecker, and lowering issues have
landed.

## Core Rule

```text
bool decides.
quad means.
```

The migration goal is denser syntax without collapsing the `bool` / `quad`
boundary.

## 1. `== S` -> `is S`

Current core form:

```semantic
if merged == S {
    ...
}
```

Proposed surface form:

```semantic
if merged is S {
    ...
}
```

Canonical lowering note:

`is` is a readable predicate alias. It lowers to the same canonical equality
check as `==` against the quad literal.

Boundary warning:

This is still a `bool` predicate. It does not make `quad` into `bool`.

## 2. Nested `if` -> `else if`

Current core form:

```semantic
let state: quad = if slot == 0 {
    N
} else {
    if slot == 1 {
        F
    } else {
        if slot == 2 {
            T
        } else {
            S
        }
    }
};
```

Proposed surface form:

```semantic
let state: quad = if slot == 0 {
    N
} else if slot == 1 {
    F
} else if slot == 2 {
    T
} else {
    S
};
```

Canonical lowering note:

`else if` lowers to nested `else { if ... }` structure. The meaning stays the
same; only the source shape becomes denser.

Boundary warning:

The condition still must be explicit and boolean.

## 3. Nested Rule Selection -> `when`

Current core form:

```semantic
let value: f64 = if true {
    1.0
} else {
    2.0
};
```

Proposed surface form:

```semantic
let value: f64 = when true {
    1.0
} else {
    2.0
};
```

Canonical lowering note:

`when` lowers to the same canonical expression-`if` as the core form.

Boundary warning:

`when` is a surface selection form, not a new truthiness model.

## 4. Scalar Mapping -> `match`

Current core form:

```semantic
fn quad_wave(index: i32) -> quad {
    let slot: i32 = index % 4;
    let state: quad = if slot == 0 {
        N
    } else if slot == 1 {
        F
    } else if slot == 2 {
        T
    } else {
        S
    };
    return state;
}
```

Proposed surface form:

```semantic
fn quad_wave(index: i32) -> quad = match index % 4 {
    0..=0 => N,
    1..=1 => F,
    2..=2 => T,
    _ => S,
};
```

Canonical lowering note:

`match` is the denser decision-tree surface. It still lowers to deterministic
canonical branching.

Boundary warning:

`match` is for selection density, not for loosening quad-state semantics.

## 5. Explicit Local Types -> Inferred Locals

Current core form:

```semantic
let left: quad = quad_wave(i);
let right: quad = quad_wave(i + 1);
let merged: quad = left || right;
let checksum: i32 = 0;
```

Proposed surface form:

```semantic
let left = quad_wave(i);
let right = quad_wave(i + 1);
let merged = left || right;
let checksum = 0;
```

Canonical lowering note:

Local inference is a convenience over the same explicit typed bindings.

Boundary warning:

Inference is conservative. It should not introduce hidden coercions or hidden
`quad -> bool` conversions.

Implementation note:

Conservative local `let` inference for obvious values is implemented today.
Use explicit annotations when the type would otherwise be ambiguous.

## 6. Explicit Return Block -> Expression-Bodied Function

Current core form:

```semantic
fn quad_probe_step(left: quad, right: quad) -> quad {
    let merged: quad = left || right;
    return merged;
}
```

Proposed surface form:

```semantic
fn quad_probe_step(left: quad, right: quad) -> quad = left || right;
```

Canonical lowering note:

The expression-bodied form lowers to a block with the same returned value.

Boundary warning:

The shorter surface form should not hide effectful work or weaken the return
type boundary.

## Migration Advice

1. Keep the core form as the source of truth while the surface syntax remains
   proposed.
2. Migrate one idiom at a time:
   - predicates first;
   - branch structure second;
   - selection forms third;
   - local inference last.
3. Prefer surface density only when it stays easy to audit.
4. Keep `S` visible as conflict.
5. Keep `bool` conditions explicit.

## Status Notes

- `== S`, `is`, `when`, `else if`, `match`, and expression-bodied functions
  are part of the current QLD surface-syntax track.
- conservative local `let` inference is implemented today for obvious local
  values.
- This guide documents the migration path; it does not widen release claims.

## Related Docs

- [`docs/language/quad_language_design.md`](quad_language_design.md)
- [`docs/language/semantic_language_experience.md`](semantic_language_experience.md)
- [`docs/language/quad_lexical_model.md`](quad_lexical_model.md)
- [`docs/language/quad_operation_families.md`](quad_operation_families.md)
- [`docs/language/quad_predicate_vocabulary.md`](quad_predicate_vocabulary.md)
- [`docs/language/quad_is_syntax.md`](quad_is_syntax.md)
- [`docs/language/quad_else_if_surface_syntax.md`](quad_else_if_surface_syntax.md)
- [`docs/language/quad_match_surface_syntax.md`](quad_match_surface_syntax.md)
- [`docs/language/quad_when_expression.md`](quad_when_expression.md)
- [`docs/language/quad_expression_bodied_functions.md`](quad_expression_bodied_functions.md)
- [`docs/language/quad_local_type_inference.md`](quad_local_type_inference.md)
- [`docs/examples/quad_language_examples.md`](../examples/quad_language_examples.md)
- [`docs/roadmap/language_maturity/quad_language_design_roadmap.md`](../roadmap/language_maturity/quad_language_design_roadmap.md)
