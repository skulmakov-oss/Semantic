# POST-UI Roadmap Next Lane Selection After Layout Metadata Module Split Source Audit

## 1. Purpose
This document records the roadmap selection for the next POST-UI lane after the completion of the R12 UI Renderer Layout Metadata Module Split Source line.

## 2. DNA Alignment
The selection aligns with Semantic UI DNA by maintaining explicit, deterministic, behavior-preserving boundaries for the renderer layout models without adopting unproven external backend or layout engines. The architecture defines a fully native Semantic-owned UI tree, ensuring all UI claims map explicitly to the UI source truth.

## 3. Closed Basis
*   #1063 — renderer layout metadata module split boundary ledger audit
*   #1064 — roadmap selected layout metadata module split source lane
*   #1065 — source split renderer layout metadata into modules
*   #1066 — renderer layout metadata module split source closeout
*   #1067 — source ledger audit: audit renderer layout metadata module split source

## 4. Module Split Source State
The `layout.rs` file has been cleanly split into `layout/mod.rs` and the corresponding ownership modules (`base.rs`, `geometry.rs`, `constraints.rs`, `sizing.rs`, `sizing_algorithm.rs`, `measuring.rs`, `size_to_fit.rs`, `constraint_solver.rs`, `solving.rs`). The split has been verified as behavior-preserving and is currently merged and ledger-audited.

## 5. Current Post-Split Metadata Stack
The verified post-split stack comprises:
```text
layout/mod.rs
  ↓ façade / re-export

layout/base.rs
  ↓ UiLayoutModel

layout/geometry.rs
  ↓ UiLayoutGeometryModel

layout/constraints.rs
  ↓ UiLayoutConstraintsModel

layout/sizing.rs
  ↓ UiLayoutSizingModel

layout/sizing_algorithm.rs
  ↓ UiLayoutSizingAlgorithmModel

layout/measuring.rs
  ↓ UiLayoutMeasuringModel

layout/size_to_fit.rs
  ↓ UiLayoutSizeToFitModel

layout/constraint_solver.rs
  ↓ UiLayoutConstraintSolverModel

layout/solving.rs
  ↓ UiLayoutSolvingModel
```

## 6. Candidate Lanes

| Candidate                                                             | Classification       | Reason                                                                                                                    |
| --------------------------------------------------------------------- | -------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `R12-UI-RENDERER-LAYOUT-METADATA-MODULE-SPLIT-CONSOLIDATION-AUDIT-PR` | Selected             | Source split is merged and ledger-audited. Before any real layout behavior, consolidate the new module tree as canonical. |
| `R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-BOUNDARY-PR`           | Deferred             | Possible next structural behavior boundary, but should wait until post-split tree is consolidated.                        |
| `R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-LINE`                  | Deferred / too early | Real placement/final-rectangle behavior remains too early without explicit boundary.                                      |
| `R12-UI-RENDERER-LAYOUT-CONSTRAINT-SOLVER-IMPLEMENTATION-LINE`        | Deferred / too early | Constraint solver remains metadata-only.                                                                                  |
| `R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE`                  | Deferred / too early | Backend authority should wait until renderer-local layout structure is canonical.                                         |
| `R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE`                             | Deferred / high-risk | Event/effect/capability surfaces remain higher-risk.                                                                      |

## 7. Selection Criteria
The selected lane must sequentially continue the post-UI consolidation strategy. Because the source split is complete but not yet consolidated, the next logical step must be an audit proving that the split structure is the new canonical baseline.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-METADATA-MODULE-SPLIT-CONSOLIDATION-AUDIT-PR

This selection is planning-only.
This selection does not perform the post-split consolidation audit.
This selection does not change source.
This selection does not change tests.
This selection does not refactor modules.
This selection does not move files.
This selection does not change public APIs.
This selection does not change behavior.
This selection does not implement real layout solving.
This selection does not introduce backend/runtime/capability authority.

## 9. Deferred Lanes
*   `R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-BOUNDARY-PR`
*   `R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-LINE`
*   `R12-UI-RENDERER-LAYOUT-CONSTRAINT-SOLVER-IMPLEMENTATION-LINE`
*   `R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE`
*   `R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE`

## 10. Project #2 State
Project #2 state: OBSERVED / PARTIAL API EVIDENCE

## 11. Untracked Workspace Artifacts
Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.

## 12. Admission Guard
No real layout solving, event routing, runtime logic, or capability execution may be introduced. This lane ensures structural stability prior to any behavioral additions.

## 13. Non-Scope
*   no source changes
*   no test changes
*   no post-split consolidation audit performed in this PR
*   no additional module split work
*   no file moves
*   no refactor
*   no public API changes
*   no behavior changes
*   no real layout solving
*   no placement algorithm
*   no final rectangle production
*   no computed rectangle production
*   no metadata mutation
*   no draw/event/backend/runtime/capability authority
*   no Workbench/Studio integration

## 14. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout metadata module split source audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-METADATA-MODULE-SPLIT-CONSOLIDATION-AUDIT-PR.

This selection is planning-only and does not perform the post-split consolidation audit, change source, change tests, refactor modules, move files, change public APIs, change behavior, implement real layout solving, implement placement algorithm, produce final rectangles, produce computed rectangles, mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit/constraint-solver/layout-solving metadata, introduce backend systems, introduce runtime/verifier/VM integration, introduce capability admission, introduce proof/debugger authority, or introduce Workbench/Studio integration.
