# POST-UI Roadmap Next Lane Selection After Layout Solving Implementation Boundary Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completion and ledger audit of the R12 UI Renderer Layout Solving Implementation Boundary line.

## 2. DNA Alignment
This selection aligns with Semantic UI DNA by maintaining the boundary between renderer-local layout solving and foreign backend/runtime/capability authority. The selected next lane targets renderer-local derived metadata implementation only, without adopting foreign runtime behaviors.

## 3. Closed Basis
- #1070 — roadmap selected layout solving implementation boundary
- #1071 — renderer layout solving implementation boundary document
- #1072 — renderer layout solving implementation boundary closeout
- #1073 — renderer layout solving implementation boundary ledger audit

## 4. Current Canonical Layout Module Tree
The canonical layout module tree has been verified:
- `layout.rs` is absent.
- `layout/mod.rs` is present as the public façade and re-export root.
- The layout ownership modules (`base`, `geometry`, `constraints`, `sizing`, `sizing_algorithm`, `measuring`, `size_to_fit`, `constraint_solver`, `solving`) are present and consolidated.

## 5. Current Metadata Stack
The current layout metadata stack is verified and includes:
- `UiLayoutModel`
- `UiLayoutGeometryModel`
- `UiLayoutConstraintsModel`
- `UiLayoutSizingModel`
- `UiLayoutSizingAlgorithmModel`
- `UiLayoutMeasuringModel`
- `UiLayoutSizeToFitModel`
- `UiLayoutConstraintSolverModel`
- `UiLayoutSolvingModel`
- `UiLayoutSolvingEntry`
Real layout solving behavior is absent; the structure is valid for the implementation source lane.

## 6. Boundary Ledger State
The boundary ledger confirms that the future implementation scope, forbidden scope, authority boundaries, constraint solver separation, backend/runtime/capability separation, determinism requirements, and future test surface requirements are fully defined and active. Actual implementation remains separately gated.

## 7. Candidate Lanes
| Candidate | Classification | Reason |
|---|---|---|
| `R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-SOURCE-PR` | Selected | Boundary, closeout, and ledger audit are complete. Next valid step is a narrow source PR for deterministic renderer-local derived solving result metadata. |
| `R12-UI-RENDERER-LAYOUT-SOLVING-FINAL-RECTANGLES-SOURCE-PR` | Deferred / too broad | Final rectangle authority must be introduced only if explicitly included and tested in a future source gate. |
| `R12-UI-RENDERER-LAYOUT-SOLVING-PLACEMENT-ALGORITHM-PR` | Deferred / too broad | Placement algorithm must not be introduced without a narrow implementation plan and golden tests. |
| `R12-UI-RENDERER-LAYOUT-CONSTRAINT-SOLVER-IMPLEMENTATION-BOUNDARY-PR` | Alternative / later | Constraint solver behavior remains separately gated. |
| `R12-UI-RENDERER-BACKEND-BOUNDARY-LINE-FULL-PACKAGE` | Deferred / too early | Backend authority should wait until renderer-local layout solving source is stable. |
| `R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE` | Deferred / high-risk | Event/effect/capability surfaces remain higher-risk. |

## 8. Source Lane Guardrails
The future source PR must be narrow.

Allowed future source scope:
- introduce derived renderer-local layout solving result metadata;
- preserve existing metadata inputs;
- preserve deterministic IDs;
- preserve source references;
- preserve public API through existing façade;
- add tests for deterministic order/count;
- add tests for source-reference preservation;
- add tests proving no input mutation;
- add tests proving no backend/runtime/capability authority.

Not allowed in the future source PR unless separately gated:
- backend draw commands;
- event dispatch;
- runtime/verifier/VM actions;
- capability admission;
- Workbench/Studio integration;
- real constraint solver behavior;
- broad layout engine rewrite;
- dependency additions.

Final rectangle metadata may only be introduced if the future source PR explicitly defines it as derived renderer-local result metadata and proves determinism with tests. It must not produce draw commands or backend actions.

## 9. Selection Criteria
The selection requires a completed, closed, and audited boundary, which has been achieved in the `#1070`–`#1073` lineage. The next step must advance implementation safely under those defined boundaries.

## 10. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-SOURCE-PR

This selection is planning-only.
This selection does not implement layout solving.
This selection does not implement placement.
This selection does not produce final rectangles.
This selection does not produce computed rectangles.
This selection does not change source.
This selection does not change tests.
This selection does not mutate metadata.
This selection does not introduce backend/runtime/capability authority.

## 11. Deferred Lanes
Final rectangles, placement algorithms, constraint solver logic, backend integration, and event/capability surfaces are explicitly deferred and must not be included in the selected next lane without a separate, explicit boundary gate.

## 12. Project #2 State
OBSERVED / PARTIAL API EVIDENCE.

## 13. Untracked Workspace Artifacts
Pre-existing untracked local workspace artifacts (`.claude/`, `examples/baseline/`, `scratch/`) remain present but were not staged, not committed, not deleted, and not merged.

## 14. Admission Guard
The admission guard verifies that this selection defines future authority only and introduces no source, test, or behavior changes.

## 15. Non-Scope
This PR does not implement layout solving, modify source or tests, or alter existing DNA or metadata structure.

## 16. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout solving implementation boundary audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-SOLVING-IMPLEMENTATION-SOURCE-PR.

This selection is planning-only and does not change source, change tests, implement real layout solving, implement placement algorithm, produce final rectangles, produce computed rectangles, mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit/constraint-solver/layout-solving metadata, introduce real constraint satisfaction, introduce real solver execution, introduce backend systems, introduce runtime/verifier/VM integration, introduce capability admission, introduce proof/debugger authority, or introduce Workbench/Studio integration.

The future source PR is constrained to a narrow deterministic renderer-local implementation surface and remains separately gated.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
