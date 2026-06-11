# R12 UI Projection Builder Diagnostics Boundary

## 1. Purpose

This document defines the diagnostics boundary for the R12 UI Projection Builder.

It records current implemented diagnostics behavior and separates it from future diagnostic expansion.

This document does not implement diagnostics.
This document does not authorize renderer/layout/draw/event/runtime/Workbench/Studio diagnostics.

> [!IMPORTANT]
> Diagnostics boundary defines what may be reported; it does not make unimplemented diagnostics real.

## 2. Truth classification legend

| Classification | Meaning |
|---|---|
| IMPLEMENTED | Present in current main code and locally verifiable. |
| DOCUMENTED | Present in docs/policy but not necessarily implemented. |
| AUTHORIZED_FOR_FUTURE | May be implemented only through a later explicit gate. |
| ABSENT | Not present in current main. |
| FORBIDDEN | Must not be introduced in this boundary. |

## 3. Current factual state

Current main:
0805e1f9c574a303a16999127af2e84976588f65

Implemented:
- UiProjectionError exists in crates/prom-ui/src/projection.rs.
- UiProjectionError::InvalidIr wraps UiIrValidationDiagnostics.
- project_ir_to_projection calls validate_ir with UiIrValidationConfig::default().
- invalid IR is rejected before successful projection output.
- project_ir_to_projection returns Result<UiProjectionArtifact, UiProjectionError>.
- focused unit tests cover invalid IR rejection.

Documented:
- prior R12 contract documents mention future diagnostics boundaries.

Absent:
- no dedicated projection diagnostics renderer.
- no projection diagnostic catalog.
- no projection diagnostic severity model.
- no projection diagnostic codes.
- no builder-specific source-span rendering.
- no CLI/Workbench diagnostics output integration.
- no renderer/layout/event/runtime diagnostics.

## 4. Evidence Matrix

| Claim | Classification | Evidence | Status |
|---|---|---|---|
| UiProjectionError exists | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| InvalidIr wraps UiIrValidationDiagnostics | IMPLEMENTED | UiProjectionError::InvalidIr | PASS |
| project_ir_to_projection calls validate_ir | IMPLEMENTED | project_ir_to_projection | PASS |
| invalid IR rejection is tested | IMPLEMENTED | test_invalid_ir_rejected | PASS |
| rich projection diagnostics renderer exists | ABSENT | source absence | PASS |
| diagnostic catalog exists | ABSENT | source absence | PASS |
| renderer diagnostics exist | FORBIDDEN / ABSENT | source absence | PASS |
| layout diagnostics exist | FORBIDDEN / ABSENT | source absence | PASS |
| event diagnostics exist | FORBIDDEN / ABSENT | source absence | PASS |
| runtime diagnostics exist | FORBIDDEN / ABSENT | source absence | PASS |
| Workbench/Studio diagnostics exist | FORBIDDEN / ABSENT | source absence | PASS |
| future diagnostics seed is allowed | AUTHORIZED_FOR_FUTURE | R12-UI-PROJECTION-BUILDER-DIAGNOSTICS-SEED | PASS |

## 5. Current diagnostics boundary

Current projection builder diagnostics are minimal and structural.

Currently implemented:
- validation failure via UiProjectionError::InvalidIr;
- missing root if reachable after validation;
- minimal enum variants for missing node / unsupported node kind / inconsistent handle if present in code;
- no rendering layer;
- no severity levels;
- no diagnostic code catalog;
- no user-facing formatting.

## 6. Future diagnostics seed boundary

Future gate:
R12-UI-PROJECTION-BUILDER-DIAGNOSTICS-SEED

Allowed future scope:
- improve projection-layer error specificity;
- add focused tests for each reachable projection error;
- add deterministic diagnostic data structure if needed;
- preserve validate_ir as validation boundary;
- keep output inert;
- no renderer/layout/event/runtime/Workbench coupling.

Not allowed in future diagnostics seed unless separately authorized:
- renderer diagnostics;
- layout diagnostics;
- event diagnostics;
- runtime diagnostics;
- CLI formatting;
- Workbench/Studio display;
- dependency additions;
- broad diagnostic framework.

## 7. Wishful claim guard

The following claims are explicitly not true in current main:

- The projection builder has rich diagnostics.
- The projection builder has renderer diagnostics.
- The projection builder has layout diagnostics.
- The projection builder has event diagnostics.
- The projection builder has runtime diagnostics.
- The projection builder has Workbench/Studio diagnostic integration.
- The projection builder has a diagnostic catalog.
- The projection builder has user-facing diagnostic rendering.

These are absent or forbidden until separate explicit gates.

## 8. Authority boundary

> [!IMPORTANT]
> UI may display truth. UI does not become truth.

Diagnostics may describe projection failures.
Diagnostics do not define Semantic truth.
Diagnostics do not define verifier admission.
Diagnostics do not define capability admission.
Diagnostics do not define runtime state.
Diagnostics do not define renderer readiness.

## 9. Quad-state boundary

Unknown must not be dropped by diagnostics.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.
Diagnostics must not collapse N/F/T/S state meaning.

No dedicated Quad-state diagnostics are implemented in current projection builder.

## 10. Explicit non-scope

No implementation in this PR.
No source changes.
No projection.rs changes.
No validation.rs changes.
No model.rs changes.
No lowering.rs changes.
No lib.rs changes.
No Cargo.toml / Cargo.lock changes.
No dependency additions.
No renderer/backend/layout/draw/event.
No parser/verifier/VM/runtime integration.
No Workbench/Studio diagnostics.
No user-facing diagnostics renderer.

## 11. Admission Guard table

| Area | Boundary state | Admission Guard classification | Status |
|---|---|---|---|
| diagnostics boundary document | Present | ADMITTED | PASS |
| UiProjectionError | Implemented | ADMITTED | PASS |
| InvalidIr validation bridge | Implemented | ADMITTED | PASS |
| rich projection diagnostics | Absent | AUTHORIZED_FOR_FUTURE | PASS |
| diagnostics renderer | Absent | FUTURE_ONLY_NOT_AUTHORIZED_HERE | PASS |
| diagnostic catalog | Absent | FUTURE_ONLY_NOT_AUTHORIZED_HERE | PASS |
| CLI diagnostics integration | Absent | FUTURE_ONLY_NOT_AUTHORIZED_HERE | PASS |
| Workbench/Studio diagnostics | Absent | FORBIDDEN | PASS |
| renderer/backend diagnostics | Absent | FORBIDDEN | PASS |
| layout/draw/event diagnostics | Absent | FORBIDDEN | PASS |
| parser/verifier/VM/runtime diagnostics | Absent | FORBIDDEN | PASS |
| source changes in this PR | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## 12. Final decision

Final decision:
BOUNDARY DEFINED — R12 UI Projection Builder diagnostics are currently minimal and structural. Current main implements UiProjectionError and validate_ir bridging only. Rich projection diagnostics remain absent and require a separate explicit diagnostics seed gate.
