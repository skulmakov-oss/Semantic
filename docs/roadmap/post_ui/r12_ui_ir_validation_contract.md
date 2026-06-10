# R12 UI IR Validation Contract

## 1. Purpose
Define the minimal structural validation boundary for UiIr.
This document authorizes only local inert IR structural validation.
It does not authorize renderer/backend/runtime/layout/draw/event behavior.

## 2. Current Factual State
* UiAst exists.
* UiIr exists.
* validate_ast exists.
* lower_ast_to_ir exists.
* IR validation does not exist before this PR.
* UiIr is inert.
* UiIr is not executable.
* UiIr is not renderer-ready.
* UiIr is not runtime-ready.

## 3. IR Validation Definition
UiIr validation is a pure local structural check over UiIr nodes and handles.
It is not rendering.
It is not layout.
It is not execution.
It is not admission.
It is not capability enforcement.
It is not Semantic truth.

## 4. Minimal Seed Rules
The first seed must check:
* empty IR is valid
* zero or one Root is valid
* multiple Roots return diagnostic
* duplicate UiIrNodeId returns diagnostic
* missing parent target returns diagnostic
* missing child target returns diagnostic
* inconsistent parent/child relationship returns diagnostic
* self-parent returns diagnostic
* self-child returns diagnostic

The first seed must not implement:
* cycle detection
* layout validation
* draw validation
* EffectBoundary semantics
* Property semantics
* Action semantics
* renderer readiness
* runtime readiness

## 5. IR Kind Boundary
* Root / Element / Text / Fragment / Property / Action / EffectBoundary are structural vocabulary only.
* EffectBoundary is not capability admission.
* Action is not execution.
* Property is not renderer property binding.
* validation must not assign execution meaning to any IR kind.

## 6. Lowering Relationship
* lower_ast_to_ir may produce UiIr.
* validate_ir may validate UiIr structure.
* lowering success does not imply verifier admission.
* IR validation success does not imply renderer readiness.
* IR validation success does not imply runtime readiness.
* validate_ir must not call renderer/backend/runtime/verifier/VM.

## 7. Authority Boundary
UI may display truth. UI does not become truth.

* IR validation is not semantic truth.
* IR validation is not verifier admission.
* IR validation is not Local Admission Guard admission.
* IR validation is not release readiness.

## 8. State Boundary
UI state is projection/cache, not semantic state.

* UiIr state is not Semantic state.
* UiIr validation state is not runtime state.
* UiIr validation state is not renderer state.
* UiIr validation must not mutate repository truth.

## 9. Quad-State Boundary
* minimal IR validation does not implement Quad-state UI semantics yet.
* future Quad-state IR overlays must preserve N/F/T/S.
* unknown must not be dropped.
* conflict must not be flattened into ordinary failure.
* denied must not be treated as false.
* not admitted must not be treated as invalid source.

## 10. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| IR validation contract | Implemented | ADMITTED | contract definition |
| validate_ir | Implemented | ADMITTED | minimal seed |
| IR structural diagnostics | Implemented | ADMITTED | minimal diagnostics |
| EffectBoundary semantics | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Action execution | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Property binding/rendering | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| cycle detection | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| renderer/backend integration | Absent | FORBIDDEN | renderer out of scope |
| runtime/VM integration | Absent | FORBIDDEN | runtime out of scope |
| verifier/admission ownership | Absent | FORBIDDEN | verifier owns admission |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |
| indexing/vectorization/Turbovec | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |

## 11. Future Gates
* R12-UI-IR-VALIDATION-AUDIT
* R12-UI-IR-DIAGNOSTICS-HARDENING
* R12-UI-IR-CYCLE-DETECTION-CONTRACT
* R12-UI-IR-EFFECT-BOUNDARY-CONTRACT
* R12-UI-RENDERER-ADAPTER-CONTRACT
* R12-UI-RUNTIME-BOUNDARY-CONTRACT

## 12. Final Decision

Final decision:
READY — UI IR VALIDATION MAY CHECK LOCAL STRUCTURE, BUT DOES NOT AUTHORIZE RENDERING, RUNTIME, LAYOUT, DRAW, EVENTS, OR ADMISSION
