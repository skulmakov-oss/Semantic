# POST-UI Roadmap Next Lane Selection After Layout Sizing Algorithm Seed Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Sizing Algorithm Seed line.

## 2. DNA Alignment
DNA inspected: YES
DNA source path: docs/dna/SEMANTIC_UI_DNA.md; docs/DNA.md present as repository fallback
docs/dna directory present: YES
docs/DNA.md present: YES
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- geometry seed remains inert renderer-local metadata;
- constraints seed remains inert renderer-local metadata declarations;
- sizing seed remains inert renderer-local metadata/result declarations;
- sizing algorithm seed remains deterministic renderer-local metadata derivation substrate;
- measuring boundary must remain docs-only;
- measuring boundary must not implement text/glyph/image/widget measurement;
- measuring boundary must not introduce backend/font/GPU measurement authority;
- measuring boundary must not introduce size-to-fit authority;
- measuring boundary must not introduce constraint solver authority;
- measuring boundary must not introduce constraint satisfaction authority;
- measuring boundary must not introduce layout solving;
- measuring boundary must not introduce draw/event/backend authority;
- measuring boundary must not introduce runtime/verifier/VM/capability authority;
- measuring boundary must not introduce proof/debugger authority;
- measuring boundary must not introduce Workbench/Studio integration;
- this roadmap selection does not implement measuring source.

## 3. Closed Basis
- #1013 — roadmap selected sizing algorithm seed
- #1014 — layout sizing algorithm seed source
- #1015 — layout sizing algorithm seed closeout
- #1016 — layout sizing algorithm seed ledger audit

## 4. Sizing Algorithm Seed State
Sizing algorithm seed is closed as a minimal deterministic renderer-local sizing metadata derivation substrate. It implements deterministic sizing algorithm metadata only and does not implement measuring algorithm behavior, size-to-fit behavior, intrinsic/content size calculation, glyph/text/image/widget measurement, constraint solver behavior, constraint satisfaction, layout solving, layout engine rewrite, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 5. Project #2 State
Project #2 item states are clean for the corrected sizing seed line:

- #1013: Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | #1012
- #1014: Done | POST-UI | R12 | Code | High | Renderer | PRReady | PR | #1013
- #1015: Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | #1014
- #1016: Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | #1015

## 6. Candidate Lanes
| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Measuring Boundary | Selected | Sizing algorithm seed is complete and audited; measuring remains explicitly bounded before any measuring source exists. | Medium | Selected |
| Measuring Seed / Source | Deferred / too early | Too early. A measuring boundary must exist before any measuring source package. | High | Deferred |
| Size-to-Fit Boundary | Deferred | Fit/fill/shrink/grow behavior is higher-authority and should remain separated from measuring. | Medium | Deferred |
| Constraint Solver Boundary | Deferred / too early | Constraint solver boundary remains premature until measuring and fit boundaries are handled or explicitly deferred. | High | Deferred |
| Layout Solving Boundary | Deferred / too early | Layout solving is higher-authority and should wait until measuring, fit, and solver boundaries are separately handled. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout authority is more mature. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside current layout metadata and measuring boundary work. | High | Deferred |
| Layout Metadata / Algorithm / Measuring Consolidation Audit | Deferred | Useful later, but the immediate architectural pressure point is defining measuring authority before source. | Medium | Deferred |

## 7. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer/layout/geometry/constraints/sizing seed inertness.
3. Must preserve sizing algorithm seed as metadata derivation substrate only.
4. Must not implement measuring source in the roadmap selection PR.
5. Must not introduce text/glyph/image/widget measurement.
6. Must not introduce font/backend/GPU measurement.
7. Must not introduce size-to-fit behavior.
8. Must not introduce intrinsic/content size calculation as executable behavior.
9. Must not introduce constraint solver behavior.
10. Must not introduce constraint satisfaction behavior.
11. Must not introduce layout solving.
12. Must not introduce draw/event/backend.
13. Must not introduce runtime/verifier/VM/capability authority.
14. Must build naturally on closed sizing algorithm seed and audit.
15. Must be boundary-gated before any measuring source exists.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-MEASURING-BOUNDARY-LINE-FULL-PACKAGE

This selection is planning-only.
This selection does not implement measuring source.
This selection does not implement text/glyph/image/widget measurement.
This selection does not implement font/backend/GPU measurement.
This selection does not implement size-to-fit behavior.
This selection does not implement intrinsic/content size calculation as executable behavior.
This selection does not implement constraint solver behavior.
This selection does not implement constraint satisfaction.
This selection does not implement layout solving.
This selection only authorizes the next boundary package to be prepared under a separate gate.

## 9. Deferred Lanes
- Measuring Seed / Source — Deferred / too early
- Size-to-Fit Boundary — Deferred
- Constraint Solver Boundary — Deferred / too early
- Layout Solving Boundary — Deferred / too early
- Event Boundary — Deferred / high-risk
- Backend Boundary — Deferred / too early
- Layout Metadata / Algorithm / Measuring Consolidation Audit — Deferred

## 10. Untracked Workspace Artifacts
Untracked workspace artifacts are treated as local-only, non-merged artifacts.
They must not be staged or committed by this roadmap selection PR.

Known artifacts:

- `.claude/`
- `examples/baseline/`
- `scratch/`

## 11. Admission Guard
Sizing algorithm seed remains deterministic renderer-local metadata derivation substrate. Measuring boundary remains planning-only and does not implement measuring, fit, solver, or layout-solving authority. This roadmap selection PR remains docs-only and does not change source, tests, or dependencies.

## 12. Non-Scope
- no source changes
- no test changes
- no docs/DNA.md changes
- no docs/dna changes
- no agent skill changes
- no Cargo.toml / Cargo.lock changes
- no dependency additions
- no measuring source
- no text/glyph/image/widget measurement
- no font/backend/GPU measurement
- no size-to-fit behavior
- no intrinsic/content size calculation as executable behavior
- no constraint solver
- no constraint satisfaction
- no layout solving
- no draw/event/backend implementation
- no runtime/verifier/VM integration
- no capability admission
- no proof/debugger authority
- no Workbench/Studio integration

## 13. Final Decision
Final decision:
PASS WITH WARNINGS — POST-UI next lane selected after layout sizing algorithm seed audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-MEASURING-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement measuring source, text/glyph/image/widget measurement, font/backend/GPU measurement, size-to-fit behavior, intrinsic/content size calculation as executable behavior, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, and not merged.
