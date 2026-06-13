# R12 UI Renderer Layout Sizing Boundary

## 1. Purpose
This document defines the R12 UI Renderer Layout Sizing Boundary after the completed and audited renderer layout constraints seed line.

## 2. DNA Alignment
docs/dna inspected: NO directory in this repository
DNA files inspected:
- docs/DNA.md
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- geometry seed remains inert renderer-local metadata;
- constraints seed remains inert renderer-local metadata declarations;
- sizing boundary remains planning/documentation only;
- no sizing implementation;
- no sizing algorithm implementation;
- no measuring algorithm implementation;
- no constraint solver implementation;
- no layout solving implementation;
- no draw/event/backend authority;
- no runtime/verifier/VM/capability authority;
- no proof/debugger authority;
- no Workbench/Studio integration.

## 3. Closed Basis
#997 — roadmap selected constraints seed
#998 — layout constraints seed source
#999 — layout constraints seed closeout
#1000 — layout constraints seed ledger audit
#1001 — roadmap selected sizing boundary

## 4. Sizing Position in Pipeline
UiProjectionArtifact
  ↓
UiRenderModel
  ↓
Renderer presentation models
  ↓
UiLayoutModel
  ↓
UiLayoutGeometryModel
  ↓
UiLayoutConstraintsModel
  ↓
Future sizing model
  ↓
Future solver / layout refinement layers, not part of this boundary
  ↓
Future draw/backend/event layers, not part of this boundary

## 5. Sizing Boundary Definition
Sizing Boundary defines where a future sizing layer may sit and what authority it must not receive.

It does not implement sizing source.

It does not implement sizing structs.

It does not implement sizing algorithm behavior.

It does not implement constraint solver behavior.

It does not implement layout solving.

## 6. Allowed Future Inputs
The boundary may admit future consumption of:

- UiLayoutModel
- UiLayoutGeometryModel
- UiLayoutConstraintsModel
- layout nodes
- geometry nodes
- constraint declarations
- source layout/geometry/constraint references exposed by existing APIs

No source code input contract is implemented in this PR.
This is a boundary-level declaration only.

## 7. Allowed Future Outputs
The boundary may describe future sizing outputs as:

- deterministic sizing model;
- deterministic sizing entries;
- source layout/geometry/constraint references;
- intrinsic size metadata;
- minimum size metadata;
- maximum size metadata;
- preferred size metadata;
- available size metadata;
- unresolved/deferred size metadata;
- audit metadata for future solver admission.

No sizing structs, sizing IDs, sizing entries, sizing model, sizing functions, sizing tests, or sizing algorithm are implemented in this PR.

## 8. Future Sizing Categories
The boundary may name future conceptual categories, strictly as non-implemented concepts:

- unresolved size;
- intrinsic size;
- minimum size;
- maximum size;
- preferred size;
- available size;
- content size;
- container size;
- fit/fill preference;
- fixed size declaration;
- flexible size declaration;

These are future conceptual categories only.
They are not implemented by this PR.

## 9. Explicit Non-Authority Rules
Forbidden sizing authority:

- sizing source;
- sizing structs;
- sizing IDs;
- sizing functions;
- sizing tests;
- sizing algorithm;
- measure algorithm;
- size-to-fit algorithm;
- constraint solving;
- constraint satisfaction;
- layout solving;
- layout engine rewrite;
- geometry mutation;
- constraints mutation;
- draw commands;
- rasterization;
- GPU/backend calls;
- WGPU/winit/Tauri;
- event dispatch;
- callbacks/handlers;
- action execution;
- effect authorization;
- runtime/verifier/VM calls;
- capability admission;
- semantic truth authority;
- proof/debugger authority;
- Workbench/Studio integration.

## 10. Deferred Implementation
Deferred to a future source package:
R12-UI-RENDERER-LAYOUT-SIZING-SEED-LINE-FULL-PACKAGE

That future package must not proceed until this boundary is closed and audited.

## 11. Candidate Future Seed
The only future implementation allowed by this boundary is a separate sizing seed package under an explicit later source gate.

This document does not authorize creating that source package.

## 12. Admission Guard
| Surface | Boundary status | Admission classification | Status |
|---|---|---|---|
| future sizing metadata | allowed only after source gate | ADMITTED FUTURE | PASS |
| current sizing implementation | absent | DEFERRED | PASS |
| sizing algorithm | forbidden | FORBIDDEN | PASS |
| measuring algorithm | forbidden | FORBIDDEN | PASS |
| constraint solver | forbidden | FORBIDDEN | PASS |
| constraint satisfaction | forbidden | FORBIDDEN | PASS |
| layout solving | forbidden | FORBIDDEN | PASS |
| draw commands | forbidden | FORBIDDEN | PASS |
| backend API | forbidden | FORBIDDEN | PASS |
| event handlers | forbidden | FORBIDDEN | PASS |
| runtime/verifier/VM | forbidden | FORBIDDEN | PASS |
| capability admission | forbidden | FORBIDDEN | PASS |
| semantic truth authority | forbidden | FORBIDDEN | PASS |
| proof/debugger authority | forbidden | FORBIDDEN | PASS |

## 13. Evidence Matrix
| Area | Current state | Classification | Status |
|---|---|---|---|
| Sizing boundary | Documented | ADMITTED | PASS |
| Sizing implementation | Not implemented | DEFERRED | PASS |
| Sizing seed | Future package only | DEFERRED | PASS |
| Sizing algorithm | Not implemented | FORBIDDEN | PASS |
| Measuring algorithm | Not implemented | FORBIDDEN | PASS |
| Constraint solver | Not implemented | FORBIDDEN | PASS |
| Constraint satisfaction | Not implemented | FORBIDDEN | PASS |
| Layout solving | Not implemented | FORBIDDEN | PASS |
| Layout engine rewrite | Not implemented | FORBIDDEN | PASS |
| Draw/render backend | Not implemented | FORBIDDEN | PASS |
| Event dispatch | Not implemented | FORBIDDEN | PASS |
| Action execution | Not implemented | FORBIDDEN | PASS |
| Effect authorization | Not implemented | FORBIDDEN | PASS |
| Runtime/verifier/VM | Not implemented | FORBIDDEN | PASS |
| Capability admission | Not implemented | FORBIDDEN | PASS |
| Proof/debugger authority | Not implemented | FORBIDDEN | PASS |
| Workbench/Studio | Not implemented | FORBIDDEN | PASS |
| Dependency additions | None | FORBIDDEN | PASS |

## 14. Final Decision
Final decision:
PASS — R12 UI Renderer Layout Sizing Boundary defined.

This boundary admits future sizing only as deterministic renderer-local layout metadata/result declarations.

This PR does not implement sizing source, sizing structs, sizing IDs, sizing functions, sizing tests, sizing algorithm behavior, measuring algorithm behavior, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.
