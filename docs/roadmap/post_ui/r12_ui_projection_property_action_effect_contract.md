# R12 UI Projection Property / Action / EffectBoundary Contract

## Purpose

This document defines the Property / Action / EffectBoundary contract for the R12 UI Projection Builder.

It records current implemented projection behavior and separates inert projection carriers from renderer binding, event dispatch, runtime effects, verifier admission, and capability admission.

This document does not implement new behavior.

Property / Action / EffectBoundary contract prevents inert projection carriers from becoming execution semantics.

## Truth classification legend

| Classification | Meaning |
|---|---|
| IMPLEMENTED | Present in current main code and locally verifiable. |
| DOCUMENTED | Present in docs/policy but not necessarily implemented. |
| AUTHORIZED_FOR_FUTURE | May be implemented only through a later explicit gate. |
| ABSENT | Not present in current main. |
| FORBIDDEN | Must not be introduced in this contract. |

## Current factual state

Current main:
8c2af636f859e1c33c097fbf39028e5b73006810

Implemented:
- Property is projected as inert PropertyCarrier if verified in current code.
- Action is projected as inert ActionCarrier if verified in current code.
- EffectBoundary is projected as inert EffectBoundaryMarker if verified in current code.
- project_ir_to_projection validates UiIr before projection.
- projection artifact remains inert.
- diagnostics seed and traceability seed remain local to projection.rs.

Absent:
- no renderer property binding.
- no event handler dispatch.
- no capability admission.
- no runtime effect execution.
- no verifier/VM integration.
- no Workbench/Studio integration.
- no layout/draw/event semantics.

## Evidence Matrix

| Claim | Classification | Evidence | Status |
|---|---|---|---|
| PropertyCarrier exists | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| ActionCarrier exists | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| EffectBoundaryMarker exists | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| Property maps to inert carrier | IMPLEMENTED | UiIrNodeKind::Property maps to UiProjectedNodeKind::PropertyCarrier | PASS |
| Action maps to inert carrier | IMPLEMENTED | UiIrNodeKind::Action maps to UiProjectedNodeKind::ActionCarrier | PASS |
| EffectBoundary maps to inert marker | IMPLEMENTED | UiIrNodeKind::EffectBoundary maps to UiProjectedNodeKind::EffectBoundaryMarker | PASS |
| renderer binding exists | ABSENT / FORBIDDEN | No wgpu/renderer integrations | PASS |
| event handler dispatch exists | ABSENT / FORBIDDEN | No event loop or dispatch logic | PASS |
| capability admission exists | ABSENT / FORBIDDEN | No admission gate calls in projection | PASS |
| runtime effect execution exists | ABSENT / FORBIDDEN | No Runtime execution logic | PASS |
| verifier/VM integration exists | ABSENT / FORBIDDEN | No Verifier/VM logic | PASS |
| Workbench/Studio integration exists | ABSENT / FORBIDDEN | No Workbench integration in projection | PASS |
| future property/action/effect seed is allowed | AUTHORIZED_FOR_FUTURE | R12-UI-PROJECTION-PROPERTY-ACTION-EFFECT-SEED | PASS |

## Property contract

Current Property behavior:
Property is projection metadata carried as inert PropertyCarrier.

Property may describe:
- projected UI structure metadata;
- source-level property shape;
- inert value references if current model supports them.

Property must not:
- bind to renderer resources;
- mutate UI runtime state;
- read host state;
- trigger layout;
- trigger drawing;
- trigger events;
- imply verifier admission;
- imply capability admission.

PropertyCarrier is data, not binding.

## Action contract

Current Action behavior:
Action is projection metadata carried as inert ActionCarrier.

Action may describe:
- projected UI action shape;
- source-level action identity or structure if current model supports it;
- inert association with projected nodes.

Action must not:
- execute;
- dispatch;
- register event handlers;
- call runtime;
- call VM;
- call verifier;
- mutate Semantic state;
- mutate host state;
- imply capability admission;
- imply Workbench/Studio command execution.

ActionCarrier is data, not an event handler.

## EffectBoundary contract

