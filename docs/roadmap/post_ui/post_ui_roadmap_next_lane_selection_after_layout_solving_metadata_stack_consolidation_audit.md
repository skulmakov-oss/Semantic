# POST-UI Roadmap Next Lane Selection After Layout Solving Metadata Stack Consolidation Audit

## 1. Purpose
Select the next POST-UI roadmap lane after the completion of the R12 UI Renderer Layout Solving Metadata Stack Consolidation Audit.

## 2. DNA Alignment
*   DNA directory present: YES
*   docs/dna/SEMANTIC_UI_DNA.md present: YES
*   docs/DNA.md present: NO
*   DNA conflicts detected: NO

## 3. Closed Basis
*   #1053 — roadmap selected layout solving seed
*   #1054 — layout solving seed source
*   #1055 — layout solving seed closeout
*   #1056 — layout solving seed closeout evidence correction
*   #1057 — layout solving seed ledger audit
*   #1058 — roadmap selected layout solving metadata stack consolidation audit
*   #1059 — layout solving metadata stack consolidation audit

## 4. Current Metadata Stack
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
UiLayoutSolvingModel
```

## 5. Module Split Rationale
The layout metadata stack is now deep and heavily consolidated within `crates/prom-ui/src/layout.rs`. The file has grown as each layer was added. Before adding real implementation logic, defining a boundary for future module split/refactor is the strategic next step to maintain code organization.

## 6. Candidate Lanes
| Candidate                                                      | Classification       | Reason                                                                                                                                                                   |
| -------------------------------------------------------------- | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `R12-UI-RENDERER-LAYOUT-METADATA-MODULE-SPLIT-BOUNDARY-PR`     | Selected             | The layout metadata stack is now consolidated through `UiLayoutSolvingModel`; before real layout behavior, define a docs-only boundary for future module split/refactor. |
| `R12-UI-RENDERER-LAYOUT-METADATA-MODULE-SPLIT-SOURCE-PR`       | Deferred / too early | Actual file split/refactor needs boundary first.                                                                                                                         |
| `R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-LINE`           | Deferred / too early | Real placement/final-rectangle solving remains unauthorized.                                                                                                             |
| `R12-UI-RENDERER-LAYOUT-CONSTRAINT-SOLVER-IMPLEMENTATION-LINE` | Deferred / too early | Constraint solver remains metadata-only.                                                                                                                                 |
| `R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE`           | Deferred / too early | Backend authority should wait until renderer-local layout layers are structurally stable.                                                                                |
| `R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE`                      | Deferred / high-risk | Event/effect/capability surfaces remain higher-risk.                                                                                                                     |

## 7. Selection Criteria
A docs-only boundary must be established to document intent and scope before executing a potentially disruptive structural refactor or file split on `layout.rs`.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-METADATA-MODULE-SPLIT-BOUNDARY-PR

This selection is planning-only.
This selection does not perform the module split boundary work.
This selection does not split layout.rs.
This selection does not change source.
This selection does not change tests.
This selection does not refactor modules.
This selection does not move files.
This selection does not change public APIs.
This selection does not implement real layout solving.
This selection does not introduce backend/runtime/capability authority.

## 9. Deferred Lanes
*   R12-UI-RENDERER-LAYOUT-METADATA-MODULE-SPLIT-SOURCE-PR
*   R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-LINE
*   R12-UI-RENDERER-LAYOUT-CONSTRAINT-SOLVER-IMPLEMENTATION-LINE
*   R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE
*   R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE

## 10. Project #2 State
Project #2 state: OBSERVED / PARTIAL API EVIDENCE
Project #2 item count: UNKNOWN
Project #2 duplicate count: NOT FULLY VERIFIED

## 11. Untracked Workspace Artifacts
Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.

## 12. Admission Guard
Admission to the next lane is restricted solely to the Layout Metadata Module Split Boundary PR. No other logic is authorized.

## 13. Non-Scope
* no source changes
* no test changes
* no module split performed in this PR
* no layout.rs split
* no refactor
* no file moves
* no public API changes
* no real layout solving
* no placement algorithm
* no final rectangle production
* no computed rectangle production
* no metadata mutation
* no draw/event/backend/runtime/capability authority
* no Workbench/Studio integration
* no dependency additions
* local untracked artifacts are not deleted

## 14. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout solving metadata stack consolidation audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-METADATA-MODULE-SPLIT-BOUNDARY-PR.

This selection is planning-only and does not perform the module split boundary work, split layout.rs, change source, change tests, refactor modules, move files, change public APIs, implement real layout solving, implement placement algorithm, produce final rectangles, produce computed rectangles, mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit/constraint-solver/layout-solving metadata, introduce backend systems, introduce runtime/verifier/VM integration, introduce capability admission, introduce proof/debugger authority, or introduce Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
