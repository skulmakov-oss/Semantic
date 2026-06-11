# R12 UI Projection Builder Validated IR Wrapper Boundary

## Purpose

This document defines the boundary for a future validated-IR wrapper in the R12 UI Projection Builder line.

It records current validation behavior and separates projection-layer validation evidence from verifier admission, runtime admission, capability admission, and Semantic truth.

This document does not implement a validated wrapper.

Validated IR wrapper boundary defines validation evidence, not admission authority.

## Truth classification legend

| Classification | Meaning |
|---|---|
| IMPLEMENTED | Present in current main code and locally verifiable. |
| DOCUMENTED | Present in docs/policy but not necessarily implemented. |
| AUTHORIZED_FOR_FUTURE | May be implemented only through a later explicit gate. |
| ABSENT | Not present in current main. |
| FORBIDDEN | Must not be introduced in this boundary. |

## Current factual state

Current main:
82b910cc31da8ccfc84997e2b870612a15952ed8

Implemented:
- project_ir_to_projection accepts &UiIr.
- project_ir_to_projection calls validate_ir before successful projection output.
- validate_ir uses UiIrValidationConfig::default() in current projection path if verified.
- validate_ir failure maps to UiProjectionError::InvalidIr.
- UiIrValidationDiagnostics is available through diagnostics accessors.

Absent:
- no ValidatedUiIr wrapper exists.
- no project_ir_to_projection_validated exists.
- no type-level validated-state proof exists.
- no projection entry point requires a validated wrapper.
- no verifier admission is represented by current projection validation.
- no runtime/capability admission is represented by current projection validation.

## Evidence Matrix

| Claim | Classification | Evidence | Status |
|---|---|---|---|
| project_ir_to_projection accepts &UiIr | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| project_ir_to_projection calls validate_ir | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| validate_ir failure maps to InvalidIr | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| UiIrValidationDiagnostics exists | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| ValidatedUiIr wrapper exists | ABSENT | crates/prom-ui/src/projection.rs | PASS |
| project_ir_to_projection_validated exists | ABSENT | crates/prom-ui/src/projection.rs | PASS |
| type-level validation proof exists | ABSENT | crates/prom-ui/src/projection.rs | PASS |
| ValidatedUiIr equals Semantic truth | FORBIDDEN | R12 v0 closeout | PASS |
| ValidatedUiIr equals verifier admission | FORBIDDEN | R12 v0 closeout | PASS |
| ValidatedUiIr equals runtime admission | FORBIDDEN | R12 v0 closeout | PASS |
| ValidatedUiIr equals capability admission | FORBIDDEN | R12 v0 closeout | PASS |
| future ValidatedUiIr seed is allowed | AUTHORIZED_FOR_FUTURE | this document | PASS |

## Current validation boundary

Current projection validation is internal and local.

Current behavior:
- caller passes raw &UiIr;
- project_ir_to_projection validates internally;
- invalid IR returns UiProjectionError::InvalidIr;
- valid IR may produce inert UiProjectionArtifact.

This is implemented behavior.

Current limitations:
- validation status is not represented at the type level;
- callers cannot pass a distinct ValidatedUiIr wrapper;
- repeated validation may occur if future callers need explicit staged projection;
- current validation is not verifier admission.

## Future wrapper boundary

Future gate:
R12-UI-PROJECTION-BUILDER-VALIDATED-IR-WRAPPER-SEED

Allowed future scope:
- introduce a small projection-layer validated wrapper if needed;
- wrapper may borrow or own UiIr according to future design;
- wrapper must be created only after validate_ir succeeds;
- wrapper may preserve validation config identity if needed;
- wrapper may expose underlying UiIr read-only;
- add tests proving invalid IR cannot produce wrapper;
- add tests proving projection from wrapper does not skip validation unsafely;
- preserve existing project_ir_to_projection behavior unless a separate migration gate authorizes change.

Not allowed in future seed unless separately authorized:
- replacing verifier admission;
- representing Semantic truth;
- runtime/capability admission;
- effect authorization;
- renderer readiness;
- Workbench/Studio readiness;
- dependency additions;
- broad builder framework;
- breaking current project_ir_to_projection callers without migration gate.

## Wrapper non-authority rules

A future ValidatedUiIr wrapper would not be:

- Semantic truth;
- verifier admission;
- runtime admission;
- capability admission;
- effect authorization;
- renderer readiness;
- layout readiness;
- Workbench/Studio readiness;
- release readiness.

ValidatedUiIr may prove projection-layer validation only.

## Authority boundary

UI may display truth. UI does not become truth.

Projection validation may reject malformed UI IR for projection.

Projection validation does not define Semantic truth.
Projection validation does not define verifier admission.
Projection validation does not define capability admission.
Projection validation does not define runtime state.
Projection validation does not define renderer readiness.

## Quad-state boundary

Unknown must not be dropped by validated-wrapper policy.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.
Validated-wrapper policy must not collapse N/F/T/S meaning.

No dedicated Quad-state validated-wrapper semantics are implemented in current main.

## Wishful claim guard

The following claims are explicitly not true in current main:

- ValidatedUiIr exists.
- project_ir_to_projection requires ValidatedUiIr.
- validated wrapper proves Semantic truth.
- validated wrapper replaces verifier admission.
- validated wrapper admits runtime effects.
- validated wrapper grants capabilities.
- validated wrapper implies renderer readiness.
- validated wrapper integrates with Workbench/Studio.

These are absent or forbidden until separate explicit gates.

## Explicit non-scope

No implementation in this PR.
No source changes.
No projection.rs changes.
No validation.rs changes.
No model.rs changes.
No lowering.rs changes.
No lib.rs changes.
No Cargo.toml / Cargo.lock changes.
No dependency additions.
No ValidatedUiIr type.
No project_ir_to_projection_validated.
No verifier/runtime/capability admission.
No renderer/backend/layout/draw/event.
No Workbench/Studio integration.

## Admission Guard table

| Area | Boundary state | Admission Guard classification | Status |
|---|---|---|---|
| validated IR wrapper boundary document | Present | ADMITTED | PASS |
| current internal validate_ir bridge | Implemented | ADMITTED | PASS |
| UiIrValidationDiagnostics | Implemented | ADMITTED | PASS |
| ValidatedUiIr wrapper | Absent | AUTHORIZED_FOR_FUTURE | PASS |
| project_ir_to_projection_validated | Absent | AUTHORIZED_FOR_FUTURE | PASS |
| Semantic truth authority | Absent | FORBIDDEN | PASS |
| verifier admission | Absent | FORBIDDEN | PASS |
| runtime admission | Absent | FORBIDDEN | PASS |
| capability admission | Absent | FORBIDDEN | PASS |
| renderer readiness | Absent | FORBIDDEN | PASS |
| Workbench/Studio readiness | Absent | FORBIDDEN | PASS |
| source changes in this PR | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## Final decision

Final decision:
BOUNDARY DEFINED — A future ValidatedUiIr wrapper may be introduced only as projection-layer validation evidence. It must not represent Semantic truth, verifier admission, runtime admission, capability admission, renderer readiness, or Workbench/Studio readiness. Current main does not implement the wrapper.
