# R12 UI Renderer Adapter Contract

## 1. Purpose

Define the future boundary between validated Semantic UI IR and renderer/backend layers.
This document does not authorize implementation.
This document does not authorize WGPU, winit, layout, draw commands, event loop, runtime, Workbench, or Semantic Studio.

## 2. Current Factual State

List:

* UiAst exists.
* UiIr exists.
* validate_ast exists.
* lower_ast_to_ir exists.
* validate_ir exists.
* UiIr validation is local and structural.
* UiIr is inert.
* UiIr is not renderer-ready.
* UiIr is not runtime-ready.
* No renderer adapter exists.
* No WGPU/winit backend exists.
* No layout engine exists.
* No draw command layer exists.
* No event loop exists.
* No Workbench/Studio UI shell exists.

## 3. Renderer Adapter Definition

A future renderer adapter may translate validated UiIr into renderer-facing projection data.
It is not:

* Semantic truth
* verifier admission
* runtime execution
* layout engine
* draw command generation
* GPU command generation
* event loop
* widget framework
* application shell

## 4. Input Boundary

Future adapter input may be:

* UiIr
* or a future validated UiIr wrapper
* or future renderer-neutral UI projection input

Input must not imply:

* verifier admission
* runtime readiness
* renderer readiness
* release readiness

## 5. Output Boundary

Future adapter output may be a renderer-neutral projection artifact.
It must not be:

* draw commands
* GPU commands
* layout result
* event loop state
* host effects
* capability admission
* Semantic state

No output structure is implemented here.

## 6. Renderer-Neutral Requirement

The first renderer adapter contract must remain backend-neutral.
It must not choose:

* WGPU
* winit
* Tauri
* browser DOM
* native widget toolkit
* platform-specific renderer

Backend choice requires a separate gate.

## 7. Relationship To UiIr Validation

State:

* validate_ir checks structure only.
* validate_ir success does not imply renderer readiness.
* future adapter may require validated UiIr only after a separate gate.
* adapter must not call verifier, VM, runtime, parser, or backend.
* adapter must not mutate UiIr.
* adapter must not create Semantic truth.

## 8. EffectBoundary / Action / Property Boundary

State:

* EffectBoundary is structural vocabulary, not capability admission.
* Action is structural vocabulary, not execution.
* Property is structural vocabulary, not renderer binding.
* Future meaning for these requires separate contracts.

## 9. Authority Boundary

UI may display truth. UI does not become truth.

State:

* renderer adapter does not own truth.
* renderer adapter does not own verifier admission.
* renderer adapter does not own Local Admission Guard.
* renderer adapter does not own runtime readiness.
* renderer adapter does not own release readiness.

## 10. State Boundary

UI state is projection/cache, not semantic state.

State:

* renderer adapter state is not Semantic state.
* renderer adapter state is not runtime state.
* renderer adapter state is not repository truth.
* renderer adapter state is not Workbench/Studio state.

## 11. Quad-State Boundary

State:

* future renderer-facing projection must preserve N/F/T/S meaning where applicable.
* unknown must not be dropped.
* conflict must not be flattened into ordinary failure.
* denied must not be treated as false.
* not admitted must not be treated as invalid source.
* visual representation of Quad-state requires a separate contract.

## 12. Forbidden Behavior

List:

* no implementation
* no code changes
* no dependency addition
* no WGPU
* no winit
* no renderer backend
* no layout engine
* no draw commands
* no event loop
* no widget framework
* no parser/verifier/VM/runtime integration
* no Workbench/Studio
* no release/stable claim

## 13. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| renderer adapter contract | Implemented | ADMITTED | contract definition |
| renderer adapter implementation | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| renderer-neutral projection | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| WGPU backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| winit event loop | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| layout engine | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| draw commands | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| EffectBoundary semantics | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Action execution | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Property renderer binding | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| parser/verifier/VM/runtime integration | Absent | FORBIDDEN | verifier owns admission |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 14. Future Gates

List:

* R12-UI-RENDERER-ADAPTER-AUDIT
* R12-UI-RENDERER-NEUTRAL-PROJECTION-CONTRACT
* R12-UI-LAYOUT-BOUNDARY-CONTRACT
* R12-UI-DRAW-COMMAND-BOUNDARY-CONTRACT
* R12-UI-EFFECT-BOUNDARY-CONTRACT
* R12-UI-WGPU-BACKEND-POSTURE
* R12-UI-WINIT-EVENT-LOOP-POSTURE
* R12-UI-WORKBENCH-SHELL-CONTRACT
* R12-UI-STUDIO-SHELL-CONTRACT

## 15. Final Decision

Final decision:
READY — A FUTURE RENDERER ADAPTER MAY BE DEFINED AS A BACKEND-NEUTRAL PROJECTION BOUNDARY, BUT NO RENDERER, LAYOUT, DRAW, EVENT, RUNTIME, OR APPLICATION SHELL IMPLEMENTATION IS AUTHORIZED
