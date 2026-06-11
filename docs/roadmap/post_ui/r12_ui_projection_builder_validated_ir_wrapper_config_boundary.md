# R12 UI Projection Builder Validated IR Wrapper Config Boundary

## Purpose

This document defines the boundary for future config-aware ValidatedUiIr construction.

It records the current default-config validation behavior and separates projection-layer validation configuration from Semantic truth, verifier admission, runtime admission, capability admission, and renderer readiness.

This document does not implement config-aware validation.

Validated IR config boundary defines projection validation policy, not admission authority.

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
c10b4442733119a8a1dab1635066cf38d697da6d

Implemented:
- ValidatedUiIr exists.
- ValidatedUiIr::new validates through validate_ir.
- Current ValidatedUiIr::new uses UiIrValidationConfig::default().
- validate_ui_ir_for_projection exists.
- project_validated_ir_to_projection exists.
- raw project_ir_to_projection(&UiIr) remains preserved.

Absent:
- no config-aware ValidatedUiIr constructor exists.
- no wrapper stores validation config identity.
- no validate_ui_ir_for_projection_with_config exists.
- no project_validated_ir_to_projection_with_config exists.
- no validation config authority beyond projection-layer validation.

## Evidence Matrix

| Claim | Classification | Evidence | Status |
|---|---|---|---|
| ValidatedUiIr exists | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| ValidatedUiIr::new uses validate_ir | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| ValidatedUiIr::new uses UiIrValidationConfig::default | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| validate_ui_ir_for_projection exists | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| project_validated_ir_to_projection exists | IMPLEMENTED | crates/prom-ui/src/projection.rs | PASS |
| config-aware constructor exists | ABSENT | crates/prom-ui/src/projection.rs | PASS |
| wrapper stores config identity | ABSENT | crates/prom-ui/src/projection.rs | PASS |
| validate_ui_ir_for_projection_with_config exists | ABSENT | crates/prom-ui/src/projection.rs | PASS |
| config equals Semantic truth policy | FORBIDDEN | Validated IR Config Boundary | PASS |
| config equals verifier policy | FORBIDDEN | Validated IR Config Boundary | PASS |
| config equals runtime admission policy | FORBIDDEN | Validated IR Config Boundary | PASS |
| config equals capability policy | FORBIDDEN | Validated IR Config Boundary | PASS |
| future config seed is allowed | AUTHORIZED_FOR_FUTURE | Validated IR Config Boundary | PASS |

## Current config boundary

Current config behavior is implicit and defaulted.

Current behavior:
- ValidatedUiIr::new uses UiIrValidationConfig::default().
- The wrapper does not expose config identity.
- The wrapper does not distinguish validation profiles.
- The wrapper remains projection-layer validation evidence only.

Current limitations:
- future callers cannot request explicit validation profiles;
- future callers cannot inspect which config created the wrapper;
- config-specific validation policy is not represented in the type system;
- this does not affect verifier/runtime/capability admission.

## Future config seed boundary

Future gate:
R12-UI-PROJECTION-BUILDER-VALIDATED-IR-WRAPPER-CONFIG-SEED-FULL-PACKAGE

Allowed future scope:
- add config-aware constructor if needed;
- add validate_ui_ir_for_projection_with_config if needed;
- optionally store or expose validation config identity if the config type supports stable identity;
- add tests proving invalid IR cannot produce wrapper under explicit config;
- add tests proving default constructor behavior remains stable;
- preserve raw project_ir_to_projection(&UiIr);
- preserve existing ValidatedUiIr::new unless separate migration gate authorizes change.

Not allowed unless separately authorized:
- verifier policy;
- runtime policy;
- capability policy;
- Semantic truth policy;
- renderer readiness policy;
- Workbench/Studio readiness policy;
- dependency additions;
- broad validation framework;
- breaking existing ValidatedUiIr callers.

## Config non-authority rules

A future validation config would not be:

- Semantic truth policy;
- verifier policy;
- runtime admission policy;
- capability policy;
- effect authorization policy;
- renderer readiness policy;
- layout readiness policy;
- Workbench/Studio readiness policy;
- release readiness policy.

UiIrValidationConfig may configure projection validation only.

## Authority boundary

UI may display truth. UI does not become truth.

Projection validation config may tune projection-layer validation.

Projection validation config does not define Semantic truth.
Projection validation config does not define verifier admission.
Projection validation config does not define runtime state.
Projection validation config does not define capability admission.
Projection validation config does not define renderer readiness.

## Quad-state boundary

Unknown must not be dropped by validation config policy.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.
Validated wrapper config policy must not collapse N/F/T/S meaning.

No dedicated Quad-state config semantics are implemented in current main.

## Wishful claim guard

The following claims are explicitly not true in current main:

- ValidatedUiIr supports explicit configs.
- ValidatedUiIr stores validation config identity.
- validate_ui_ir_for_projection_with_config exists.
- validation config represents verifier policy.
- validation config represents Semantic truth policy.
- validation config grants runtime or capability admission.
- validation config implies renderer readiness.
- validation config integrates with Workbench/Studio.

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
No config-aware ValidatedUiIr constructor.
No config identity storage.
No verifier/runtime/capability admission.
No renderer/backend/layout/draw/event.
No Workbench/Studio integration.

## Admission Guard table

| Area | Boundary state | Admission Guard classification | Status |
|---|---|---|---|
| validated IR config boundary document | Present | ADMITTED | PASS |
| current default config validation | Implemented | ADMITTED | PASS |
| ValidatedUiIr::new | Implemented | ADMITTED | PASS |
| config-aware constructor | Absent | AUTHORIZED_FOR_FUTURE | PASS |
| config identity storage | Absent | AUTHORIZED_FOR_FUTURE | PASS |
| Semantic truth policy | Absent | FORBIDDEN | PASS |
| verifier policy | Absent | FORBIDDEN | PASS |
| runtime admission policy | Absent | FORBIDDEN | PASS |
| capability policy | Absent | FORBIDDEN | PASS |
| renderer readiness policy | Absent | FORBIDDEN | PASS |
| Workbench/Studio readiness policy | Absent | FORBIDDEN | PASS |
| source changes in this PR | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## Final decision

Final decision:
BOUNDARY DEFINED — Future config-aware ValidatedUiIr construction may be introduced only as projection-layer validation policy. UiIrValidationConfig may configure projection validation only. It must not represent Semantic truth policy, verifier policy, runtime admission policy, capability policy, renderer readiness policy, or Workbench/Studio readiness policy. Current main does not implement config-aware wrapper construction.
