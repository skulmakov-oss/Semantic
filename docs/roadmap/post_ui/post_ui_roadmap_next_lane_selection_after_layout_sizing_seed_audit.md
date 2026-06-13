# POST-UI Roadmap Next Lane Selection After Layout Sizing Seed Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Sizing Seed line.

## 2. DNA Alignment
DNA inspected: YES
DNA source path: docs/dna/SEMANTIC_UI_DNA.md; docs/DNA.md present as repository fallback
docs/dna directory present: YES
docs/DNA.md present: YES
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- geometry seed remains inert renderer-local metadata;
- constraints seed remains inert renderer-local metadata declarations;
- sizing seed remains inert renderer-local metadata/result declarations;
- sizing algorithm boundary must remain docs-only;
- sizing algorithm boundary must not implement sizing algorithm source;
- sizing algorithm boundary must not implement measuring algorithm source;
- sizing algorithm boundary must not implement size-to-fit behavior;
- sizing algorithm boundary must not introduce intrinsic/content size calculation;
- sizing algorithm boundary must not introduce constraint solver authority;
- sizing algorithm boundary must not introduce constraint satisfaction authority;
- sizing algorithm boundary must not introduce layout solving;
- sizing algorithm boundary must not introduce draw/event/backend authority;
- sizing algorithm boundary must not introduce runtime/verifier/VM/capability authority;
- sizing algorithm boundary must not introduce proof/debugger authority;
- sizing algorithm boundary must not introduce Workbench/Studio integration;
- this roadmap selection does not implement algorithm source.

## 3. Closed Basis
- #1005 — roadmap selected sizing seed
- #1006 — layout sizing seed source
- #1007 — layout sizing seed closeout
- #1008 — layout sizing seed ledger audit

## 4. Sizing Seed State
Sizing seed is closed as minimal inert renderer-local layout sizing metadata/result declarations. It implements deterministic sizing entries and kind/state metadata without implementing sizing algorithm behavior, measuring algorithm behavior, size-to-fit behavior, constraint solver behavior, constraint satisfaction, layout solving, layout engine rewrite, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 5. Project #2 State
| Item | Status | Track | Wave | Type | Risk | Boundary | Gate | Evidence | Depends on | Item count | Duplicate count |
|---|---|---|---|---|---|---|---|---|---|---:|---:|
| #1005 | Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | #1004 | 1 | 0 |
| #1006 | Done | POST-UI | R12 | Code | High | Renderer | PRReady | PR | #1005 | 1 | 0 |
| #1007 | Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | #1006 | 1 | 0 |
| #1008 | Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | #1007 | 1 | 0 |

## 6. Candidate Lanes
| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Sizing Algorithm Boundary | Selected | Sizing boundary and sizing seed are completed and ledger-audited. The next structurally safe step is to define a sizing algorithm boundary before any sizing algorithm, measuring algorithm, size-to-fit behavior, constraint solver, or layout-solving source exists. | Medium | Selected |
| Sizing Algorithm Seed / Source | Deferred / too early | Too early. A sizing algorithm boundary must exist before any sizing algorithm source package. | High | Deferred |
| Measuring Boundary | Deferred | Measuring is adjacent but should remain separated from algorithm boundary. | Medium | Deferred |
| Constraint Solver Boundary | Deferred / too early | Solver boundary remains premature until sizing algorithm authority is bounded and separated from solver authority. | High | Deferred |
| Layout Solving Boundary | Deferred / too early | Layout solving is higher-authority and should wait until algorithm and solver boundaries are separately handled. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout metadata and algorithm boundaries are more mature. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside current layout metadata and algorithm boundary work. | High | Deferred |
| Layout Metadata Consolidation Audit | Deferred | Useful later, but the immediate architectural pressure point is bounding sizing algorithm authority. | Medium | Deferred |

## 7. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer/layout/geometry/constraints/sizing seed inertness.
3. Must not implement algorithm source in the roadmap selection PR.
4. Must not introduce sizing algorithm behavior.
5. Must not introduce measuring algorithm behavior.
6. Must not introduce size-to-fit behavior.
7. Must not introduce constraint solver behavior.
8. Must not introduce constraint satisfaction behavior.
9. Must not introduce layout solving.
10. Must not introduce draw/event/backend.
11. Must not introduce runtime/verifier/VM/capability authority.
12. Must build naturally on closed sizing seed and audit.
13. Must be boundary-gated before any sizing algorithm source exists.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-SIZING-ALGORITHM-BOUNDARY-LINE-FULL-PACKAGE

## 9. Deferred Lanes
- Sizing Algorithm Seed / Source
- Measuring Boundary
- Constraint Solver Boundary
- Layout Solving Boundary
- Event Boundary
- Backend Boundary
- Layout Metadata Consolidation Audit

## 10. Admission Guard
Sizing algorithm boundary is planning-only. It authorizes only the next boundary package to be prepared under a separate gate. It does not implement sizing algorithm source, measuring algorithm source, size-to-fit behavior, intrinsic/content size calculation, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 11. Non-Scope
- no source changes
- no test changes
- no docs/DNA.md changes
- no docs/dna changes
- no agent skill changes
- no Cargo.toml / Cargo.lock changes
- no dependency additions
- no sizing algorithm source
- no measuring algorithm source
- no size-to-fit behavior
- no intrinsic/content size calculation
- no constraint solver
- no constraint satisfaction
- no layout solving
- no draw/event/backend implementation
- no runtime/verifier/VM integration
- no capability admission
- no proof/debugger authority
- no Workbench/Studio integration

## 12. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout sizing seed audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-SIZING-ALGORITHM-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement sizing algorithm source, measuring algorithm source, size-to-fit behavior, intrinsic/content size calculation, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.
