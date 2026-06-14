# POST-UI Roadmap Next Lane Selection After Layout Measuring Seed Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Measuring Seed line.

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
  - consolidation audit must remain docs-only;
  - consolidation audit must not introduce source behavior;
  - consolidation audit must not introduce size-to-fit authority;
  - consolidation audit must not introduce intrinsic/content size calculation as executable behavior;
  - consolidation audit must not introduce constraint solver authority;
  - consolidation audit must not introduce constraint satisfaction authority;
  - consolidation audit must not introduce layout solving;
  - consolidation audit must not introduce draw/event/backend authority;
  - consolidation audit must not introduce runtime/verifier/VM/capability authority;
  - consolidation audit must not introduce proof/debugger authority;
  - consolidation audit must not introduce Workbench/Studio integration;
  - this roadmap PR must remain docs-only.

## 3. Closed Basis
- #1022 — roadmap selected measuring seed
- #1023 — layout measuring seed source
- #1024 — layout measuring seed closeout
- #1025 — layout measuring seed ledger audit

## 4. Layout Metadata Stack State
The current renderer layout metadata stack includes deterministic renderer-local metadata layers for layout, geometry, constraints, sizing, sizing algorithm, and measuring seed.

The stack remains metadata-only and does not implement real measuring, size-to-fit behavior, intrinsic/content size calculation as executable behavior, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Before introducing a higher-authority layer such as size-to-fit, constraint solving, or layout solving, the next selected lane is a docs-only consolidation audit across the metadata stack.

## 5. Project #2 State
- Project #2 item for this lane: pending creation under `#1025`
- Current verified related items:
  - `#1022` Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | `#1021`
  - `#1023` Done | POST-UI | R12 | Code | High | Renderer | PRReady | PR | `#1022`
  - `#1024` Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | `#1023`
  - `#1025` Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | `#1024`

## 6. Candidate Lanes
| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Metadata Stack Consolidation Audit | Selected | The layout metadata stack now contains multiple deterministic renderer-local layers: geometry, constraints, sizing, sizing algorithm, measuring boundary, and measuring seed. Before introducing size-to-fit, solver, or layout-solving authority, the structurally safest next step is a docs-only consolidation audit across the whole metadata stack. | Medium | Selected |
| Size-to-Fit Boundary | Deferred | Size-to-fit is the next likely functional authority layer, but it should wait until the existing metadata stack is consolidated and audited as one chain. | High | Deferred |
| Constraint Solver Boundary | Deferred / too early | Constraint solver authority is higher-risk than metadata consolidation and remains premature before size-to-fit and stack consolidation are handled. | High | Deferred |
| Layout Solving Boundary | Deferred / too early | Layout solving would introduce placement/refinement authority and must wait until fit and solver boundaries are separately gated. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside the metadata stack and should not be introduced before layout authority is consolidated. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout metadata and authority boundaries are stable. | High | Deferred |
| Measuring Real Implementation | Deferred / forbidden for now | Measuring seed is metadata/request substrate only. Real measurement remains explicitly outside the current authority envelope. | High | Deferred |

## 7. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer-local layout metadata stack boundaries.
3. Must preserve geometry seed inertness.
4. Must preserve constraints seed inertness.
5. Must preserve sizing seed inertness.
6. Must preserve sizing algorithm seed as metadata derivation substrate only.
7. Must preserve measuring seed as metadata/request substrate only.
8. Must not perform the consolidation audit in this roadmap PR.
9. Must not introduce source changes.
10. Must not introduce test changes.
11. Must not introduce size-to-fit behavior.
12. Must not introduce intrinsic/content size calculation as executable behavior.
13. Must not introduce constraint solver behavior.
14. Must not introduce constraint satisfaction behavior.
15. Must not introduce layout solving.
16. Must not introduce real measuring.
17. Must not introduce draw/event/backend.
18. Must not introduce runtime/verifier/VM/capability authority.
19. Must select an audit gate before higher-authority layout work.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-METADATA-STACK-CONSOLIDATION-AUDIT-PR

This selection is planning-only.
This selection does not perform the consolidation audit.
This selection does not change source.
This selection does not change tests.
This selection does not implement size-to-fit behavior.
This selection does not implement intrinsic/content size calculation.
This selection does not implement constraint solver behavior.
This selection does not implement constraint satisfaction.
This selection does not implement layout solving.
This selection does not implement real measuring.
This selection does not implement backend/event/runtime/capability authority.
This selection only authorizes the next audit package to be prepared under a separate gate.

## 9. Deferred Lanes
- Size-to-Fit Boundary
- Constraint Solver Boundary
- Layout Solving Boundary
- Backend Boundary
- Event Boundary
- Measuring Real Implementation

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
| layout metadata stack consolidation audit | planned only | ADMITTED FUTURE AUDIT | PASS |
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
- no consolidation audit performed in this PR
- no size-to-fit behavior
- no intrinsic/content size calculation as executable behavior
- no real text/glyph/image/widget measurement
- no font/backend/GPU measurement
- no WGPU/winit/Tauri measurement
- no constraint solver
- no constraint satisfaction algorithm
- no layout solving
- no layout engine rewrite
- no geometry mutation
- no layout mutation
- no sizing metadata mutation
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
PASS — POST-UI next lane selected after layout measuring seed audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-METADATA-STACK-CONSOLIDATION-AUDIT-PR.

This selection is planning-only and does not perform the consolidation audit, change source, change tests, implement size-to-fit behavior, intrinsic/content size calculation as executable behavior, constraint solver behavior, constraint satisfaction, layout solving, real measuring, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
