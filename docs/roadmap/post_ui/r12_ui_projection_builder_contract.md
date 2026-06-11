# R12 UI Projection Builder Contract

## 1. Purpose

Define the future boundary for building inert renderer-neutral projection artifacts from UI IR.
This document does not authorize implementation.
This document does not authorize Rust code.
This document does not authorize renderer/backend/layout/draw/event/runtime behavior.

## 2. Current Factual State

List:

* UiAst exists.
* UiIr exists.
* validate_ast exists.
* lower_ast_to_ir exists.
* validate_ir exists.
* UiProjectionArtifact exists.
* UiProjectedNode exists.
* projection artifact seed exists.
* projection artifact seed is inert.
* no projection builder exists.
* no `project_ir_to_projection` function exists.
* no `From<UiIr>` / `TryFrom<UiIr>` projection conversion exists.
* no renderer adapter implementation exists.
* no renderer backend exists.
* no layout engine exists.
* no draw command layer exists.
* no event loop exists.
* no Workbench/Studio shell exists.

## 3. Builder Definition

A future projection builder may transform validated `UiIr` into an inert `UiProjectionArtifact`.

The builder may eventually:

* read UiIr nodes;
* map UiIr node handles to projected node handles;
* preserve source UiIr references;
* copy structural parent/child relationships;
* produce inert projected nodes;
* attach inert property/action/effect/trace references only after separately gated semantics.

The builder is not:

* renderer adapter implementation;
* renderer backend;
* layout engine;
* draw command generator;
* event handler;
* runtime execution;
* verifier admission;
* Semantic truth.

## 4. Input Boundary

Future builder input may be:

* borrowed `&UiIr`;
* a future validated UiIr wrapper;
* a future projection builder config.

Input must not imply:

* Semantic validity;
* verifier admission;
* Local Admission Guard admission;
* runtime readiness;
* renderer readiness;
* release readiness.

## 5. Validation Relationship

State:

* `validate_ir` checks local structure only.
* `validate_ir` success does not imply renderer readiness.
* future builder should require valid structure before projection.
* whether builder calls `validate_ir` internally or requires a validated wrapper must be decided by a separate implementation gate.
* this contract does not authorize changing `validate_ir`.

## 6. Output Boundary

Future builder output may be:

* `UiProjectionArtifact`;
* or a future projection builder result type;
* or diagnostics plus artifact only if separately gated.

Output must not be:

* draw commands;
* layout result;
* GPU command stream;
* event loop state;
* renderer resource handles;
* runtime state;
* Semantic state;
* admission proof.

## 7. Determinism Contract

Future builder must be deterministic:

* same UiIr input + same builder config = same projection artifact or same diagnostics.
* no wall-clock time.
* no randomness.
* no file I/O.
* no network access.
* no host effects.
* no command execution.
* no global mutable state.

## 8. Mapping Contract

Future builder may map:

* UiIrNodeId to UiProjectedNodeId;
* UiIrNodeKind to UiProjectedNodeKind;
* UiIr parent/children handles to projected parent/children handles;
* UiIr source references to projection source references.

Mapping must be explicit and tested before code.
Mapping must not imply renderer semantics.

## 9. IR Kind Mapping Boundary

Future mapping may eventually handle:

* Root
* Element
* Text
* Fragment
* Property
* Action
* EffectBoundary

Boundary:

* Property must not become renderer binding without separate contract.
* Action must not become execution/event handler without separate contract.
* EffectBoundary must not become capability admission without separate contract.

## 10. Diagnostics Boundary

Future builder diagnostics may cover:

* invalid IR structure if builder validates internally;
* unsupported IR node kind;
* unsupported Property / Action / EffectBoundary mapping;
* inconsistent handles;
* missing source references if required later.

Diagnostics must not panic.
Diagnostics must not execute effects.
Diagnostics must not call renderer/backend/runtime/verifier/VM/parser.

## 11. Traceability Boundary

Future builder may preserve traceability:

* source UiIrNodeId reference;
* optional future source mark;
* optional lowering/validation trace reference;
* optional projection trace reference.

Trace does not become truth.
Trace does not become admission.
Trace does not become renderer readiness.

## 12. Authority Boundary

UI may display truth. UI does not become truth.

State:

* projection builder does not own truth.
* projection builder does not own verifier admission.
* projection builder does not own Local Admission Guard.
* projection builder does not own runtime readiness.
* projection builder does not own renderer readiness.
* projection builder does not own release readiness.

## 13. State Boundary

UI state is projection/cache, not semantic state.

State:

* projection builder output is projection/cache.
* projection builder state is not Semantic state.
* projection builder state is not runtime state.
* projection builder state is not repository truth.
* projection builder state is not Workbench/Studio state.
* projection builder must not mutate repository truth.

## 14. Quad-State Boundary

State:

* future builder must preserve N/F/T/S meaning where applicable.
* unknown must not be dropped.
* conflict must not be flattened into ordinary failure.
* denied must not be treated as false.
* not admitted must not be treated as invalid source.
* visual representation of Quad-state requires a separate contract.

## 15. Forbidden Behavior

List:

* no implementation
* no Rust code
* no dependency addition
* no projection.rs change
* no builder function
* no `project_ir_to_projection`
* no `from_ir`
* no `From<UiIr>` / `TryFrom<UiIr>`
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

## 16. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| projection builder contract | Implemented | ADMITTED | boundary contract |
| projection builder implementation | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| `project_ir_to_projection` | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| `From<UiIr>` / `TryFrom<UiIr>` | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| UiIr to UiProjectedNode mapping | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| builder diagnostics | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| builder traceability | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Property mapping | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Action mapping | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| EffectBoundary mapping | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| validate_ir integration | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| renderer adapter | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| layout engine | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| draw commands | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| event loop | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| WGPU/winit backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| parser/verifier/VM/runtime integration | Absent | FORBIDDEN | verifier owns admission |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 17. Future Gates

List:

* R12-UI-PROJECTION-BUILDER-CONTRACT-AUDIT
* R12-UI-PROJECTION-BUILDER-SEED-APPROVAL
* R12-UI-PROJECTION-BUILDER-SEED
* R12-UI-PROJECTION-BUILDER-DIAGNOSTICS-CONTRACT
* R12-UI-PROJECTION-TRACE-CONTRACT
* R12-UI-PROPERTY-SEMANTICS-CONTRACT
* R12-UI-ACTION-BOUNDARY-CONTRACT
* R12-UI-EFFECT-BOUNDARY-CONTRACT
* R12-UI-LAYOUT-BOUNDARY-CONTRACT
* R12-UI-DRAW-COMMAND-BOUNDARY-CONTRACT
* R12-UI-EVENT-BOUNDARY-CONTRACT

## 18. Final Decision

Final decision:
READY — A FUTURE PROJECTION BUILDER MAY MAP VALIDATED UI IR INTO AN INERT PROJECTION ARTIFACT, BUT NO BUILDER CODE, RENDERER, LAYOUT, DRAW, EVENT, RUNTIME, OR APPLICATION SHELL IMPLEMENTATION IS AUTHORIZED
