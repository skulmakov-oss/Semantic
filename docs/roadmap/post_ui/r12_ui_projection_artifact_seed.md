# R12 UI Projection Artifact Seed

## 1. Purpose

Record the first inert Rust seed for renderer-neutral projection artifact types.
This document does not authorize projection implementation from UiIr.
This document does not authorize renderer/backend/layout/draw/event/runtime behavior.

## 2. Current Factual State

List:

* UiAst exists.
* UiIr exists.
* validate_ast exists.
* lower_ast_to_ir exists.
* validate_ir exists.
* renderer adapter contract exists.
* renderer-neutral projection contract exists.
* projection artifact shape contract exists.
* no renderer adapter implementation exists.
* no projection builder exists.
* no renderer backend exists.
* no layout engine exists.
* no draw command layer exists.
* no event loop exists.
* no Workbench/Studio shell exists.

## 3. Authorized Seed Scope

This seed may introduce:

* local projection artifact id
* projected node id
* projected node kind
* inert projection artifact container
* inert projected node container
* inert property/action/effect/trace references
* local unit tests

This seed must remain inert.

## 4. Not Authorized

List:

* no projection from UiIr
* no renderer adapter implementation
* no layout
* no draw commands
* no event loop
* no backend choice
* no WGPU/winit/Tauri
* no verifier/runtime/parser integration
* no Workbench/Studio shell

## 5. Authority Boundary

UI may display truth. UI does not become truth.

State:

* projection artifact does not own truth.
* projection artifact does not own verifier admission.
* projection artifact does not own Local Admission Guard.
* projection artifact does not own runtime readiness.
* projection artifact does not own renderer readiness.

## 6. State Boundary

UI state is projection/cache, not semantic state.

State:

* projection artifact state is not Semantic state.
* projection artifact state is not runtime state.
* projection artifact state is not repository truth.
* projection artifact state is not Workbench/Studio state.

## 7. Quad-State Boundary

State:

* this seed does not implement Quad-state visual semantics.
* future projection overlays must preserve N/F/T/S.
* unknown must not be dropped.
* conflict must not be flattened into ordinary failure.
* denied must not be treated as false.
* not admitted must not be treated as invalid source.

## 8. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| projection artifact seed | Implemented | ADMITTED | inert local seed |
| projection artifact types | Implemented | ADMITTED | inert local types |
| projected node types | Implemented | ADMITTED | inert local types |
| property/action/effect/trace references | Implemented | ADMITTED | inert local types |
| projection builder from UiIr | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| renderer adapter | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| layout engine | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| draw commands | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| event loop | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| WGPU/winit backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| parser/verifier/VM/runtime integration | Absent | FORBIDDEN | verifier owns admission |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 9. Future Gates

List:

* R12-UI-PROJECTION-ARTIFACT-SEED-AUDIT
* R12-UI-PROJECTION-BUILDER-CONTRACT
* R12-UI-PROJECTION-BUILDER-SEED
* R12-UI-PROJECTION-TRACE-CONTRACT
* R12-UI-PROPERTY-SEMANTICS-CONTRACT
* R12-UI-ACTION-BOUNDARY-CONTRACT
* R12-UI-EFFECT-BOUNDARY-CONTRACT
* R12-UI-LAYOUT-BOUNDARY-CONTRACT
* R12-UI-DRAW-COMMAND-BOUNDARY-CONTRACT
* R12-UI-EVENT-BOUNDARY-CONTRACT

## 10. Final Decision

Final decision:
READY — THE FIRST PROJECTION ARTIFACT SEED MAY ADD INERT RUST TYPES, BUT NO PROJECTION BUILDER, RENDERER, LAYOUT, DRAW, EVENT, RUNTIME, OR APPLICATION SHELL IMPLEMENTATION IS AUTHORIZED
