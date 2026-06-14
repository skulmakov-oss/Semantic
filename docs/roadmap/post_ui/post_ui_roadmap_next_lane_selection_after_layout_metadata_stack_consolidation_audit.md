# POST-UI Roadmap Next Lane Selection After Layout Metadata Stack Consolidation Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed R12 UI Renderer Layout Metadata Stack Consolidation Audit.

## 2. DNA Alignment
- DNA inspected: YES
- DNA source path: `docs/dna/SEMANTIC_UI_DNA.md`
- docs/dna directory present: YES
- docs/DNA.md present: NO
- DNA conflicts detected: NONE
- DNA-driven constraints applied:
  - renderer/UI remains downstream;
  - layout metadata stack remains renderer-local;
  - geometry seed remains inert renderer-local metadata;
  - constraints seed remains inert renderer-local metadata declarations;
  - sizing seed remains inert renderer-local metadata/result declarations;
  - sizing algorithm seed remains deterministic renderer-local metadata derivation substrate;
  - measuring boundary remains docs-only and audited;
  - measuring seed remains deterministic renderer-local measurement metadata/request substrate;
  - metadata stack consolidation audit is complete;
  - size-to-fit boundary selection must remain docs-only;
  - size-to-fit boundary selection must not introduce source behavior;
  - size-to-fit boundary selection must not introduce fit/fill/shrink/grow behavior;
  - size-to-fit boundary selection must not introduce intrinsic/content size calculation as executable behavior;
  - size-to-fit boundary selection must not introduce constraint solver authority;
  - size-to-fit boundary selection must not introduce constraint satisfaction authority;
  - size-to-fit boundary selection must not introduce layout solving;
  - size-to-fit boundary selection must not introduce real measuring;
  - size-to-fit boundary selection must not introduce draw/event/backend authority;
  - size-to-fit boundary selection must not introduce runtime/verifier/VM/capability authority;
  - size-to-fit boundary selection must not introduce proof/debugger authority;
  - size-to-fit boundary selection must not introduce Workbench/Studio integration;
  - this roadmap PR must remain docs-only.

## 3. Closed Basis
- #1022 — roadmap selected measuring seed
- #1023 — layout measuring seed source
- #1024 — layout measuring seed closeout
- #1025 — layout measuring seed ledger audit
- #1026 — roadmap selected metadata stack consolidation audit
- #1027 — layout metadata stack consolidation audit

## 4. Metadata Stack State
The current renderer layout metadata stack is consolidated as deterministic renderer-local metadata from layout through geometry, constraints, sizing, sizing algorithm, and measuring seed.

The stack remains source-reference-preserving, non-mutating, metadata-only, and does not implement real measuring, size-to-fit behavior, intrinsic/content size calculation as executable behavior, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

After this consolidation, the next selected lane is a docs-only size-to-fit boundary. That boundary must define authority before any future size-to-fit source package is admitted.

## 5. Project #2 State
- Project #2 item for this lane: pending creation under `#1027`
- Current verified related items:
  - `#1022` Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | `#1021`
  - `#1023` Done | POST-UI | R12 | Code | High | Renderer | PRReady | PR | `#1022`
  - `#1024` Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | `#1023`
  - `#1025` Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | `#1024`
  - `#1026` Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | `#1025`
  - `#1027` Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | `#1026`

## 6. Candidate Lanes
| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Size-to-Fit Boundary | Selected | The layout metadata stack has been consolidated and audited. The next structurally safe step is to define the size-to-fit authority boundary before any fit/fill/shrink/grow source behavior exists. | Medium | Selected |
| Size-to-Fit Seed / Source | Deferred / too early | A size-to-fit boundary must exist before any size-to-fit source package. | High | Deferred |
| Constraint Solver Boundary | Deferred | Constraint solver authority is higher-risk and should remain separated from size-to-fit boundary work. | High | Deferred |
| Layout Solving Boundary | Deferred / too early | Layout solving introduces placement/refinement authority and must wait until size-to-fit and solver boundaries are separately handled. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside the current layout metadata authority envelope. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout authority is more mature. | High | Deferred |
| Real Measuring Implementation | Deferred / forbidden for now | Measuring seed remains metadata/request substrate only. Real measurement remains explicitly outside the current authority envelope. | High | Deferred |

