# R12 UI Projection Builder Traceability Boundary

## 1. Purpose

This document defines the traceability boundary for the R12 UI Projection Builder.

It records current implemented traceability behavior and separates it from future trace expansion.

This document does not implement traceability expansion.
This document does not authorize renderer/layout/draw/event/runtime/Workbench/Studio trace integration.

> [!IMPORTANT]
> Traceability boundary records what can be traced today; it does not make unimplemented trace systems real.

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
b3d2d99acd84e5ffc5a420f7867d49d1c85ab977

Implemented:
- UiProjectedNode stores source_ir_node_id: Option<UiIrNodeId>.
- project_ir_to_projection populates source_ir_node_id for projected nodes.
- UiProjectionArtifact stores source_ir_root: Option<UiIrNodeId>.
- project_ir_to_projection sets source_ir_root when root exists.
- UiProjectionTraceRef exists as an inert handle type.
- UiProjectedNode has optional trace reference field.
- UiProjectionArtifact has traces collection.
- structural tests cover source_ir_node_id / parent-child / projection identity.

Documented:
- prior R12 documents mention traceability as a future boundary.

Absent:
- no automatic projection trace graph.
- no trace event stream.
- no trace diagnostic renderer.
- no trace severity or code catalog.
- no source-span trace expansion unless current code actually supports it.
- no lowering/validation trace bridge beyond source_ir_node_id preservation.
- no renderer/layout/event/runtime trace integration.
- no Workbench/Studio trace integration.

## 4. Evidence Matrix

| Claim | Classification | Evidence | Status |
|---|---|---|---|
| UiProjectedNode stores source_ir_node_id | IMPLEMENTED | UiProjectedNode | PASS |
| project_ir_to_projection populates source_ir_node_id | IMPLEMENTED | with_source_ir_node | PASS |
| UiProjectionArtifact stores source_ir_root | IMPLEMENTED | UiProjectionArtifact | PASS |
| project_ir_to_projection sets source_ir_root | IMPLEMENTED | set_source_ir_root | PASS |
| UiProjectionTraceRef exists | IMPLEMENTED | UiProjectionTraceRef | PASS |
| UiProjectedNode trace field exists | IMPLEMENTED | trace | PASS |
| UiProjectionArtifact traces collection exists | IMPLEMENTED | traces | PASS |
| automatic projection trace graph exists | ABSENT | source absence | PASS |
| trace event stream exists | ABSENT | source absence | PASS |
| trace diagnostic renderer exists | ABSENT | source absence | PASS |
| trace severity model exists | ABSENT | source absence | PASS |
| runtime/verifier/VM trace integration exists | FORBIDDEN / ABSENT | source absence | PASS |
| Workbench/Studio trace integration exists | FORBIDDEN / ABSENT | source absence | PASS |
| future traceability seed is allowed | AUTHORIZED_FOR_FUTURE | R12-UI-PROJECTION-BUILDER-TRACEABILITY-SEED | PASS |

## 5. Current traceability boundary

Current projection builder traceability is minimal and structural.

Currently implemented:
- projected node can retain source UiIr node id;
- projection artifact can retain source root UiIr node id;
- inert trace reference handles exist if present in code;
- trace references do not carry trace semantics by themselves.

Currently not implemented:
- no trace graph;
- no trace events;
- no trace rendering;
- no trace diagnostics;
- no trace bridge to verifier/runtime/VM;
- no Workbench/Studio trace UI.

UiProjectionTraceRef exists.
Automatic trace population by project_ir_to_projection: ABSENT.

## 6. Future traceability seed boundary

Future gate:
R12-UI-PROJECTION-BUILDER-TRACEABILITY-SEED

Allowed future scope:
- improve projection-layer trace data;
- add deterministic trace references if needed;
- add tests proving source_ir_node_id and source_ir_root preservation;
- add tests proving trace references remain inert;
- optionally add a minimal trace mapping structure if separately authorized;
- preserve projection artifact as inert.

Not allowed in future traceability seed unless separately authorized:
- renderer trace integration;
- layout trace integration;
- event trace integration;
- runtime trace integration;
- verifier/VM trace integration;
- Workbench/Studio display;
- user-facing trace renderer;
- dependency additions;
- broad trace framework.

## 7. Wishful claim guard

The following claims are explicitly not true in current main:

- The projection builder has a full traceability system.
- The projection builder has an automatic trace graph.
- The projection builder has trace event streams.
- The projection builder has renderer trace integration.
- The projection builder has layout trace integration.
- The projection builder has event trace integration.
- The projection builder has runtime/verifier/VM trace integration.
- The projection builder has Workbench/Studio trace integration.
- The projection builder has user-facing trace rendering.

These are absent or forbidden until separate explicit gates.

## 8. Authority boundary

> [!IMPORTANT]
> UI may display truth. UI does not become truth.

Traceability may point back to source UI IR structure.
Traceability does not define Semantic truth.
Traceability does not define verifier admission.
Traceability does not define capability admission.
Traceability does not define runtime state.
Traceability does not define renderer readiness.

## 9. Quad-state boundary

Unknown must not be dropped by traceability.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.
Traceability must not collapse N/F/T/S state meaning.

No dedicated Quad-state traceability is implemented in current projection builder.

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
No Workbench/Studio trace integration.
No user-facing trace renderer.

## 11. Admission Guard table

| Area | Boundary state | Admission Guard classification | Status |
|---|---|---|---|
| traceability boundary document | Present | ADMITTED | PASS |
| source_ir_node_id preservation | Implemented | ADMITTED | PASS |
| source_ir_root preservation | Implemented | ADMITTED | PASS |
| UiProjectionTraceRef | Inert handle | ADMITTED_WITH_BOUNDARY | PASS |
| automatic trace graph | Absent | AUTHORIZED_FOR_FUTURE | PASS |
| trace event stream | Absent | FUTURE_ONLY_NOT_AUTHORIZED_HERE | PASS |
| trace renderer | Absent | FUTURE_ONLY_NOT_AUTHORIZED_HERE | PASS |
| trace diagnostic integration | Absent | FUTURE_ONLY_NOT_AUTHORIZED_HERE | PASS |
| Workbench/Studio trace integration | Absent | FORBIDDEN | PASS |
| renderer/backend trace integration | Absent | FORBIDDEN | PASS |
| layout/draw/event trace integration | Absent | FORBIDDEN | PASS |
| parser/verifier/VM/runtime trace integration | Absent | FORBIDDEN | PASS |
| source changes in this PR | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## 12. Final decision

Final decision:
BOUNDARY DEFINED — R12 UI Projection Builder traceability is currently minimal and structural. Current main preserves source_ir_node_id and source_ir_root only. Full projection trace expansion remains absent and requires a separate explicit traceability seed gate.
