# POST-UI Roadmap Next Lane Selection After Layout Solving Implementation Source Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Solving Implementation Source line.

## 2. DNA Alignment
DNA inspected: YES
DNA source path: `docs/dna/SEMANTIC_UI_DNA.md`
docs/dna directory present: YES
docs/DNA.md present: NO
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout solving result metadata remains renderer-local;
- metadata remains deterministic;
- metadata remains source-reference-preserving;
- metadata remains non-mutating;
- selection remains docs-only;
- no full layout solving authority;
- no placing logic;
- no physical metrics extraction;
- no backend/event/runtime/capability authority;
- no Workbench/Studio integration.

## 3. Closed Basis
#1074 — roadmap selected layout solving implementation source
#1075 — layout solving implementation source
#1076 — layout solving implementation source fmt repair
#1077 — layout solving implementation source pr_body cleanup
#1078 — layout solving implementation source closeout
#1079 — layout solving implementation source ledger audit

## 4. Current Layout Solving Metadata Stack State
The current renderer layout metadata stack now reaches layout solving result metadata:

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

It does not implement full layout solving, placing logic, physical metrics extraction, pixel/screen/viewport placement, backend rendering, event dispatch, runtime/verifier/VM integration, capability admission, proof/debugger authority, WGPU/winit/Tauri integration, or Workbench/Studio integration.

metadata stack ready for consolidation audit: YES

## 5. Candidate Lanes

| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| `R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-METADATA-STACK-CONSOLIDATION-AUDIT-PR` | Selected | Layout metadata now reaches solving result metadata; before any stronger authority, consolidate and audit the stack. | Medium | Selected |
| `R12-UI-RENDERER-LAYOUT-PHYSICAL-PLACEMENT-BOUNDARY-LINE-FULL-PACKAGE` | Deferred / too early | Physical placement requires stronger authority and must wait. | High | Deferred |
| `R12-UI-RENDERER-LAYOUT-FINAL-RECTANGLE-PRODUCTION-LINE` | Deferred / too early | Final rectangles are not yet admitted. | High | Deferred |
| `R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE` | Deferred / too early | Backend/WGPU/winit/Tauri remains outside current layout metadata scope. | High | Deferred |
| `R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE` | Deferred / high-risk | Event dispatch is close to action/effect/capability semantics. | High | Deferred |
| `R12-UI-RENDERER-LAYOUT-MEASURING-REAL-IMPLEMENTATION-LINE` | Deferred | Real measuring remains outside current authority. | High | Deferred |

## 6. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer-local layout metadata boundaries.
3. Must not change source.
4. Must not change tests.
5. Must not perform the selected consolidation audit in this roadmap PR.
6. Must not implement full layout solving.
7. Must not introduce placing logic.
8. Must not introduce physical metrics extraction.
9. Must not introduce final physical layout.
10. Must not introduce backend/event/runtime/capability authority.
11. Must not introduce Workbench/Studio integration.
12. Must select an audit gate before higher-authority layout work.

## 7. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-METADATA-STACK-CONSOLIDATION-AUDIT-PR

This selection is planning-only.
This selection does not perform the consolidation audit.
This selection does not change source.
This selection does not change tests.
This selection does not add APIs.
This selection does not implement full layout solving.
This selection does not introduce placing logic.
This selection does not introduce physical metrics extraction.
This selection does not produce final physical layout.
This selection does not introduce backend/event/runtime/capability authority.

## 8. Deferred Lanes
- `R12-UI-RENDERER-LAYOUT-PHYSICAL-PLACEMENT-BOUNDARY-LINE-FULL-PACKAGE`
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
This selection does not implement full layout solving.
This selection does not introduce placing logic.
This selection does not introduce physical metrics extraction.
This selection does not introduce final physical layout.
This selection does not introduce backend/rendering/runtime/capability authority.
This selection does not introduce proof/debugger authority.
This selection does not introduce Workbench/Studio integration.

## 11. Non-Scope
- No source changes.
- No test changes.
- No consolidation audit performed in this PR.
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
Project #2 state: OBSERVED / PARTIAL API EVIDENCE
Project #2 item count: UNKNOWN
Project #2 duplicate count: NOT FULLY VERIFIED

## 13. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout solving implementation source audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-METADATA-STACK-CONSOLIDATION-AUDIT-PR.

This selection is planning-only and does not perform the consolidation audit, change source, change tests, implement full layout solving, introduce placing logic, introduce physical metrics extraction, produce final physical layout, introduce backend rendering, event dispatch, runtime/verifier/VM integration, capability admission, proof/debugger authority, WGPU/winit/Tauri integration, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
