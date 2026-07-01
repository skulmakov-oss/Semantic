# Semantic Language Experience

Status: language-experience doctrine
Scope type: documentation only

## Purpose

This document describes the experience layer around the Semantic language:
tone, narrative, examples, and the visible way quad-heavy code should feel.

It does not change grammar, verifier admission, lowering, or runtime behavior.

## Experience Roadmap

The experience track is intentionally ordered:

1. emotional contract
2. verifier posture
3. uncertainty and conflict visibility
4. action vocabulary
5. onboarding narrative
6. positive quad patterns
7. conflict quarantine pattern
8. quad-first examples
9. documentation tone

## Emotional Contract

Semantic should feel:

- precise
- calm
- evidence-backed
- expressive without being slippery
- deterministic without being sterile

The language should not feel like a toy, a slogan, or a hidden policy engine.

## Verifier As Exoskeleton

The verifier is the exoskeleton that keeps meaning upright.

It should be described as support for expression, not as a cage around it.
The point is to keep hard boundaries visible while still allowing the language
to carry complex intent.

## Uncertainty And Conflict

Uncertainty and conflict are first-class semantic material.

- unknown is visible
- conflict is visible
- denial is visible
- quarantine is visible

The language experience should not flatten these into generic failure states.

## Freedom With A Spine

Semantic should remain flexible in how users express intent, but the spine is
fixed:

- canonical lowering
- verifier-first admission
- deterministic execution
- explicit boundaries

This is the principle that keeps the language free without making it vague.

## Action Vocabulary

The language-experience vocabulary should make intent legible:

- state
- require
- observe
- complete
- compare
- select
- admit
- quarantine

These words should help the reader understand the role of a construct before
they inspect the implementation.

## Onboarding Narrative

The first readable story about Semantic should be:

1. `bool` decides.
2. `quad` means.
3. explicit predicates control flow.
4. verifier-first keeps semantics admitted.
5. canonical lowering keeps meaning deterministic.

## Positive Quad Handling Pattern

When quad values are handled well, the code should:

- keep `N`, `F`, `T`, and `S` visible
- avoid pretending unknown is false
- avoid pretending conflict is harmless
- use explicit selection when control flow matters

## Conflict Quarantine Pattern

Conflict should not disappear.

It should be:

- visible in the surface
- quarantined where needed
- traceable in docs and diagnostics
- handled explicitly rather than hidden behind a generic no-op

## Quad-First Examples

Examples should prefer quad-shaped intent when the semantic task is about
uncertainty, evidence, or conflict.

```semantic
let boot:quad = T;
if boot==T { observe "ready"; }
```

```semantic
match boot {
    N=>{ observe "unknown"; }
    F=>{ observe "not ready"; }
    T=>{ observe "ready"; }
    S=>{ observe "conflict"; }
    _=>{ observe "fallback"; }
}
```

## Documentation Tone

Documentation should be:

- direct
- honest about current scope
- careful about current vs planned behavior
- free of marketing language
- free of accidental stability claims

Tone should tell the reader what the language does, what it does not do, and
what is still planned.

## Non-Goals

This document does not:

- add UI or Workbench policy
- redefine the verifier
- widen runtime behavior
- change parser admission
- define a release promise

## Related Docs

- [`docs/language/semantic_sugar_track.md`](semantic_sugar_track.md)
- [`docs/roadmap/language_maturity/semantic_sugar_track_roadmap.md`](../roadmap/language_maturity/semantic_sugar_track_roadmap.md)

