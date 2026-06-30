# Quad Surface Lowering Density Audit

Status: documentation audit
Track: Semantic language / quad surface
Scope type: documentation only

## Purpose

This audit records whether quad-heavy surface syntax stays readable without
changing the canonical lowering contract.

The goal is not to tighten the grammar. The goal is to make the lowering path
visible enough that dense quad code still reads as deliberate semantics rather
than accidental boolean noise.

## Current Findings

- `let q = T;` stays compact when the initializer already fixes the meaning
- `if q == T { ... }` keeps the branch predicate explicit
- `match q { ... }` is the densest readable form when all four quad values
  matter
- `else if` remains nested `if` sugar and does not add a hidden branch family
- `bool` remains the control type; `quad` remains semantic state

## Accepted Density Pattern

The current accepted density pattern is:

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

## Audit Conclusion

The lowering density is acceptable when:

- branch intent is explicit
- conflict remains visible
- canonical lowering stays deterministic
- the source does not require truthiness to be understood

## Non-Goals

This audit does not:

- change compiler behavior
- add a formatter
- define a new grammar
- replace `docs/spec/*`

