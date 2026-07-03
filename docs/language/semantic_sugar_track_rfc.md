# Semantic Sugar Track RFC

**Status:** proposal

## Purpose
This document records a narrow syntactic-sugar track for Semantic.
The goal is to improve readability and reduce ritual without changing the
deterministic execution model, verifier boundaries, or SemCode lowering
discipline.

## Design Principles
- Sugar must lower to an obvious canonical core form.
- Sugar must not add hidden runtime behavior.
- Sugar must not blur bool and quad.
- Sugar must not create a second canonical way to express the same core rule.
- Sugar must be readable in dense policy-heavy code.
- Sugar must stay easy to diagnose when malformed.

## Proposed Surface

### Field Punning
Field punning allows record fields to omit the redundant `field: field` form
when a local binding with the same name is already in scope.

**Example:**
```semantic
let consensus = S;
let alert = T;
let quality = 0.8;

let plan = DecisionPlan { consensus, alert, quality };
```

**Canonical lowering:**
```semantic
let plan = DecisionPlan {
    consensus: consensus,
    alert: alert,
    quality: quality,
};
```

### Tail Expression Return
Tail expression return allows a value-producing block to omit the final
`return` when the last expression is already in value position.

**Example:**
```semantic
fn abs_delta(x: f64, y: f64) -> f64 {
    let d = x - y;
    abs(d)
}
```

**Canonical lowering:**
```semantic
fn abs_delta(x: f64, y: f64) -> f64 {
    let d = x - y;
    return abs(d);
}
```

### Quad Predicate Vocabulary
Readable quad predicates may be surfaced as aliases for explicit comparisons.

**Proposed forms:**
- `known(x)` lowers to `x != N`
- `unknown(x)` lowers to `x == N`
- `conflict(x)` lowers to `x == S`

**Optional companion alias:**
- `x is S` lowers to `x == S`

**Example:**
```semantic
if conflict(camera_state) {
    return S;
}
```

**Canonical lowering:**
```semantic
if camera_state == S {
    return S;
}
```

## Alternatives Considered
- Keep only explicit core syntax and avoid all sugar.
- Add `unless` as a general negated guard keyword.
- Add `when let` before the simpler record and tail-expression sugar.
- Introduce a pipeline operator before pinning down quad predicate vocabulary.

These alternatives were rejected or deferred because they either add more
surface than they remove, or they complicate the lowering story before the
lowest-risk improvements land.

## Rollout
Suggested implementation order:
1. field punning
2. tail expression return
3. quad predicate vocabulary aliases

Only after those are stable should the repository consider:
- `when let`
- `unless`
- pipeline operator `|>`

## Open Questions
- Should field punning be allowed in record update blocks on day one, or added
after record literals only?
- Should tail expressions apply only in explicit value position, or also inside
arm bodies where the context is already value-producing?
- Should `is` remain a short alias for `==` against `S`, or stay deferred until
the quad predicate vocabulary settles?

## Deferred Ideas
- `when let`
- `unless`
- pipeline operator `|>`

These may remain roadmap candidates, but they are not required for the first
pass of this sugar track.

## Non-Goals
This document does not:
- change the runtime model
- introduce dynamic typing
- introduce reflection
- relax verifier admission
- add implicit truthiness
- widen quad into bool
- claim implementation status

## Related Docs
- [`docs/language/semantic_language_experience.md`](semantic_language_experience.md)
- [`docs/language/semantic_code_style_principles.md`](semantic_code_style_principles.md)
- [`docs/language/semantic_quad_surface.md`](semantic_quad_surface.md)
- [`docs/roadmap/language_maturity/semantic_sugar_track_roadmap.md`](../roadmap/language_maturity/semantic_sugar_track_roadmap.md)
