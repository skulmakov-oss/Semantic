# POST-UI Roadmap Next Lane Selection After Layout Constraints Seed Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Constraints Seed line.

## 2. DNA Alignment
docs/dna inspected: YES
DNA files inspected:
- docs/dna/SEMANTIC_UI_DNA.md
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- geometry seed remains inert renderer-local metadata;
- constraints seed remains inert renderer-local metadata declarations;
- sizing boundary must remain planning/documentation only;
- sizing boundary must not introduce sizing source;
- sizing boundary must not introduce sizing algorithm authority;
- sizing boundary must not introduce constraint solver authority;
- sizing boundary must not introduce layout solving;
- sizing boundary must not introduce draw/event/backend authority;
- sizing boundary must not introduce runtime/verifier/VM/capability authority;
- sizing boundary must not introduce proof/debugger authority;
- sizing boundary must not introduce Workbench/Studio integration.

## 3. Closed Basis
- #997 — roadmap selected constraints seed
- #998 — layout constraints seed source
- #999 — layout constraints seed closeout
- #1000 — layout constraints seed ledger audit

## 4. Constraints Seed State
Constraints seed is closed as minimal inert renderer-local layout constraints metadata. It implements deterministic constraint declarations and kind/state metadata without implementing constraint solver behavior, constraint satisfaction, sizing behavior, layout solving, layout engine rewrite, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 5. Project #2 State
| Item | Status | Track | Wave | Type | Risk | Boundary | Gate | Evidence | Depends on |
|---|---|---|---|---|---|---|---|---|---|
| #997 | Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | #996 |
| #998 | Done | POST-UI | R12 | Code | High | Renderer | PRReady | PR | #997 |
| #999 | Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | #998 |
| #1000 | Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | #999 |

## 6. Candidate Lanes
| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Sizing Boundary | Selected | Geometry seed and constraints seed are both implemented, closed, and ledger-audited as inert renderer-local metadata. The next structurally safe step is to define a sizing boundary before any sizing source, sizing algorithm, solver, or layout-solving behavior exists. | Medium | Selected |
| Layout Sizing Seed | Deferred | Too early. A sizing boundary must exist first. | High | Deferred |
| Constraint Solver Boundary | Deferred / too early | Solver boundary is premature until sizing boundary is defined and sizing authority is explicitly separated from solver authority. | High | Deferred |
| Layout Solving Boundary | Deferred / too early | Layout solving is too high-authority and should wait until sizing boundary and sizing seed are handled. | High | Deferred |
| Geometry Solver Boundary | Deferred / too early | Geometry solver should wait until sizing and constraints relationships are more explicitly bounded. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout metadata layers are more mature. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside current layout metadata work. | High | Deferred |
| Layout Metadata Consolidation Audit | Deferred | Useful later, but the immediate architectural pressure point is sizing boundary. | Medium | Deferred |

## 7. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer/layout/geometry/constraints inertness.
3. Must not implement sizing in the roadmap selection PR.
4. Must not introduce sizing algorithm behavior.
5. Must not introduce constraint solver behavior.
6. Must not introduce layout solving.
7. Must not introduce draw/event/backend.
8. Must not introduce runtime/verifier/VM/capability authority.
9. Must build naturally on closed constraints seed and audit.
10. Must be boundary-gated before any sizing source exists.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-SIZING-BOUNDARY-LINE-FULL-PACKAGE

This selection is planning-only.
This selection does not implement sizing source.
This selection does not implement sizing structs.
This selection does not implement sizing functions.
This selection does not implement sizing tests.
This selection does not implement sizing algorithm behavior.
This selection does not implement constraint solver behavior.
This selection does not implement layout solving.
This selection only authorizes the next boundary package to be prepared under a separate gate.

## 9. Deferred Lanes
Deferred lanes:
- Layout Sizing Seed
- Constraint Solver Boundary
- Layout Solving Boundary
- Geometry Solver Boundary
- Event Boundary
- Backend Boundary
- Layout Metadata Consolidation Audit

## 10. Admission Guard
| Area | Final state | Admission Guard classification | Status |
|---|---|---|---|
| sizing boundary | DOCUMENTED / ADMITTED | ADMITTED | PASS |
| sizing source | ABSENT / DEFERRED | FORBIDDEN UNTIL GATED | PASS |
| sizing structs | ABSENT / DEFERRED | FORBIDDEN UNTIL GATED | PASS |
| sizing functions | ABSENT / DEFERRED | FORBIDDEN UNTIL GATED | PASS |
| sizing tests | ABSENT / DEFERRED | FORBIDDEN UNTIL GATED | PASS |
| sizing algorithm | ABSENT / FORBIDDEN | FORBIDDEN | PASS |
| constraint solver | ABSENT / FORBIDDEN | FORBIDDEN | PASS |
| constraint satisfaction | ABSENT / FORBIDDEN | FORBIDDEN | PASS |
| layout solving | ABSENT / FORBIDDEN | FORBIDDEN | PASS |
| draw/event/backend | ABSENT / FORBIDDEN | FORBIDDEN | PASS |
| runtime/verifier/VM | ABSENT / FORBIDDEN | FORBIDDEN | PASS |
| capability admission | ABSENT / FORBIDDEN | FORBIDDEN | PASS |
| proof/debugger authority | ABSENT / FORBIDDEN | FORBIDDEN | PASS |
| Workbench/Studio | ABSENT / FORBIDDEN | FORBIDDEN | PASS |

## 11. Non-Scope
- no source changes
- no test changes
- no docs/dna changes
- no agent skill changes
- no Cargo.toml / Cargo.lock changes
- no dependency additions
- no sizing source implementation
- no sizing structs
- no sizing algorithm
- no constraint solver
- no constraint satisfaction algorithm
- no layout solving
- no draw/event/backend implementation
- no runtime/verifier/VM integration
- no capability admission
- no proof/debugger authority
- no Workbench/Studio integration

## 12. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout constraints seed audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-SIZING-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement sizing source, sizing structs, sizing functions, sizing tests, sizing algorithm behavior, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.
