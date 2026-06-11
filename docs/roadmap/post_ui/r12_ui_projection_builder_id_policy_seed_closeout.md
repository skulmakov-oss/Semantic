# R12 UI Projection Builder ID Policy Seed Closeout

## 1. Purpose

This document closes out the R12 UI Projection Builder ID Policy Seed after PR #919.

It records that deterministic artifact identity has been implemented while preserving the inert projection boundary.

This document does not authorize renderer/layout/draw/event/runtime/Workbench/Studio widening.

> [!IMPORTANT]
> ID policy seed closeout records implementation completion; it does not authorize further UI system widening.

## 2. Closed chain

#913 — Projection Builder Contract — MERGED
#914 — Contract Audit — CLOSED / PASS
#915 — Seed Approval — MERGED
#916 — Inert Projection Builder Seed — MERGED
#917 — Seed Closeout — MERGED
#918 — ID Policy — MERGED
#919 — ID Policy Seed — MERGED

## 3. Final implementation state

Current main:
995d7e99fb5117acb275d1dab15e705c0c91c0ba

Implemented function:
projection_artifact_id_for_ir

Projection function:
project_ir_to_projection

Validation:
validate_ir remains the internal validation boundary

Artifact ID:
deterministic projection-layer artifact ID

Projected node ID:
UiProjectedNodeId remains structural and derived from UiIrNodeId raw value

Source traceability:
source_ir_node_id and source_ir_root remain preserved

## 4. What changed in #919

- replaced/formalized seed-only artifact ID behavior;
- introduced projection_artifact_id_for_ir;
- artifact ID now follows source-root identity policy or empty-IR identity policy;
- added focused deterministic ID tests;
- preserved projected node identity policy;
- preserved parent/child projection behavior;
- preserved inert projection artifact boundary.

## 5. What did not change

- no renderer/backend;
- no WGPU/winit/Tauri;
- no layout engine;
- no draw commands;
- no event loop;
- no parser/verifier/VM/runtime integration;
- no Workbench/Studio;
- no Semantic state mutation;
- no capability admission;
- no ProjectionBuilder type;
- no From<UiIr> / TryFrom<UiIr>;
- no dependencies;
- no Cargo.toml / Cargo.lock changes.

## 6. Final identity policy state

UiProjectionArtifactId:
  deterministic projection-layer artifact identity.

UiProjectedNodeId:
  structural deterministic node identity derived from UiIrNodeId.

UiProjectionPropertyRef / UiProjectionActionRef / UiProjectionEffectBoundaryRef / UiProjectionTraceRef:
  inert projection references.

Projection identity is not:
  renderer identity;
  runtime identity;
  Semantic truth;
  verifier admission;
  capability admission;
  Workbench/Studio identity.

> [!IMPORTANT]
> Projection identity remains deterministic, inert, and projection-layer local.

## 7. Determinism and safety

same UiIr input + same ID policy = same UiProjectionArtifact identity graph

No randomness.
No wall-clock time.
No network.
No file I/O.
No command execution.
No host effects.
No global mutable state.

## 8. Admission Guard table

| Area | Final state | Admission Guard classification | Status |
|---|---|---|---|
| ID policy seed | Merged | ADMITTED | PASS |
| projection_artifact_id_for_ir | Implemented | ADMITTED | PASS |
| old artifact ID seed constant | Replaced/formalized | ADMITTED | PASS |
| UiProjectedNodeId policy | Unchanged structural deterministic | ADMITTED | PASS |
| validate_ir integration | Unchanged internal call | ADMITTED | PASS |
| parent/child mapping | Preserved | ADMITTED | PASS |
| source traceability | Preserved | ADMITTED | PASS |
| ProjectionBuilder type | Absent | FUTURE_ONLY_NOT_AUTHORIZED | PASS |
| From/TryFrom UiIr | Absent | FUTURE_ONLY_NOT_AUTHORIZED | PASS |
| renderer/backend integration | Absent | FORBIDDEN | PASS |
| layout/draw/event implementation | Absent | FORBIDDEN | PASS |
| parser/verifier/VM/runtime integration | Absent | FORBIDDEN | PASS |
| Workbench/Studio integration | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## 9. Known remaining limits

- no builder config exists;
- no validated UiIr wrapper exists;
- no digest-based artifact identity exists;
- no projection trace expansion exists;
- no renderer-facing adapter exists;
- no layout/draw/event semantics exist;
- no Workbench/Studio integration exists.

## 10. Recommended next gates

R12-UI-PROJECTION-BUILDER-DIAGNOSTICS
R12-UI-PROJECTION-BUILDER-TRACEABILITY
R12-UI-PROJECTION-PROPERTY-ACTION-EFFECT-CONTRACT

## 11. Final decision

Final decision:
CLOSED — R12 UI Projection Builder ID Policy Seed is merged as a deterministic, inert projection-layer identity policy. Further widening requires separate explicit gates.
