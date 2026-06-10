# R12 UI Projection Artifact Shape Contract

## 1. Purpose

Define the future shape of renderer-neutral projection artifacts.
This document does not authorize implementation.
This document does not authorize Rust types.
This document does not authorize renderer/backend/runtime/layout/draw/event behavior.

## 2. Current Factual State

List:

* UiAst exists.
* UiIr exists.
* validate_ast exists.
* lower_ast_to_ir exists.
* validate_ir exists.
* renderer adapter contract exists.
* renderer-neutral projection contract exists.
* no projection artifact type exists.
* no projection implementation exists.
* no renderer adapter implementation exists.
* no renderer backend exists.
* no layout engine exists.
* no draw command layer exists.
* no event loop exists.
* no Workbench/Studio shell exists.

## 3. Artifact Shape Definition

A future projection artifact may be a backend-neutral, immutable projection of validated UiIr.
It may describe UI intent in a renderer-independent form.

It is not:

* Semantic truth
* verifier admission
* Local Admission Guard admission
* runtime execution
* layout result
* draw command stream
* GPU command stream
* event loop state
* widget framework
* application shell

## 4. Future Top-Level Artifact Fields

Future artifact may contain:

* artifact id or local projection id
* source UiIr reference or trace
* projected nodes collection
* optional diagnostics or warnings
* optional metadata
* optional version/epoch marker

All fields are conceptual only.
No type is implemented here.

## 5. Future Projected Node Fields

Future projected node may contain:

* projected node id
* source UiIrNodeId
* projected node kind
* parent projected node id
* child projected node ids
* neutral property references
* neutral action references
* neutral effect boundary references
* optional source trace reference

No projected node type is implemented here.

## 6. Future Projected Node Kind Boundary

Future projected node kinds may mirror current neutral IR vocabulary:

* Root
* Element
* Text
* Fragment
* PropertyCarrier if separately gated
* ActionCarrier if separately gated
* EffectBoundaryMarker if separately gated

This vocabulary is not final and must not imply renderer semantics.

## 7. Property Reference Boundary

Future property references:

* may point to renderer-neutral property records
* must not be CSS
* must not be DOM attributes
* must not be native widget properties
* must not be renderer bindings
* must not execute effects
* must not imply capability admission

Property semantics require separate contract.

## 8. Action Reference Boundary

Future action references:

* may point to renderer-neutral action records
* must not be event handlers
* must not be command dispatch
* must not execute code
* must not grant capability
* must not imply runtime readiness

Action semantics require separate contract.

## 9. Effect Boundary Reference Boundary

Future effect boundary references:

* may mark places where effects are visually or structurally represented
* must not admit effects
* must not execute effects
* must not replace Local Admission Guard
* must not imply verifier admission

Effect boundary semantics require separate contract.

## 10. Trace Reference Boundary

Future trace references:

* may point back to UiIr nodes
* may point to future source marks
* may point to validation/lowering diagnostics
* may support explainability
* must not become truth
* must not become admission

Trace shape requires separate contract.

## 11. Immutability / Cache Posture

Future projection artifact should be treated as projection/cache.
It should not mutate UiIr.
It should not mutate repository truth.
It should not own Semantic state.
It should be reproducible from same input/config if deterministic projection is later implemented.

## 12. Relationship To Renderer Adapter

State:

* renderer adapter contract defines the boundary.
* renderer-neutral projection contract defines the artifact category.
* this document defines possible artifact shape.
* adapter implementation is not authorized here.
* projection implementation is not authorized here.
* artifact types are not authorized here.

## 13. Relationship To Layout / Draw / Events

State:

* artifact is not layout.
* artifact does not compute positions/sizes.
* artifact is not draw commands.
* artifact does not create GPU resources.
* artifact is not event loop.
* artifact does not handle input.
* layout/draw/event boundaries require separate contracts.

## 14. Authority Boundary

UI may display truth. UI does not become truth.

State:

* artifact does not own truth.
* artifact does not own verifier admission.
* artifact does not own Local Admission Guard.
* artifact does not own runtime readiness.
* artifact does not own release readiness.

## 15. State Boundary

UI state is projection/cache, not semantic state.

State:

* artifact state is not Semantic state.
* artifact state is not runtime state.
* artifact state is not repository truth.
* artifact state is not Workbench/Studio state.
* artifact must not mutate repository truth.

## 16. Quad-State Boundary

State:

* future artifact must preserve N/F/T/S meaning where applicable.
* unknown must not be dropped.
* conflict must not be flattened into ordinary failure.
* denied must not be treated as false.
* not admitted must not be treated as invalid source.
* visual representation of Quad-state requires a separate contract.

## 17. Forbidden Behavior

List:

* no implementation
* no Rust types
* no code changes
* no dependency addition
* no projection code
* no renderer backend
* no WGPU
* no winit
* no Tauri
* no browser DOM implementation
* no native widget toolkit
* no layout engine
* no draw commands
* no event loop
* no widget framework
* no parser/verifier/VM/runtime integration
* no Workbench/Studio
* no release/stable claim

## 18. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| projection artifact shape contract | Implemented | ADMITTED | contract definition |
| projection artifact type | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| projected node type | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| projected node identity | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| property references | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| action references | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| effect boundary references | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| trace references | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| artifact metadata | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| layout fields | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| draw commands | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| event loop data | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| WGPU backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| winit event loop | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| parser/verifier/VM/runtime integration | Absent | FORBIDDEN | verifier owns admission |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 19. Future Gates

List:

* R12-UI-PROJECTION-ARTIFACT-SHAPE-AUDIT
* R12-UI-PROJECTION-ARTIFACT-SEED-APPROVAL
* R12-UI-PROJECTION-ARTIFACT-SEED
* R12-UI-PROJECTION-TRACE-CONTRACT
* R12-UI-PROPERTY-SEMANTICS-CONTRACT
* R12-UI-ACTION-BOUNDARY-CONTRACT
* R12-UI-EFFECT-BOUNDARY-CONTRACT
* R12-UI-LAYOUT-BOUNDARY-CONTRACT
* R12-UI-DRAW-COMMAND-BOUNDARY-CONTRACT
* R12-UI-EVENT-BOUNDARY-CONTRACT

## 20. Final Decision

Final decision:
READY — A FUTURE PROJECTION ARTIFACT MAY HAVE A BACKEND-NEUTRAL SHAPE, BUT NO RUST TYPES, PROJECTION IMPLEMENTATION, RENDERER, LAYOUT, DRAW, EVENT, RUNTIME, OR APPLICATION SHELL IMPLEMENTATION IS AUTHORIZED
