# POST-UI Roadmap Next Lane Selection After Layout Geometry Seed Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Geometry Seed line.

## 2. DNA Alignment
docs/dna inspected: YES
DNA files inspected: SEMANTIC_UI_DNA.md
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- geometry seed remains inert renderer-local metadata;
- constraints boundary must remain planning/documentation only;
- constraints boundary must not introduce constraints source;
- constraints boundary must not introduce solver authority;
- constraints boundary must not introduce sizing authority;
- constraints boundary must not introduce draw/event/backend authority;
- constraints boundary must not introduce runtime/verifier/VM/capability authority;
- constraints boundary must not introduce proof/debugger authority;
- constraints boundary must not introduce Workbench/Studio integration.

## 3. Closed Basis
#989 — roadmap selected geometry seed
#990 — layout geometry seed source
#991 — layout geometry seed closeout
#992 — layout geometry seed ledger audit

## 4. Geometry Seed State
Geometry seed is closed as minimal inert renderer-local geometry metadata. It implements deterministic geometry model/node identity and integer-only rect metadata without implementing a full geometry solver, constraint solver, sizing algorithm, layout engine rewrite, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 5. Project #2 State
| Item | Status | Track | Wave | Type | Risk | Boundary | Gate | Evidence | Depends on |
|---|---|---|---|---|---|---|---|---|---|
| #989 | Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | #988 |
| #990 | Done | POST-UI | R12 | Code | High | Renderer | PRReady | PR | #989 |
| #991 | Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | #990 |
| #992 | Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | #991 |

## 6. Candidate Lanes
| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Constraints Boundary | Selected | Geometry seed is now implemented and ledger-audited as inert renderer-local geometry metadata. The next structurally safe step is to define a constraints boundary before any constraints source or solver behavior exists. | Medium | Selected |
| Layout Constraints Seed | Deferred | Too early. A constraints boundary must exist first. | High | Deferred |
| Geometry Solver Boundary | Deferred / too early | Solver boundary is premature before constraints boundary exists. | High | Deferred |
| Sizing Boundary | Deferred | Sizing should wait until constraints boundary is defined. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout constraints are explicitly bounded. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside current layout metadata work. | High | Deferred |
| Layout Geometry Consolidation Audit | Deferred | Useful later, but the immediate architectural pressure point is constraints boundary. | Low | Deferred |

## 7. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer/layout/geometry inertness.
3. Must not implement constraints in the roadmap selection PR.
4. Must not introduce solver behavior.
5. Must not introduce sizing behavior.
6. Must not introduce draw/event/backend.
7. Must not introduce runtime/verifier/VM/capability authority.
8. Must build naturally on closed geometry seed and audit.
9. Must be boundary-gated before any constraints source exists.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-CONSTRAINTS-BOUNDARY-LINE-FULL-PACKAGE

This selection is planning-only.
This selection does not implement constraints source.
This selection does not implement constraint solver behavior.
This selection only authorizes the next boundary package to be prepared under a separate gate.

## 9. Deferred Lanes
- Layout Constraints Seed
- Geometry Solver Boundary
- Sizing Boundary
- Event Boundary
- Backend Boundary
- Layout Geometry Consolidation Audit

## 10. Admission Guard
This selection is planning-only.
This selection does not modify layout.rs.
This selection does not add tests.
This selection does not introduce constraints, solver, or sizing behavior.
This selection only authorizes the next boundary package to be prepared under a separate gate.

## 11. Non-Scope
Allowed future boundary scope:
- define what future constraints may consume;
- define what future constraints may produce;
- define what future constraints must not do;
- define constraints as metadata/boundary concept, not solver behavior;
- define separation from sizing, solving, draw, event, backend, runtime, verifier, capability, proof/debugger, Workbench/Studio.

Forbidden future boundary scope:
- actual constraint structs;
- actual constraint solver;
- sizing algorithm;
- layout solving;
- layout engine rewrite;
- draw commands;
- event handling;
- backend/WGPU/winit/Tauri;
- runtime/verifier/VM calls;
- capability admission;
- proof/debugger authority;
- Workbench/Studio integration.

## 12. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout geometry seed audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-CONSTRAINTS-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement constraints source, constraint solver behavior, sizing behavior, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.
