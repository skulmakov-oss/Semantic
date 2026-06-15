# R12 UI Renderer Layout Metadata Module Split Boundary

## 1. Purpose
This document defines the docs-only boundary for a future renderer layout metadata module split after the layout metadata stack was consolidated through UiLayoutSolvingModel.

## 2. DNA Alignment
*   DNA directory present: YES
*   docs/dna/SEMANTIC_UI_DNA.md present: YES
*   docs/DNA.md present: NO
*   DNA conflicts detected: NO

## 3. Closed Basis
*   #1059 — layout solving metadata stack consolidation audit
*   #1060 — roadmap selected layout metadata module split boundary

## 4. Current Layout Metadata Stack
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
The current stack remains deterministic, renderer-local, source-reference-preserving, non-mutating, and metadata-only.

## 5. Module Split Rationale
The split is considered because layout.rs now contains a long deterministic metadata stack. Future module separation should improve ownership clarity, review surface, and regression isolation without changing behavior.

## 6. Future Module Ownership Map
| Future module | Intended ownership | Must not own |
|---|---|---|
| layout.rs | façade/re-export/compatibility layer | behavior changes |
| layout/base.rs | base layout model and nodes | geometry/solver/backend authority |
| layout/geometry.rs | geometry metadata | placement/final rect production |
| layout/constraints.rs | constraint declarations metadata | constraint satisfaction |
| layout/sizing.rs | sizing metadata | real sizing algorithm behavior |
| layout/sizing_algorithm.rs | deterministic sizing algorithm metadata | real measuring or solving |
| layout/measuring.rs | inert measuring metadata | font/backend/GPU measurement |
| layout/size_to_fit.rs | inert size-to-fit metadata | executable fit/fill/shrink/grow |
| layout/constraint_solver.rs | inert constraint solver metadata | real constraint solving |
| layout/solving.rs | inert layout solving metadata | placement/final rectangle production |

## 7. Allowed Future Split Scope
Allowed in a future separately gated source PR:
- move existing metadata types into owned modules;
- preserve public API through re-exports if needed;
- preserve deterministic IDs;
- preserve source references;
- preserve existing test behavior;
- preserve non-mutation guarantees;
- add module-local tests only if required for moved surfaces;
- keep behavior identical.

## 8. Forbidden Scope
Forbidden in this boundary PR and still forbidden until separately gated:
- actual module split;
- file moves;
- source refactor;
- public API changes;
- real layout solving;
- placement algorithm;
- final rectangle production;
- computed rectangle production;
- geometry/layout/sizing/constraints/measuring/size-to-fit/constraint-solver/layout-solving mutation;
- real constraint satisfaction;
- real solver execution;
- executable fit/fill/shrink/grow behavior;
- intrinsic/content size calculation;
- real measuring;
- draw/event/backend systems;
- runtime/verifier/VM integration;
- capability admission;
- proof/debugger authority;
- Workbench/Studio integration.

## 9. Public API / Compatibility Boundary
A future split must not break existing public callers unless a separate API change gate is selected.
The preferred future split strategy is façade preservation:
- keep public names stable;
- keep build_* entrypoints stable;
- re-export moved types/functions from the existing public surface where needed;
- prove compatibility with existing tests.

## 10. Test Surface Boundary
A future split must preserve existing tests:
- renderer_layout_seed.rs
- renderer_layout_geometry_seed.rs
- renderer_layout_constraints_seed.rs
- renderer_layout_sizing_seed.rs
- renderer_layout_sizing_algorithm_seed.rs
- renderer_layout_measuring_seed.rs
- renderer_layout_size_to_fit_seed.rs
- renderer_layout_constraint_solver_seed.rs
- renderer_layout_solving_seed.rs

## 11. Migration Constraints
The migration to split modules must be a pure structural refactor. No functionality or behavioral changes are permitted during the split operation.

## 12. Project #2 State
Project #2 state: OBSERVED / PARTIAL API EVIDENCE
Project #2 item count: UNKNOWN
Project #2 duplicate count: NOT FULLY VERIFIED

## 13. Untracked Workspace Artifacts
Tracked repository state remains clean for this boundary definition. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.

## 14. Admission Guard
Admission to split source files is blocked until this boundary PR is fully closed out and audited.

## 15. Final Decision
Final decision:
PASS — R12 UI Renderer Layout Metadata Module Split Boundary is defined as a docs-only boundary.

This boundary authorizes only a future separately gated module split/refactor package. It does not split layout.rs, move files, change source, change tests, change public APIs, implement real layout solving, implement placement algorithm, produce final rectangles, produce computed rectangles, mutate metadata, introduce real constraint satisfaction, introduce real solver execution, introduce executable fit/fill/shrink/grow behavior, introduce intrinsic/content size calculation, introduce real measuring, introduce draw/event/backend systems, introduce runtime/verifier/VM integration, introduce capability admission, introduce proof/debugger authority, or introduce Workbench/Studio integration.
