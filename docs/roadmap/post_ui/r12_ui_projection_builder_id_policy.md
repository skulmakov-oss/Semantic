# R12 UI Projection Builder ID Policy

### 8.1 Purpose

This document defines the identity policy for UI projection artifacts and projected nodes after the initial R12 projection builder seed.

This document does not implement ID policy changes.
This document does not authorize renderer/layout/draw/event/runtime integration.
This document does not authorize Workbench/Studio.

ID policy is a projection-layer concern, not renderer identity, runtime identity, Semantic truth, or verifier admission.

### 8.2 Current factual state

#913 added the projection builder contract.
#914 audited the contract.
#915 approved a future seed.
#916 merged the inert projection builder seed.
#917 closed out the seed.

Current builder:
project_ir_to_projection(&UiIr) -> Result<UiProjectionArtifact, UiProjectionError>

Current validation:
validate_ir with UiIrValidationConfig::default()

Current artifact ID:
UiProjectionArtifactId::new(1)

Current projected node ID:
UiProjectedNodeId::new(ir_node.id().raw())

Current source preservation:
UiProjectedNode.source_ir_node_id = Some(UiIrNodeId)
UiProjectionArtifact.source_ir_root = Some(UiIrNodeId)

### 8.3 Problem statement

The seed is deterministic but the artifact ID is still seed-level.
UiProjectionArtifactId::new(1) is acceptable for the first seed but not a final policy.
Projected node IDs currently mirror UiIrNodeId raw values, which is deterministic and traceable but must be named as an explicit policy.
The project needs a stable rule before further widening.

### 8.4 Identity domains

Source identity:
  UiIrNodeId

Projection artifact identity:
  UiProjectionArtifactId

Projected node identity:
  UiProjectedNodeId

Projection reference identities:
  UiProjectionPropertyRef
  UiProjectionActionRef
  UiProjectionEffectBoundaryRef
  UiProjectionTraceRef

Runtime identity:
  out of scope

Renderer identity:
  out of scope

Workbench/Studio identity:
  out of scope

These identity domains must not be collapsed into one universal ID space.

### 8.5 Policy decision

Decision:
ADOPT TWO-LEVEL DETERMINISTIC PROJECTION ID POLICY

1. UiProjectedNodeId remains deterministically derived from UiIrNodeId for the current structural seed.
2. UiProjectionArtifactId must move from constant seed ID to a deterministic artifact ID policy in a future implementation gate.
3. Until the future implementation gate, UiProjectionArtifactId::new(1) remains explicitly classified as SEED_ONLY.

Terms:
SEED_ONLY
STRUCTURAL_DETERMINISTIC
FUTURE_POLICY_REQUIRED

### 8.6 Projected node ID policy

For the current seed:
UiProjectedNodeId = deterministic projection of UiIrNodeId.

Allowed:
- preserving raw UiIrNodeId value as projected node raw ID while projection remains one-to-one and structural.
- preserving source_ir_node_id separately for traceability.

Forbidden:
- using random IDs;
- using allocation-order IDs if they can diverge from UiIrNodeId without explicit policy;
- using wall-clock/time-based IDs;
- using renderer handles;
- using runtime handles;
- using global mutable counters;
- using host-generated IDs.

Projected node ID equality does not mean Semantic truth equality.
Projected node ID equality does not mean renderer resource equality.
Projected node ID equality does not mean runtime object identity.

### 8.7 Artifact ID policy

Current:
UiProjectionArtifactId::new(1) is SEED_ONLY.

Future allowed policies:
A. deterministic constant for single-artifact seed mode;
B. deterministic ID derived from source root UiIrNodeId;
C. deterministic ID derived from builder config + source root;
D. deterministic digest-based ID if digest policy is separately authorized.

Not allowed without separate gate:
- random artifact IDs;
- timestamp IDs;
- UUID generation;
- file-system-derived IDs;
- host/session IDs;
- renderer resource IDs;
- runtime object IDs;
- VM/verifier IDs.

Preferred future direction:
A future implementation gate should replace the seed constant with an explicit deterministic artifact ID constructor or builder config policy.

### 8.8 Reference ID policy

UiProjectionPropertyRef
UiProjectionActionRef
UiProjectionEffectBoundaryRef
UiProjectionTraceRef

Current state:
structural handles only.

Policy:
These references remain inert projection references.
They must not become:
- renderer bindings;
- event handlers;
- capability admissions;
- runtime handles;
- Semantic state references;
- Workbench/Studio object IDs.

### 8.9 Determinism requirements

same UiIr input + same ID policy = same UiProjectionArtifact identity graph

Forbidden:
no randomness
no wall-clock time
no network
no file I/O
no command execution
no host effects
no global mutable state
no nondeterministic map iteration affecting output IDs

### 8.10 Traceability requirements

Projection identity must preserve traceability to source UiIr where current structures support it.

UiProjectedNode.source_ir_node_id remains the canonical trace back to UiIr.
UiProjectionArtifact.source_ir_root remains the canonical trace to source root.
Traceability is not truth.
Traceability is not admission.
Traceability is not renderer readiness.

### 8.11 Authority boundary

UI may display truth. UI does not become truth.

Projection IDs do not define Semantic truth.
Projection IDs do not define runtime state.
Projection IDs do not define verifier admission.
Projection IDs do not define capability admission.
Projection IDs do not define release readiness.

### 8.12 Quad-state boundary

Unknown must not be dropped by identity policy.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.
Identity equality must not collapse N/F/T/S state meaning.

### 8.13 Implementation gate requirements

Future gate:
R12-UI-PROJECTION-BUILDER-ID-POLICY-SEED

Allowed future implementation:
- replace or formalize UiProjectionArtifactId construction;
- add tests for deterministic artifact ID;
- add tests proving node ID derivation stability;
- keep current projection inert.

Forbidden in future implementation unless separately authorized:
- renderer/backend;
- layout/draw/event;
- parser/verifier/VM/runtime;
- Workbench/Studio;
- dependencies;
- From/TryFrom UiIr;
- ProjectionBuilder type.

### 8.14 Explicit non-scope

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
No parser/verifier/VM/runtime integration.
No Workbench/Studio.

### 8.15 Admission Guard table

| Area | Policy state | Admission Guard classification | Status |
|---|---|---|---|
| ID policy document | Present | ADMITTED | PASS |
| UiProjectedNodeId policy | Structural deterministic | ADMITTED | PASS |
| UiProjectionArtifactId current seed constant | SEED_ONLY | ADMITTED_WITH_LIMIT | PASS |
| UiProjectionArtifactId future policy | Required | FUTURE_ONLY_NOT_AUTHORIZED_HERE | PASS |
| Reference IDs | Inert structural refs | ADMITTED_WITH_BOUNDARY | PASS |
| Runtime identity | Out of scope | FORBIDDEN | PASS |
| Renderer identity | Out of scope | FORBIDDEN | PASS |
| Workbench/Studio identity | Out of scope | FORBIDDEN | PASS |
| Random IDs | Forbidden | FORBIDDEN | PASS |
| Timestamp IDs | Forbidden | FORBIDDEN | PASS |
| Host/session IDs | Forbidden | FORBIDDEN | PASS |
| Source changes in this PR | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

### 8.16 Final decision

Final decision:
APPROVED — R12 UI Projection Builder ID Policy is defined as a deterministic projection-layer identity policy. UiProjectedNodeId remains structural and traceable to UiIrNodeId in the current seed. UiProjectionArtifactId::new(1) is classified as SEED_ONLY and requires a future explicit implementation gate before widening.
