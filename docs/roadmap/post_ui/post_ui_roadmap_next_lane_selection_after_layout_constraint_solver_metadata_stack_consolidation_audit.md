# POST-UI Roadmap Next Lane Selection After Layout Constraint Solver Metadata Stack Consolidation Audit

## 1. Purpose

This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Constraint Solver Metadata Stack Consolidation Audit.

## 2. DNA Alignment

DNA inspected: YES
DNA source paths inspected:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/DNA.md

docs/dna directory present: YES
docs/dna/SEMANTIC_UI_DNA.md present: YES
docs/DNA.md present: YES
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout metadata stack remains renderer-local;
- constraint solver metadata remains renderer-local;
- metadata layers remain deterministic;
- metadata layers remain source-reference-preserving;
- metadata layers remain non-mutating;
- constraint solver seed remains deterministic renderer-local solver metadata / intent substrate;
- next lane selection must remain docs-only;
- next lane selection must not introduce source behavior;
- next lane selection must not introduce real constraint satisfaction;
- next lane selection must not introduce equation solving;
- next lane selection must not introduce relation solving;
- next lane selection must not introduce iterative convergence;
- next lane selection must not introduce fixed-point solving;
- next lane selection must not introduce graph solving;
- next lane selection must not introduce layout solving;
- next lane selection must not introduce final rectangle production;
- next lane selection must not mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit/constraint-solver metadata;
- next lane selection must not introduce executable fit/fill/shrink/grow behavior;
- next lane selection must not introduce intrinsic/content size calculation;
- next lane selection must not introduce real measuring;
- next lane selection must not introduce draw/event/backend authority;
- next lane selection must not introduce runtime/verifier/VM/capability authority;
- next lane selection must not introduce proof/debugger authority;
- next lane selection must not introduce Workbench/Studio integration;
- this roadmap selection does not perform the layout solving boundary definition.

## 3. Closed Basis

#1041 — layout constraint solver boundary ledger audit
#1042 — roadmap selected constraint solver seed
#1043 — layout constraint solver seed source
#1044 — layout constraint solver seed closeout
#1045 — layout constraint solver seed ledger audit
#1046 — roadmap selected constraint solver metadata stack consolidation audit
#1047 — factual evidence wording correction
#1048 — constraint solver metadata stack consolidation audit

## 4. Constraint Solver Metadata Stack State

The current renderer layout metadata stack is fully consolidated through constraint solver metadata:

UiLayoutModel
  ↓
UiLayoutGeometryModel
  ↓
UiLayoutConstraintsModel
  ↓
UiLayoutSizingModel
  ↓
UiLayoutSizingAlgorithmModel
  ↓
UiLayoutMeasuringModel
  ↓
UiLayoutSizeToFitModel
  ↓
UiLayoutConstraintSolverModel

The stack is deterministic, renderer-local, source-reference-preserving, non-mutating, and metadata-only. With the metadata stack consolidated, the logical next step is defining the boundary for layout solving—the first step towards actual layout resolution.

## 5. Project #2 State

Project #2 state observed.
Project #2 metadata for the roadmap PR was recorded.
Project #2 duplicate/count verification remains manual-review pending where API evidence was incomplete.

## 6. Candidate Lanes

| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Solving Boundary | Selected | With the constraint solver metadata stack consolidated and audited, the structurally safest next step before introducing executable solver logic or final rectangle production is formally defining the Layout Solving Boundary. | Low | Proceed |
| Layout Solving Implementation | Deferred / too early | Layout solving introduces placement/refinement/finalization authority and must wait until its boundary is strictly defined. | High | Defer |
| Real Constraint Solver Implementation | Deferred / too early | Constraint Solver Seed is only metadata / intent substrate. Real executable solver behavior remains outside the current authority envelope. | High | Defer |
| Real Size-to-Fit Implementation | Deferred / too early | Size-to-fit seed remains metadata/intent substrate only. Executable fitting remains outside the current authority envelope. | High | Defer |
| Real Measuring Implementation | Deferred / forbidden for now | Measuring seed remains metadata/request substrate only. Real measurement remains outside the current authority envelope. | High | Defer |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside the current layout metadata and solver authority envelope. | High | Defer |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout authority is more mature. | High | Defer |

## 7. Selection Criteria

1. Must preserve DNA alignment.
2. Must preserve renderer-local layout metadata stack boundaries.
3. Must preserve constraint solver seed as metadata/intent substrate only.
4. Must build on the completed constraint solver metadata stack consolidation audit.
5. Must not define the layout solving boundary in this roadmap PR.
6. Must not introduce source changes.
7. Must not introduce test changes.
8. Must not introduce real constraint satisfaction.
9. Must not introduce equation solving.
10. Must not introduce relation solving.
11. Must not introduce iterative convergence.
12. Must not introduce fixed-point solving.
13. Must not introduce graph solving.
14. Must not introduce layout solving.
15. Must not introduce final rectangle production.
16. Must not introduce geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit/constraint-solver mutation.
17. Must not introduce executable fit/fill/shrink/grow behavior.
18. Must not introduce intrinsic/content size calculation.
19. Must not introduce real measuring.
20. Must not introduce draw/event/backend.
21. Must not introduce runtime/verifier/VM/capability authority.
22. Must select a boundary definition gate before implementation work.

## 8. Selected Next Lane

Selected next lane:
R12-UI-RENDERER-LAYOUT-SOLVING-BOUNDARY-LINE-FULL-PACKAGE

## 9. Deferred Lanes

- R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-LINE
- R12-UI-RENDERER-LAYOUT-CONSTRAINT-SOLVER-IMPLEMENTATION-LINE
- R12-UI-RENDERER-LAYOUT-SIZE-TO-FIT-IMPLEMENTATION-LINE
- R12-UI-RENDERER-LAYOUT-MEASURING-IMPLEMENTATION-LINE
- R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE
- R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE

## 10. Untracked Workspace Artifacts

Untracked workspace artifacts are treated as local-only, non-merged artifacts.
They must not be staged, committed, deleted, or merged by this roadmap selection PR.

## 11. Admission Guard

No source/behavior admitted in this PR. Only docs.

## 12. Non-Scope

This selection is planning-only.
This selection does not define the layout solving boundary.
This selection does not change source.
This selection does not change tests.
This selection does not implement real constraint satisfaction.
This selection does not implement equation solving.
This selection does not implement relation solving.
This selection does not implement iterative convergence.
This selection does not implement fixed-point solving.
This selection does not implement graph solving.
This selection does not implement layout solving.
This selection does not implement final rectangle production.
This selection does not mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit/constraint-solver metadata.
This selection does not introduce executable fit/fill/shrink/grow behavior.
This selection does not introduce intrinsic/content size calculation.
This selection does not introduce real measuring.
This selection does not introduce draw/event/backend/runtime/capability/proof/Workbench/Studio authority.
This selection only authorizes the next docs-only boundary package to be prepared under a separate gate.

## 13. Final Decision

Final decision:
PASS — POST-UI next lane selected after layout constraint solver metadata stack consolidation audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-SOLVING-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not define the layout solving boundary, change source, change tests, implement real constraint satisfaction, implement equation solving, implement relation solving, implement iterative convergence, implement fixed-point solving, implement graph solving, implement layout solving, implement final rectangle production, mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit/constraint-solver metadata, or introduce executable fit/fill/shrink/grow behavior, intrinsic/content size calculation, real measuring, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
