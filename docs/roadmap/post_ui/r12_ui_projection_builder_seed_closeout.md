# R12 UI Projection Builder Seed Closeout

### 1. Purpose

This document closes out the R12 UI Projection Builder Seed track.
It records that the seed is merged and remains inert.
It does not authorize renderer, layout, draw, event, runtime, VM, verifier, Workbench, or Studio implementation.

### 2. Closed chain

#913 — Projection Builder Contract — CLOSED
#914 — Contract Audit — CLOSED / PASS
#915 — Seed Approval — MERGED / PASS
#916 — Inert Projection Builder Seed — MERGED / PASS

### 3. Final seed state

Current main:
b696cff5fed15c2c10b36925f221a701c3af9565

Seed function:
project_ir_to_projection

Input:
&UiIr

Validation:
validate_ir with default UiIrValidationConfig

Output:
Result<UiProjectionArtifact, UiProjectionError>

### 4. Implemented scope

- validates UiIr before projection
- maps UiIr nodes to UiProjectedNode
- maps Root / Element / Text / Fragment
- maps Property to PropertyCarrier
- maps Action to ActionCarrier
- maps EffectBoundary to EffectBoundaryMarker
- preserves parent/child handles
- preserves source UiIr node references
- remains deterministic
- keeps projection artifact inert

### 5. Explicit non-scope

- no renderer/backend
- no WGPU
- no winit
- no Tauri
- no layout engine
- no draw commands
- no event loop
- no parser/verifier/VM/runtime integration
- no Workbench/Studio
- no Semantic state mutation
- no capability admission
- no From/TryFrom<UiIr>
- no ProjectionBuilder type
- no dependency additions

### 6. Admission Guard table

| Area | Final state | Admission Guard classification | Status |
|---|---|---|---|
| projection builder contract | Closed | ADMITTED | PASS |
| contract audit | Closed | ADMITTED | PASS |
| seed approval | Merged | ADMITTED | PASS |
| projection builder seed | Merged | ADMITTED | PASS |
| project_ir_to_projection | Implemented | ADMITTED | PASS |
| validate_ir integration | Internal call only | ADMITTED | PASS |
| UiIr to UiProjectedNode mapping | Implemented minimally | ADMITTED | PASS |
| Property mapping | Inert carrier | ADMITTED_WITH_BOUNDARY | PASS |
| Action mapping | Inert carrier | ADMITTED_WITH_BOUNDARY | PASS |
| EffectBoundary mapping | Inert marker | ADMITTED_WITH_BOUNDARY | PASS |
| From/TryFrom UiIr conversion | Absent | FUTURE_ONLY_NOT_AUTHORIZED | PASS |
| ProjectionBuilder type | Absent | FUTURE_ONLY_NOT_AUTHORIZED | PASS |
| renderer/backend integration | Absent | FORBIDDEN | PASS |
| layout/draw/event implementation | Absent | FORBIDDEN | PASS |
| parser/verifier/VM/runtime integration | Absent | FORBIDDEN | PASS |
| Workbench/Studio integration | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

### 7. Known seed limitations

- artifact id policy is still seed-level and deterministic but not final API policy
- no builder config exists
- no validated UiIr wrapper exists
- no projection trace expansion exists
- no renderer-facing adapter exists
- no layout/draw/event semantics exist

### 8. Recommended next gates

R12-UI-PROJECTION-BUILDER-ID-POLICY
R12-UI-PROJECTION-BUILDER-DIAGNOSTICS
R12-UI-PROJECTION-BUILDER-TRACEABILITY
R12-UI-PROJECTION-PROPERTY-ACTION-EFFECT-CONTRACT

### 9. Final decision

Final decision:
CLOSED — R12 UI Projection Builder Seed is merged as an inert, deterministic projection layer. Further widening requires separate explicit gates.
