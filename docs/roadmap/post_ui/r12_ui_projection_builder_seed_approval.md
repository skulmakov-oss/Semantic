# R12 UI Projection Builder Seed Approval

## 6.1 Purpose

This document authorizes only a future seed task shape.
It does not authorize implementation.
It does not authorize builder code in this PR.
It does not authorize renderer/layout/draw/event/runtime integration.

This approval document does not implement the projection builder.

Builder code remains forbidden until a separate explicit implementation task is authorized.

## 6.2 Current factual state

UiAst exists.
UiIr exists.
validate_ast exists.
lower_ast_to_ir exists.
validate_ir exists.
UiProjectionArtifact exists.
UiProjectedNode exists.
projection artifact seed exists.
projection artifact seed is inert.
projection builder contract exists.
projection builder audit passed.
no projection builder exists.
no project_ir_to_projection exists.
no From<UiIr> / TryFrom<UiIr> projection conversion exists.
no renderer/backend/layout/draw/event loop exists.
no Workbench/Studio shell exists.

References:
PR #913
Issue #914
docs/roadmap/post_ui/r12_ui_projection_builder_contract.md

## 6.3 Seed approval decision

Decision:
APPROVED_FOR_FUTURE_SEED — A minimal projection builder seed may be proposed in a separate future implementation PR, but this approval does not itself authorize code.

## 6.4 Future seed scope

R12-UI-PROJECTION-BUILDER-SEED

Allowed future purpose:
Map validated UiIr into inert UiProjectionArtifact.

Allowed future function name, if later authorized:
`project_ir_to_projection(...)`

The function name is reserved for a future task.
It must not be implemented in this approval PR.

Allowed future file, if later authorized:
crates/prom-ui/src/projection.rs

Potential future tests, if later authorized:
projection builder preserves root node
projection builder preserves parent/child relationships
projection builder preserves source UiIr references
projection builder rejects or reports invalid handles
projection builder remains deterministic
projection builder does not create renderer/layout/draw/event artifacts

## 6.5 Future seed minimum contract

Input:
- validated UiIr or borrowed &UiIr plus explicit validation boundary.

Output:
- inert UiProjectionArtifact.

Allowed mapping:
- UiIrNodeId → UiProjectedNodeId.
- UiIrNodeKind → UiProjectedNodeKind.
- parent/child structural relationships.
- source references where already available.

Forbidden in future seed:
- renderer resources.
- layout computation.
- draw commands.
- event handlers.
- runtime state.
- Semantic state mutation.
- verifier admission.
- capability admission.
- Workbench/Studio integration.

## 6.6 Validation boundary decision

The future seed must choose one of two validation strategies:

A. require a validated UiIr wrapper;
B. call validate_ir internally before projection.

The exact strategy must be chosen in the implementation task.
This approval does not change validate_ir.

No validation.rs change is authorized by this approval.

## 6.7 Diagnostics boundary

Future seed diagnostics may cover:
invalid IR if builder validates internally
unsupported node kind
inconsistent handles
missing required root
missing parent/child target
unsupported Property mapping
unsupported Action mapping
unsupported EffectBoundary mapping

But must not:
panic
execute effects
call renderer/backend/runtime/verifier/VM/parser
flatten N/F/T/S meanings
turn denied into false
turn not-admitted into invalid source

## 6.8 Determinism boundary

same UiIr input + same builder config = same projection artifact or same diagnostics

no randomness
no wall-clock time
no file I/O
no network
no command execution
no host effects
no global mutable state

## 6.9 Quad-state boundary

Unknown must not be dropped.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.
Visual representation of Quad-state requires a separate contract.

## 6.10 Authority boundary

UI may display truth. UI does not become truth.

Projection is cache/view artifact.
Projection is not Semantic truth.
Projection is not runtime state.
Projection is not verifier admission.
Projection is not capability admission.
Projection is not release readiness.

## 6.11 Explicit forbidden list

no implementation in this approval PR
no Rust source changes
no projection.rs changes
no lib.rs changes
no validation.rs changes
no lowering.rs changes
no model.rs changes
no project_ir_to_projection
no build_projection
no ProjectionBuilder
no UiProjectionBuilder
no From<UiIr>
no TryFrom<UiIr>
no renderer/backend
no WGPU
no winit
no Tauri
no layout
no draw
no event loop
no parser/verifier/VM/runtime integration
no Workbench/Studio
no Cargo.toml changes
no Cargo.lock changes
no dependency additions

## 6.12 Admission Guard table

| Area | Approval state | Admission Guard classification | Notes |
|---|---|---|---|
| seed approval document | Implemented | ADMITTED | approval gate |
| future builder seed task | Authorized | APPROVED_FOR_FUTURE_TASK | future gate |
| builder implementation in this PR | Absent | FORBIDDEN | boundary |
| project_ir_to_projection in this PR | Absent | FORBIDDEN | boundary |
| From/TryFrom UiIr in this PR | Absent | FORBIDDEN | boundary |
| UiIr to UiProjectedNode mapping in this PR | Absent | FORBIDDEN | boundary |
| future UiIr to UiProjectedNode mapping | Authorized | FUTURE_ONLY_NOT_AUTHORIZED_HERE | future gate |
| future builder diagnostics | Authorized | FUTURE_ONLY_NOT_AUTHORIZED_HERE | future gate |
| future builder traceability | Authorized | FUTURE_ONLY_NOT_AUTHORIZED_HERE | future gate |
| Property mapping | Authorized | FUTURE_ONLY_NOT_AUTHORIZED_HERE | future gate |
| Action mapping | Authorized | FUTURE_ONLY_NOT_AUTHORIZED_HERE | future gate |
| EffectBoundary mapping | Authorized | FUTURE_ONLY_NOT_AUTHORIZED_HERE | future gate |
| validate_ir integration | Authorized | FUTURE_ONLY_NOT_AUTHORIZED_HERE | future gate |
| projection.rs changes | Absent | FORBIDDEN | boundary |
| lib.rs changes | Absent | FORBIDDEN | boundary |
| validation/lowering/model changes | Absent | FORBIDDEN | boundary |
| renderer/backend integration | Absent | FORBIDDEN | boundary |
| layout/draw/event implementation | Absent | FORBIDDEN | boundary |
| parser/verifier/VM/runtime integration | Absent | FORBIDDEN | boundary |
| Workbench/Studio integration | Absent | FORBIDDEN | boundary |
| dependency additions | Absent | FORBIDDEN | boundary |

## 6.13 Future implementation gate

Next gated task after approval:
R12-UI-PROJECTION-BUILDER-SEED

This approval does not create that implementation task.
That implementation task requires explicit authorization.

## 6.14 Final decision

Final decision:
APPROVED_FOR_FUTURE_SEED — The project may later open a separate minimal projection builder seed task, limited to mapping validated UiIr into inert UiProjectionArtifact. No builder implementation is authorized by this approval document.
