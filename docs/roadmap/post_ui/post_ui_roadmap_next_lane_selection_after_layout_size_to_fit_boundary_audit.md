# POST-UI Roadmap Next Lane Selection After Layout Size-to-Fit Boundary Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Size-to-Fit Boundary line.

## 2. DNA Alignment
DNA inspected: YES
DNA source path: docs/dna/SEMANTIC_UI_DNA.md
docs/dna directory present: YES
docs/DNA.md present: NO
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout metadata stack remains renderer-local;
- geometry seed remains inert renderer-local metadata;
- constraints seed remains inert renderer-local metadata declarations;
- sizing seed remains inert renderer-local metadata/result declarations;
- sizing algorithm seed remains deterministic renderer-local metadata derivation substrate;
- measuring seed remains deterministic renderer-local measurement metadata/request substrate;
- size-to-fit boundary remains docs-only and audited;
- size-to-fit seed may only introduce deterministic renderer-local fit metadata / intent substrate;
- size-to-fit seed must not introduce executable fit/fill/shrink/grow behavior;
- size-to-fit seed must not introduce intrinsic/content size calculation as executable behavior;
- size-to-fit seed must not introduce real measuring;
- size-to-fit seed must not introduce font/backend/GPU/WGPU/winit/Tauri authority;
- size-to-fit seed must not introduce constraint solver authority;
- size-to-fit seed must not introduce constraint satisfaction authority;
- size-to-fit seed must not introduce layout solving;
- size-to-fit seed must not introduce geometry/layout/sizing/constraints/measuring mutation;
- size-to-fit seed must not introduce draw/event/backend authority;
- size-to-fit seed must not introduce runtime/verifier/VM/capability authority;
- size-to-fit seed must not introduce proof/debugger authority;
- size-to-fit seed must not introduce Workbench/Studio integration;
- this roadmap selection does not implement the size-to-fit seed.

## 3. Closed Basis
#1028 — roadmap selected size-to-fit boundary
#1029 — layout size-to-fit boundary
#1030 — layout size-to-fit boundary closeout
#1031 — layout size-to-fit boundary ledger audit

## 4. Size-to-Fit Boundary State
The size-to-fit boundary is closed as docs-only boundary work. It documents future size-to-fit authority as a separately gated deterministic renderer-local metadata interpretation layer without implementing size-to-fit source, fit/fill/shrink/grow behavior, intrinsic/content size calculation as executable behavior, real text/glyph/image/widget measurement, font/backend/GPU measurement, WGPU/winit/Tauri measurement, constraint solver behavior, constraint satisfaction, layout solving, layout engine rewrite, geometry/layout/sizing/constraints/measuring mutation, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 5. Project #2 State
All tracked Project #2 items for the closed basis are correctly assigned to the POST-UI track, Wave R12, and are Done. No metadata drift or duplication detected.

## 6. Candidate Lanes

| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Size-to-Fit Seed | Selected | The size-to-fit boundary is selected, documented, closed out, and ledger-audited. The next structurally valid step is a minimal source seed for deterministic renderer-local fit metadata / intent substrate. | Low | Selected |
| Real Size-to-Fit Implementation | Deferred / too early | Too early. Size-to-fit seed must remain metadata / intent substrate only before any executable fit behavior. | High | Deferred |
| Constraint Solver Boundary | Deferred | Constraint solver authority is higher-risk and remains explicitly separated from size-to-fit seed. | High | Deferred |
| Layout Solving Boundary | Deferred / too early | Layout solving introduces placement/refinement authority and must wait until size-to-fit seed and solver boundaries are separately handled. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside the current layout authority envelope. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout authority is more mature. | High | Deferred |
| Real Measuring Implementation | Deferred / forbidden for now | Measuring seed remains metadata/request substrate only. Real measurement remains explicitly outside the current authority envelope. | High | Deferred |

## 7. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer-local layout metadata stack boundaries.
3. Must preserve size-to-fit boundary as docs-only and audited.
4. Must not implement size-to-fit seed in the roadmap selection PR.
5. Must not introduce source changes.
6. Must not introduce test changes.
7. Must not introduce executable fit/fill/shrink/grow behavior.
8. Must not introduce intrinsic/content size calculation as executable behavior.
9. Must not introduce real measuring.
10. Must not introduce font/backend/GPU/WGPU/winit/Tauri authority.
11. Must not introduce constraint solver behavior.
12. Must not introduce constraint satisfaction behavior.
13. Must not introduce layout solving.
14. Must not introduce geometry/layout/sizing/constraints/measuring mutation.
15. Must not introduce draw/event/backend.
16. Must not introduce runtime/verifier/VM/capability authority.
17. Must build naturally on the closed size-to-fit boundary and audit.
18. Must be source-gated separately before implementation.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-SIZE-TO-FIT-SEED-LINE-FULL-PACKAGE

## 9. Deferred Lanes
- Real Size-to-Fit Implementation
- Constraint Solver Boundary
- Layout Solving Boundary
- Backend Boundary
- Event Boundary
- Real Measuring Implementation

## 10. Untracked Workspace Artifacts
Untracked workspace artifacts are treated as local-only, non-merged artifacts.
They must not be staged, committed, deleted, or merged by this roadmap selection PR.
Known artifacts:
.claude/
examples/baseline/
scratch/

## 11. Admission Guard
This selection is planning-only.
This selection does not implement size-to-fit seed.
This selection does not implement size-to-fit source.
This selection does not implement fit/fill/shrink/grow behavior.
This selection does not implement intrinsic/content size calculation as executable behavior.
This selection does not implement real measuring.
This selection does not implement font/backend/GPU measurement.
This selection does not implement WGPU/winit/Tauri measurement.
This selection does not implement constraint solver behavior.
This selection does not implement constraint satisfaction.
This selection does not implement layout solving.
This selection does not mutate geometry/layout/sizing/constraints/measuring metadata.
This selection does not introduce draw/event/backend/runtime/capability/proof/Workbench/Studio authority.
This selection only authorizes the next source package to be prepared under a separate gate.

## 12. Non-Scope
- No source changes.
- No test changes.
- No layout logic changes.
- No dependency changes.

## 13. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout size-to-fit boundary audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-SIZE-TO-FIT-SEED-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement size-to-fit seed, change source, change tests, implement executable fit/fill/shrink/grow behavior, implement intrinsic/content size calculation as executable behavior, implement real measuring, implement font/backend/GPU measurement, implement WGPU/winit/Tauri measurement, implement constraint solver behavior, implement constraint satisfaction, implement layout solving, mutate geometry/layout/sizing/constraints/measuring metadata, or introduce draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
