# R12 UI Renderer Layout Geometry Boundary

## 1. Purpose
This document defines the R12 UI Renderer Layout Geometry Boundary before any geometry source implementation exists.

This boundary is docs-only.

It does not implement geometry, coordinates, sizing, constraints, solver logic, layout engine behavior, draw commands, event dispatch, backend rendering, runtime/verifier/VM integration, capability admission, or Workbench/Studio integration.

## 2. DNA Alignment
docs/dna inspected: YES
DNA files inspected: SEMANTIC_UI_DNA.md
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout inspection presentation remains read-only observability;
- geometry boundary is docs-only;
- geometry must not gain semantic authority;
- geometry must not gain effect/capability authority;
- geometry must not introduce draw/event/backend/runtime authority;
- Workbench/Studio remains out of scope.

## 3. Closed Layout Basis
Accepted closed layout lineage:

#977 — roadmap selected layout inspection presentation
#978 — initial layout inspection presentation source
#979 — recovery test fix 1
#980 — recovery test fix 2 / final green state
#981 — original layout inspection presentation closeout
#982 — corrective recovery closeout
#983 — layout inspection presentation ledger audit after recovery correction
#984 — roadmap selected layout geometry boundary

## 4. Boundary Position in Pipeline
UiProjectionArtifact
  ↓
UiRenderModel
  ↓
UiLayoutModel
  ↓
UiLayoutInspectionPresentation
  ↓
Future UiLayoutGeometryModel / geometry seed

Future UiLayoutGeometryModel does not exist in this boundary PR.

Geometry may only be downstream of locked layout data.

Geometry must not modify UiLayoutModel.

Geometry must not modify UiRenderModel.

Geometry must not modify UiProjectionArtifact.

Geometry must not become a source of semantic truth.

## 5. Allowed Future Inputs
Allowed future geometry inputs:
- UiLayoutModel;
- UiLayoutInspectionPresentation;
- deterministic layout node identity;
- deterministic layout slot identity;
- source render/projection/IR references exposed by public APIs;
- future explicitly admitted geometry configuration, if separately bounded and audited.

Inputs are read-only.

## 6. Allowed Future Outputs
Potential future geometry outputs, only after a separate seed gate:
- deterministic geometry model identity;
- deterministic geometry node identity;
- deterministic geometry slot/box identity;
- structural bounding metadata;
- structural ordering metadata;
- validation/reporting metadata.

This boundary does not create those output types.

## 7. Forbidden Authority
Forbidden:
- semantic truth authority;
- verifier authority;
- runtime/VM authority;
- capability admission;
- action execution;
- effect authorization;
- event dispatch;
- backend rendering;
- draw command emission;
- WGPU/winit/Tauri integration;
- Workbench/Studio integration;
- proof/debugger authority.

## 8. Geometry Non-Authority Rules
Geometry may describe structural placement metadata only after future admission.

Geometry may not:
- decide whether UI IR is valid;
- decide whether capabilities are allowed;
- execute actions;
- authorize effects;
- dispatch events;
- render to backend;
- call runtime/verifier/VM;
- mutate source layout/render/projection artifacts;
- hide conflicts;
- override diagnostics;
- become a semantic authority.

## 9. Deferred State
Deferred:
- geometry seed;
- geometry public API;
- geometry public API lock;
- coordinates;
- sizing;
- constraints;
- solver logic;
- layout engine behavior;
- draw integration;
- event integration;
- backend integration;
- Workbench/Studio integration.

## 10. Future Geometry Seed Prerequisites
A future geometry seed package must prove:

1. It was selected by POST-UI roadmap selection or authorized by this boundary closeout/audit.
2. It uses only public layout APIs.
3. It is deterministic.
4. It preserves source layout/render/projection/IR references where exposed.
5. It introduces no draw/event/backend/runtime/capability authority.
6. It has public tests.
7. It has no dependency additions unless separately justified and audited.
8. It remains source-local and renderer-local.
9. It has a closeout.
10. It has a ledger audit before further geometry expansion.

## 11. Risk Matrix
| Risk | Reason | Mitigation | Status |
|---|---|---|---|
| Geometry becomes layout engine | Allowed scope is only geometry placement data | Keep engine logic deferred | Mitigated by boundary rules |
| Geometry becomes draw model | Output is geometry only | Forbid draw command emission | Mitigated by boundary rules |
| Geometry dispatches events | Output is structural only | Forbid event dispatch | Mitigated by boundary rules |
| Geometry calls backend | Backend is unauthorized | Forbid backend calls | Mitigated by boundary rules |
| Geometry calls runtime/verifier/VM | Semantics belong upstream | Forbid runtime calls | Mitigated by boundary rules |
| Geometry admits capabilities | Capability admission belongs upstream | Forbid admission | Mitigated by boundary rules |
| Geometry mutates layout/render/projection artifacts | Must remain read-only observability downstream | Forbid mutation | Mitigated by boundary rules |
| Geometry hides diagnostics | Diagnostics are upstream | Forbid overriding diagnostics | Mitigated by boundary rules |
| Geometry creates dependency drift | Vendor dependencies are banned | Block non-audited dependencies | Mitigated by boundary rules |

## 12. Admission Guard Table
| Area | Boundary decision | Admission state | Status |
|---|---|---|---|
| geometry boundary | defined | admitted | PASS |
| geometry source | not implemented | deferred | BLOCKED |
| coordinates/sizing | not implemented | deferred | BLOCKED |
| constraints/solver | not implemented | deferred | BLOCKED |
| layout engine | not implemented | deferred | BLOCKED |
| draw commands | forbidden | blocked | BLOCKED |
| event dispatch | forbidden | blocked | BLOCKED |
| backend rendering | forbidden | blocked | BLOCKED |
| runtime/verifier/VM | forbidden | blocked | BLOCKED |
| capability admission | forbidden | blocked | BLOCKED |
| Workbench/Studio | forbidden | blocked | BLOCKED |
| dependency additions | forbidden | blocked | BLOCKED |

## 13. Non-Scope
This boundary defines docs-only governance constraints. It does not introduce code or dependencies.

## 14. Final Decision
Final decision:
PASS — R12 UI Renderer Layout Geometry Boundary is defined.

This boundary is docs-only and introduces no geometry source implementation.

It defines geometry as a future downstream renderer-local structural layer after UiLayoutModel and UiLayoutInspectionPresentation.

It does not implement coordinates, sizing, constraints, solver logic, layout engine behavior, draw commands, event dispatch, backend rendering, runtime/verifier/VM integration, capability admission, proof/debugger authority, Workbench/Studio integration, or dependency additions.
