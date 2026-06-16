# POST-UI Roadmap Next Lane Selection After Layout Solving Metadata Stack Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Solving Implementation Metadata Stack Consolidation Audit.

## 2. DNA Alignment
DNA inspected: YES
DNA source path: `docs/dna/SEMANTIC_UI_DNA.md`
docs/dna directory present: YES
docs/DNA.md present: NO
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout metadata remains renderer-local;
- layout solving result metadata remains renderer-local;
- selection remains docs-only;
- physical placement boundary is not yet defined;
- physical placement is not implemented;
- final physical layout is not produced;
- pixel/screen/viewport placement is not introduced;
- backend/event/runtime/capability authority is not introduced;
- Workbench/Studio remains out of scope.

## 3. Closed Basis
#1074 — roadmap selected layout solving implementation source
#1075 — layout solving implementation source
#1076 — layout solving implementation source fmt repair
#1077 — layout solving implementation source pr_body cleanup
#1078 — layout solving implementation source closeout
#1079 — layout solving implementation source ledger audit
#1080 — roadmap selected layout solving implementation metadata stack consolidation audit
#1081 — layout solving implementation metadata stack consolidation audit

## 4. Current Layout Metadata Stack State
The current renderer layout metadata stack is consolidated through layout solving result metadata:

```text
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
  ↓
UiLayoutSolvingResultModel
```

The stack remains renderer-local, deterministic, source-reference-preserving, non-mutating, and metadata/result-oriented.

It does not implement physical placement, final physical layout, pixel/screen/viewport placement, backend rendering, event dispatch, runtime/verifier/VM integration, capability admission, proof/debugger authority, WGPU/winit/Tauri integration, or Workbench/Studio integration.

## 5. Candidate Lanes

| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| `R12-UI-RENDERER-LAYOUT-PHYSICAL-PLACEMENT-BOUNDARY-LINE-FULL-PACKAGE` | Selected | Metadata/result stack is consolidated; next safe step is to define the boundary for physical placement before admitting any placement source. | Medium | Selected |
| `R12-UI-RENDERER-LAYOUT-PHYSICAL-PLACEMENT-SOURCE-LINE` | Deferred / too early | Physical placement boundary is not defined yet. | High | Deferred |
| `R12-UI-RENDERER-LAYOUT-FINAL-RECTANGLE-PRODUCTION-LINE` | Deferred / too early | Final physical rectangles require placement boundary first. | High | Deferred |
| `R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE` | Deferred / too early | Backend/WGPU/winit/Tauri remains outside current layout metadata scope. | High | Deferred |
| `R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE` | Deferred / high-risk | Event dispatch is close to action/effect/capability semantics. | High | Deferred |
| `R12-UI-RENDERER-LAYOUT-MEASURING-REAL-IMPLEMENTATION-LINE` | Deferred | Real measuring remains outside current authority. | High | Deferred |

## 6. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer-local layout metadata boundaries.
3. Must not change source.
4. Must not change tests.
5. Must not perform the selected physical placement boundary package in this roadmap PR.
6. Must not implement physical placement.
7. Must not produce final physical layout.
8. Must not introduce pixel/screen/viewport placement.
9. Must not introduce backend/event/runtime/capability authority.
10. Must not introduce Workbench/Studio integration.
11. Must select a boundary gate before any physical placement source work.

## 7. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-PHYSICAL-PLACEMENT-BOUNDARY-LINE-FULL-PACKAGE

This selection is planning-only.
This selection does not define the physical placement boundary.
This selection does not perform the physical placement boundary package.
This selection does not change source.
This selection does not change tests.
This selection does not add APIs.
This selection does not implement physical placement.
This selection does not produce final physical layout.
This selection does not introduce pixel/screen/viewport placement.
This selection does not introduce backend/event/runtime/capability authority.

## 8. Deferred Lanes
- `R12-UI-RENDERER-LAYOUT-PHYSICAL-PLACEMENT-SOURCE-LINE`
- `R12-UI-RENDERER-LAYOUT-FINAL-RECTANGLE-PRODUCTION-LINE`
- `R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE`
- `R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE`
- `R12-UI-RENDERER-LAYOUT-MEASURING-REAL-IMPLEMENTATION-LINE`

## 9. Untracked Workspace Artifacts
Untracked workspace artifacts are treated as local-only, non-merged artifacts.
They must not be staged, committed, deleted, or merged by this roadmap selection PR.

Known artifacts:

- `.claude/`
- `examples/baseline/`
- `scratch/`

## 10. Admission Guard
This selection is planning-only.
This selection does not implement the consolidation audit.
This selection does not implement physical placement.
This selection does not implement full layout solving.
This selection does not introduce physical metrics extraction.
This selection does not introduce final physical layout.
This selection does not introduce backend/rendering/runtime/capability authority.
This selection does not introduce proof/debugger authority.
This selection does not introduce Workbench/Studio integration.

## 11. Non-Scope
- No source changes.
- No test changes.
- No physical placement boundary package performed in this PR.
- No layout solving implementation changes.
- No layout solving result metadata changes.
- No placement algorithm.
- No final rectangle production.
- No computed rectangle production.
- No metadata mutation.
- No draw/event/backend/runtime/capability authority.
- No Workbench/Studio integration.
- No dependency additions.

## 12. Project #2 State
Project #2 state: OBSERVED / VERIFIED
Project #2 item count for #1081: 1
Project #2 duplicate count for #1081: 0
Project #2 metadata for #1081:
- Status: Done
- Track: POST-UI
- Wave: R12
- Type: Audit
- Risk: Medium
- Boundary: Renderer
- Gate: FullPreflight
- Evidence: Roadmap doc
- Depends on: #1080

## 13. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout solving metadata stack audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-PHYSICAL-PLACEMENT-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not define the physical placement boundary, perform the physical placement boundary package, change source, change tests, implement physical placement, produce final physical layout, introduce pixel/screen/viewport placement, introduce backend rendering, event dispatch, runtime/verifier/VM integration, capability admission, proof/debugger authority, WGPU/winit/Tauri integration, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
