# R12 UI Renderer-Neutral Projection Contract

## 1. Purpose

Define the future renderer-neutral projection artifact boundary.
This document does not authorize implementation.
This document does not authorize renderer/backend/runtime/layout/draw/event behavior.
This document does not authorize Workbench or Semantic Studio UI shell work.

## 2. Current Factual State

List:

* UiAst exists.
* UiIr exists.
* validate_ast exists.
* lower_ast_to_ir exists.
* validate_ir exists.
* renderer adapter contract exists.
* UiIr validation is local and structural.
* UiIr is inert.
* UiIr is not renderer-ready.
* UiIr is not runtime-ready.
* no renderer-neutral projection type exists.
* no renderer adapter implementation exists.
* no renderer backend exists.
* no layout engine exists.
* no draw command layer exists.
* no event loop exists.
* no Workbench/Studio shell exists.

## 3. Projection Definition

A future renderer-neutral projection is a backend-independent representation derived from validated UI IR.
It may describe renderer-facing intent without choosing a backend.

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

## 4. Input Boundary

Future projection input may be:

* UiIr
* validated UiIr wrapper
* future renderer adapter input

Input must not imply:

* verifier admission
* runtime readiness
* renderer readiness
* layout readiness
* release readiness

## 5. Output Boundary

Future projection output may contain backend-neutral structural projection records.

It must not contain:

* GPU commands
* draw commands
* layout coordinates
* platform window handles
* event loop handles
* renderer resource handles
* host effects
* capability admission
* Semantic state

No output structure is implemented here.

## 6. Backend-Neutral Shape

Future projection may describe:

* projected node identity
* projected node kind
* structural parent/child relationships
* renderer-neutral properties
* renderer-neutral actions as inert handles
* renderer-neutral effect boundaries as inert markers
* optional source trace references

All items remain future-only and not implemented here.

## 7. Property Boundary

State:

* Property is renderer-neutral vocabulary only.
* Property is not CSS.
* Property is not DOM attribute.
* Property is not native widget property.
* Property is not renderer binding.
* Property semantics require a separate contract.

## 8. Action Boundary

State:

* Action is renderer-neutral vocabulary only.
* Action is not execution.
* Action is not event handler.
* Action is not command dispatch.
* Action is not capability grant.
* Action semantics require a separate contract.

## 9. EffectBoundary Boundary

State:

* EffectBoundary is renderer-neutral structural vocabulary only.
* EffectBoundary is not capability admission.
* EffectBoundary is not effect execution.
* EffectBoundary is not Local Admission Guard.
* EffectBoundary semantics require a separate contract.

## 10. Relationship To Renderer Adapter

State:

* renderer adapter contract defines the boundary.
* renderer-neutral projection contract defines a possible future artifact.
* adapter implementation is not authorized here.
* projection implementation is not authorized here.
* projection must not call renderer/backend/runtime/verifier/VM/parser.
* projection must not mutate UiIr.
* projection must not create Semantic truth.

## 11. Relationship To Layout

State:

* projection is not layout.
* projection does not compute positions.
* projection does not compute sizes.
* projection does not compute constraints.
* projection does not compute z-order.
* layout requires separate boundary contract.

## 12. Relationship To Draw Commands

State:

* projection is not draw commands.
* projection does not create draw lists.
* projection does not create GPU buffers.
* projection does not create render passes.
* projection does not create shaders.
* draw command boundary requires separate contract.

## 13. Relationship To Events

State:

* projection is not an event loop.
* projection does not handle input events.
* projection does not dispatch commands.
* projection does not mutate runtime state.
* event boundary requires separate contract.

## 14. Authority Boundary

UI may display truth. UI does not become truth.

State:

* projection does not own truth.
* projection does not own verifier admission.
* projection does not own Local Admission Guard.
* projection does not own runtime readiness.
* projection does not own release readiness.

## 15. State Boundary

UI state is projection/cache, not semantic state.

State:

* projection state is not Semantic state.
* projection state is not runtime state.
* projection state is not repository truth.
* projection state is not Workbench/Studio state.
* projection must not mutate repository truth.

## 16. Quad-State Boundary

State:

* future projection must preserve N/F/T/S meaning where applicable.
* unknown must not be dropped.
* conflict must not be flattened into ordinary failure.
* denied must not be treated as false.
* not admitted must not be treated as invalid source.
* visual representation of Quad-state requires a separate contract.

## 17. Forbidden Behavior

List:

* no implementation
* no code changes
* no dependency addition
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
| renderer-neutral projection contract | Implemented | ADMITTED | contract definition |
| projection artifact implementation | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| projected node identity | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| renderer-neutral properties | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| renderer-neutral actions | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| renderer-neutral effect boundaries | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| property semantics | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| action execution | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| effect admission | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| layout engine | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| draw commands | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| event loop | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| WGPU backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| winit event loop | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| parser/verifier/VM/runtime integration | Absent | FORBIDDEN | verifier owns admission |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 19. Future Gates

List:

* R12-UI-RENDERER-NEUTRAL-PROJECTION-AUDIT
* R12-UI-PROJECTION-ARTIFACT-SHAPE-CONTRACT
* R12-UI-PROPERTY-SEMANTICS-CONTRACT
* R12-UI-ACTION-BOUNDARY-CONTRACT
* R12-UI-EFFECT-BOUNDARY-CONTRACT
* R12-UI-LAYOUT-BOUNDARY-CONTRACT
* R12-UI-DRAW-COMMAND-BOUNDARY-CONTRACT
* R12-UI-EVENT-BOUNDARY-CONTRACT
* R12-UI-WGPU-BACKEND-POSTURE
* R12-UI-WINIT-EVENT-LOOP-POSTURE

## 20. Final Decision

Final decision:
READY — A FUTURE RENDERER-NEUTRAL PROJECTION MAY DESCRIBE BACKEND-INDEPENDENT UI INTENT, BUT NO PROJECTION ARTIFACT, RENDERER, LAYOUT, DRAW, EVENT, RUNTIME, OR APPLICATION SHELL IMPLEMENTATION IS AUTHORIZED