Current EffectBoundary behavior:
EffectBoundary is projection metadata carried as inert EffectBoundaryMarker.

EffectBoundary may describe:
- where effect-related UI structure is represented;
- a marker for future boundary analysis.

EffectBoundary must not:
- authorize effects;
- execute effects;
- grant capabilities;
- validate capabilities;
- replace verifier admission;
- replace runtime effect boundary;
- become Workbench/Studio trust boundary;
- become release readiness signal.

EffectBoundaryMarker is data, not capability admission.

## Current projection boundary

The current projection builder may carry Property / Action / EffectBoundary nodes into an inert projection artifact.

It does not interpret them as renderer bindings, event handlers, or capability gates.

Projection can preserve structure.
Projection cannot authorize execution.

## Future seed boundary

Future gate:
R12-UI-PROJECTION-PROPERTY-ACTION-EFFECT-SEED

Allowed future scope:
- add focused tests proving Property / Action / EffectBoundary remain inert;
- add small accessors if current carriers lack safe inspection methods;
- add stable classification helpers if needed;
- preserve project_ir_to_projection behavior;
- preserve inert projection artifact boundary.

Not allowed unless separately authorized:
- renderer property binding;
- event dispatch;
- action handler registration;
- capability admission;
- runtime effect execution;
- verifier/VM integration;
- Workbench/Studio commands;
- dependency additions;
- broad UI framework behavior.

## Wishful claim guard

The following claims are explicitly not true in current main:

- PropertyCarrier is a renderer binding.
- ActionCarrier is an event handler.
- EffectBoundaryMarker is capability admission.
- Projected actions can execute.
- Projected effect boundaries authorize effects.
- Projection builder dispatches events.
- Projection builder mutates runtime or Semantic state.
- Projection builder integrates with Workbench/Studio commands.

These are absent or forbidden until separate explicit gates.

## Authority boundary

UI may display truth. UI does not become truth.

Property / Action / EffectBoundary projection may describe UI structure.

It does not define Semantic truth.
It does not define verifier admission.
It does not define capability admission.
It does not define runtime state.
It does not define renderer readiness.

## Quad-state boundary

Unknown must not be dropped by Property / Action / EffectBoundary projection.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.
Property / Action / EffectBoundary projection must not collapse N/F/T/S meaning.

No dedicated Quad-state Property / Action / EffectBoundary semantics are implemented in current projection builder.

## Explicit non-scope

No implementation in this PR.
No source changes.
No projection.rs changes.
No model.rs changes.
No validation.rs changes.
No lowering.rs changes.
No lib.rs changes.
No Cargo.toml / Cargo.lock changes.
No dependency additions.
No renderer/backend/layout/draw/event.
No event handler system.
No parser/verifier/VM/runtime integration.
No capability admission.
No Workbench/Studio integration.
No user-facing UI framework behavior.

## Admission Guard table

| Area | Contract state | Admission Guard classification | Status |
|---|---|---|---|
| property/action/effect contract document | Present | ADMITTED | PASS |
| PropertyCarrier | Inert data carrier | ADMITTED_WITH_BOUNDARY | PASS |
| ActionCarrier | Inert data carrier | ADMITTED_WITH_BOUNDARY | PASS |
| EffectBoundaryMarker | Inert marker | ADMITTED_WITH_BOUNDARY | PASS |
| renderer property binding | Absent | FORBIDDEN | PASS |
| event handler dispatch | Absent | FORBIDDEN | PASS |
| capability admission | Absent | FORBIDDEN | PASS |
| runtime effect execution | Absent | FORBIDDEN | PASS |
| parser/verifier/VM integration | Absent | FORBIDDEN | PASS |
| Workbench/Studio command integration | Absent | FORBIDDEN | PASS |
| source changes in this PR | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## Final decision

Final decision:
CONTRACT DEFINED — Property / Action / EffectBoundary projection is currently inert structural projection data. PropertyCarrier is not renderer binding. ActionCarrier is not an event handler. EffectBoundaryMarker is not capability admission. Any widening requires separate explicit gates.