## 7. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer-local layout metadata stack boundaries.
3. Must preserve geometry seed inertness.
4. Must preserve constraints seed inertness.
5. Must preserve sizing seed inertness.
6. Must preserve sizing algorithm seed as metadata derivation substrate only.
7. Must preserve measuring seed as metadata/request substrate only.
8. Must build on the completed metadata stack consolidation audit.
9. Must not start the size-to-fit boundary package in this roadmap PR.
10. Must not introduce source changes.
11. Must not introduce test changes.
12. Must not introduce size-to-fit source.
13. Must not introduce fit/fill/shrink/grow behavior.
14. Must not introduce intrinsic/content size calculation as executable behavior.
15. Must not introduce real measuring.
16. Must not introduce constraint solver behavior.
17. Must not introduce constraint satisfaction behavior.
18. Must not introduce layout solving.
19. Must not introduce geometry/layout/sizing/constraints/measuring mutation.
20. Must not introduce draw/event/backend.
21. Must not introduce runtime/verifier/VM/capability authority.
22. Must select a boundary gate before any size-to-fit source exists.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-SIZE-TO-FIT-BOUNDARY-LINE-FULL-PACKAGE

This selection is planning-only.
This selection does not start the size-to-fit boundary package.
This selection does not implement size-to-fit source.
This selection does not implement fit/fill/shrink/grow behavior.
This selection does not implement intrinsic/content size calculation as executable behavior.
This selection does not implement real text/glyph/image/widget measurement.
This selection does not implement font/backend/GPU measurement.
This selection does not implement WGPU/winit/Tauri measurement.
This selection does not implement constraint solver behavior.
This selection does not implement constraint satisfaction.
This selection does not implement layout solving.
This selection does not mutate geometry/layout/sizing/constraints/measuring metadata.
This selection does not introduce draw/event/backend/runtime/capability/proof/Workbench/Studio authority.
This selection only authorizes the next docs-only boundary package to be prepared under a separate gate.

## 9. Deferred Lanes
- Size-to-Fit Seed / Source
- Constraint Solver Boundary
- Layout Solving Boundary
- Backend Boundary
- Event Boundary
- Real Measuring Implementation

## 10. Untracked Workspace Artifacts
Untracked workspace artifacts are treated as local-only, non-merged artifacts.
They must not be staged, committed, deleted, or merged by this roadmap selection PR.

Known artifacts:

- `.claude/`
- `examples/baseline/`
- `scratch/`

## 11. Admission Guard
| Area | Final state | Admission Guard classification | Status |
|---|---|---|---|
| size-to-fit boundary selection | planned only | ADMITTED FUTURE BOUNDARY | PASS |
| size-to-fit behavior | absent | FORBIDDEN | PASS |
| intrinsic/content size calculation | absent | FORBIDDEN | PASS |
| constraint solver | absent | FORBIDDEN | PASS |
| constraint satisfaction | absent | FORBIDDEN | PASS |
| layout solving | absent | FORBIDDEN | PASS |
| real measuring | absent | FORBIDDEN | PASS |
| draw/event/backend | absent | FORBIDDEN | PASS |
| runtime/verifier/VM | absent | FORBIDDEN | PASS |
| capability admission | absent | FORBIDDEN | PASS |
| proof/debugger authority | absent | FORBIDDEN | PASS |
| Workbench/Studio | absent | FORBIDDEN | PASS |

## 12. Non-Scope
- no source changes
- no test changes
- no size-to-fit boundary document in this PR
- no size-to-fit source
- no fit/fill/shrink/grow behavior
- no intrinsic/content size calculation as executable behavior
- no real measuring
- no font/backend/GPU measurement
- no WGPU/winit/Tauri measurement
- no constraint solver
- no constraint satisfaction algorithm
- no layout solving
- no geometry mutation
- no layout mutation
- no sizing metadata mutation
- no constraint mutation
- no measuring mutation
- no draw commands
- no event dispatch
- no backend/WGPU/winit/Tauri
- no runtime/verifier/VM integration
- no capability admission
- no action execution
- no effect authorization
- no proof/debugger authority
- no Workbench/Studio integration

## 13. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout metadata stack consolidation audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-SIZE-TO-FIT-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not start the size-to-fit boundary package, change source, change tests, implement size-to-fit source, implement fit/fill/shrink/grow behavior, implement intrinsic/content size calculation as executable behavior, implement real measuring, implement constraint solver behavior, implement constraint satisfaction, implement layout solving, mutate geometry/layout/sizing/constraints/measuring metadata, or introduce draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
