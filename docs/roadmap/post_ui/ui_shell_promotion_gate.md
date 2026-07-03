# ui-shell-kit Promotion Gate

## Status

`ui-shell-kit` is non-production by default.

Promotion is possible only through an explicit gate.

This document defines the gate.

It does not perform promotion.

## Purpose

The promotion gate exists to prevent uncontrolled UI/runtime creep while allowing successful experimental work to mature.

The intended balance is:

```text
bold experimental UI shell work
+ strict production admission boundary
```

## Boundary

The promotion gate belongs to the POST-UI roadmap.

It does not connect `ui-shell-kit` to `prom-ui`.

It does not connect `ui-shell-kit` to Workbench.

It does not modify Cargo workspace membership.

It does not modify runtime, verifier, VM, or SemCode.

## Promotion Model

```text
experimental evidence
  ↓
contract stability
  ↓
explicit review
  ↓
decision record
  ↓
limited extraction / migration
```

`ui-shell-kit` remains experimental unless the gate is passed.

## Promotion Targets

Allowed future target areas may include:

- `prom-ui`
- Workbench
- documentation examples
- demo-only reference scenes
- shared UI shell primitives

These are candidate targets only. None is approved by this document.

## Required Evidence Before Promotion

Promotion must require evidence from prior issues:

- `#1311` — experimental boundary exists
- `#1312` — calculator shell contract exists
- `#1313` — draw-command snapshot policy exists
- `#1314` — motion phase contract exists

Promotion also requires:

- runnable examples;
- deterministic snapshots or dumps;
- documented non-goals;
- known boundary risks;
- clear extraction scope.

## What May Be Promoted

Only narrow, reviewed pieces may be considered, such as:

- geometry helpers;
- layout primitives;
- draw-command model;
- snapshot helpers;
- theme tokens;
- focus primitives;
- accessibility model;
- calculator scene as demo reference.

These are candidates, not approved migrations.

## What Must Not Be Promoted Yet

The following must not be promoted automatically:

- entire `ui-shell-kit` crate;
- experimental scene internals;
- calculator controller as production runtime behavior;
- renderer backend assumptions;
- motion model as final animation system;
- Workbench dependency;
- runtime capabilities;
- VM/verifier/SemCode-facing behavior.

## Gate Criteria

Minimum gate criteria:

1. Boundary reviewed.
2. Contract reviewed.
3. Snapshot evidence reviewed.
4. Motion evidence reviewed.
5. No production wiring hidden in docs PRs.
6. Extraction target identified.
7. API owner identified.
8. Non-goals preserved.
9. Migration risk documented.
10. Separate implementation issue created.

## Required Decision Record

Before promotion, a future decision document must exist:

`docs/roadmap/post_ui/ui_shell_promotion_decision_<name>.md`

The decision record must include:

- source component;
- target component;
- reason for promotion;
- evidence links;
- accepted risks;
- rejected alternatives;
- non-goals;
- rollback plan.

## Review Checklist

- Does this promotion preserve `ui-shell-kit` isolation until implementation starts?
- Is the target layer correct?
- Is the promoted piece narrow?
- Are runtime/verifier/VM/SemCode untouched?
- Are capabilities untouched?
- Are examples and snapshots stable?
- Is there a rollback path?
- Is there a separate implementation issue?

## Relationship to Prior POST-UI Issues

- `#1310` — parent POST-UI track
- `#1311` — experimental boundary
- `#1312` — calculator shell contract
- `#1313` — draw-command snapshot policy
- `#1314` — phased motion evidence model

`#1315` consumes evidence from `#1311` through `#1314`.

## In Scope

- define promotion criteria;
- define required evidence;
- define allowed future target areas;
- require a decision record;
- prevent implicit production wiring;
- preserve experimental-track isolation.

## Hard Non-goals

- no automatic promotion;
- no production UI wiring;
- no verifier changes;
- no VM changes;
- no SemCode changes;
- no runtime capability widening;
- no Workbench implementation dependency;
- no renderer backend decision;
- no GPU/shader backend contract;
- no browser/mobile target;
- no claim that `ui-shell-kit` is the final Semantic UI framework.

## Acceptance Criteria

- promotion is gated by explicit criteria;
- the gate references boundary evidence from `#1311`;
- the gate references calculator contract evidence from `#1312`;
- the gate references snapshot policy evidence from `#1313`;
- the gate references motion evidence from `#1314`;
- no production wiring happens implicitly;
- the decision path to `prom-ui` / Workbench is clear and separate;
- the document links back to `#1310`;
- no code or production wiring changes are introduced.
