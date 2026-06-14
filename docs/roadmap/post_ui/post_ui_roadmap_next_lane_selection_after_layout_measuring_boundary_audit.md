# POST-UI Roadmap Next Lane Selection After Layout Measuring Boundary Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Measuring Boundary line.

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
- measuring boundary is closed and audited for tracked repository state;
- measuring seed may only introduce deterministic renderer-local measurement metadata/request substrate;
- measuring seed must not implement actual text/glyph/image/widget measurement;
- measuring seed must not introduce font/backend/GPU measurement authority;
- measuring seed must not introduce WGPU/winit/Tauri authority;
- measuring seed must not introduce size-to-fit authority;
- measuring seed must not introduce intrinsic/content size calculation as executable behavior;
- measuring seed must not introduce constraint solver authority;
- measuring seed must not introduce constraint satisfaction authority;
- measuring seed must not introduce layout solving;
- measuring seed must not introduce draw/event/backend authority;
- measuring seed must not introduce runtime/verifier/VM/capability authority;
- measuring seed must not introduce proof/debugger authority;
- measuring seed must not introduce Workbench/Studio integration;
- this roadmap PR must remain docs-only.

## 3. Closed Basis
- #1018 — roadmap selected measuring boundary
- #1019 — layout measuring boundary
- #1020 — layout measuring boundary closeout
- #1021 — layout measuring boundary ledger audit

## 4. Measuring Boundary State
Measuring boundary is closed as docs-only boundary work. It documents future measuring authority as a separately gated deterministic renderer-local metadata acquisition layer without implementing measuring source, text/glyph/image/widget measurement, font/backend/GPU measurement, WGPU/winit/Tauri measurement, size-to-fit behavior, intrinsic/content size calculation as executable behavior, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 5. Project #2 State
- #1018: Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | #1016
- #1019: Done | POST-UI | R12 | Docs | High | Renderer | Docs-only | Roadmap doc | #1018
- #1020: Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | #1019
- #1021: Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | #1020

## 6. Candidate Lanes
| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Measuring Seed | Selected | Measuring boundary has been selected, documented, closed out, and ledger-audited. The next structurally valid step is a small source seed for deterministic renderer-local measurement metadata/request substrate, still without real text/glyph/image/widget measurement, font/backend/GPU measurement, size-to-fit, solver, or layout-solving authority. | Medium | Selected |
| Measuring Implementation / Real Measurement | Deferred / too early | Too early. Measuring seed must remain metadata/request substrate only before any real measurement implementation. | High | Deferred |
| Size-to-Fit Boundary | Deferred | Fit/fill/shrink/grow behavior is higher-authority and should remain separated from measuring seed. | Medium | Deferred |
| Constraint Solver Boundary | Deferred / too early | Constraint solver boundary remains premature until measuring seed exists and solver authority remains explicitly separated. | High | Deferred |
| Layout Solving Boundary | Deferred / too early | Layout solving is higher-authority and should wait until measuring, fit, and solver boundaries are separately handled. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should wait until layout authority is more mature. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside current layout metadata and measuring seed work. | High | Deferred |
| Layout Metadata / Algorithm / Measuring Consolidation Audit | Deferred | Useful later, but the immediate architectural pressure point is creating the minimal measuring seed. | Medium | Deferred |

## 7. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer/layout/geometry/constraints/sizing seed inertness.
3. Must preserve sizing algorithm seed as metadata derivation substrate only.
4. Must preserve measuring boundary as docs-only authority boundary.
5. Must not implement measuring source in the roadmap selection PR.
6. Must not introduce text/glyph/image/widget measurement.
7. Must not introduce font/backend/GPU measurement.
8. Must not introduce WGPU/winit/Tauri measurement.
9. Must not introduce size-to-fit behavior.
10. Must not introduce intrinsic/content size calculation as executable behavior.
11. Must not introduce constraint solver behavior.
12. Must not introduce constraint satisfaction behavior.
13. Must not introduce layout solving.
14. Must not introduce draw/event/backend.
15. Must not introduce runtime/verifier/VM/capability authority.
16. Must build naturally on closed measuring boundary and audit.
17. Must be source-gated separately before implementation.

## 8. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-MEASURING-SEED-LINE-FULL-PACKAGE

This selection is planning-only.
This selection does not implement measuring source.
This selection does not implement real text/glyph/image/widget measurement.
This selection does not implement font/backend/GPU measurement.
This selection does not implement WGPU/winit/Tauri measurement.
This selection does not implement size-to-fit behavior.
This selection does not implement intrinsic/content size calculation as executable behavior.
This selection does not implement constraint solver behavior.
This selection does not implement constraint satisfaction.
This selection does not implement layout solving.
This selection only authorizes the next source package to be prepared under a separate gate.

## 9. Deferred Lanes
- Measuring Implementation / Real Measurement — Deferred / too early
- Size-to-Fit Boundary — Deferred
- Constraint Solver Boundary — Deferred / too early
- Layout Solving Boundary — Deferred / too early
- Event Boundary — Deferred / high-risk
- Backend Boundary — Deferred / too early
- Layout Metadata / Algorithm / Measuring Consolidation Audit — Deferred

## 10. Untracked Workspace Artifacts
Untracked workspace artifacts were present in the prior audit and are treated as local-only, non-merged artifacts.
They must not be staged or committed by this roadmap selection PR.

Known artifacts:

- .claude/
- examples/baseline/
- scratch/

## 11. Admission Guard
| Surface | Boundary status | Admission classification | Status |
|---|---|---|---|
| future deterministic measurement metadata/request substrate | allowed only after source gate | ADMITTED FUTURE | PASS |
| current measuring source implementation | absent | DEFERRED | PASS |
| text/glyph/image/widget measurement | forbidden | FORBIDDEN | PASS |
| font/backend/GPU measurement | forbidden | FORBIDDEN | PASS |
| WGPU/winit/Tauri measurement | forbidden | FORBIDDEN | PASS |
| size-to-fit behavior | forbidden | FORBIDDEN | PASS |
| intrinsic/content size calculation | forbidden | FORBIDDEN | PASS |
| constraint solver | forbidden | FORBIDDEN | PASS |
| constraint satisfaction | forbidden | FORBIDDEN | PASS |
| layout solving | forbidden | FORBIDDEN | PASS |
| layout engine rewrite | forbidden | FORBIDDEN | PASS |
| draw commands | forbidden | FORBIDDEN | PASS |
| backend API | forbidden | FORBIDDEN | PASS |
| event handlers | forbidden | FORBIDDEN | PASS |
| runtime/verifier/VM | forbidden | FORBIDDEN | PASS |
| capability admission | forbidden | FORBIDDEN | PASS |
| semantic truth authority | forbidden | FORBIDDEN | PASS |
| proof/debugger authority | forbidden | FORBIDDEN | PASS |

## 12. Non-Scope
- no source changes
- no test changes
- no docs/DNA.md changes
- no docs/dna changes
- no agent skill changes
- no Cargo.toml / Cargo.lock changes
- no dependency additions
- no measuring source
- no measuring structs
- no measuring IDs
- no measuring functions
- no measuring tests
- no text/glyph/image/widget measurement
- no font/backend/GPU measurement
- no WGPU/winit/Tauri measurement
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
PASS — POST-UI next lane selected after layout measuring boundary audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-MEASURING-SEED-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement measuring source, real text/glyph/image/widget measurement, font/backend/GPU measurement, WGPU/winit/Tauri measurement, size-to-fit behavior, intrinsic/content size calculation as executable behavior, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, and not merged.
