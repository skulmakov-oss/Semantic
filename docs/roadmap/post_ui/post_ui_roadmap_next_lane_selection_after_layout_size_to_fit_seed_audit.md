# POST-UI Roadmap Next Lane Selection After Layout Size-to-Fit Seed Audit

## 1. Purpose

This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Size-to-Fit Seed line.

## 2. DNA Alignment

DNA inspected: YES
DNA source path: docs/dna
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
- size-to-fit seed remains deterministic renderer-local fit metadata / intent substrate;
- fit metadata stack consolidation audit must remain docs-only;
- fit metadata stack consolidation audit must not introduce source behavior;
- fit metadata stack consolidation audit must not introduce executable fit/fill/shrink/grow behavior;
- fit metadata stack consolidation audit must not introduce intrinsic/content size calculation as executable behavior;
- fit metadata stack consolidation audit must not introduce real measuring;
- fit metadata stack consolidation audit must not introduce font/backend/GPU/WGPU/winit/Tauri authority;
- fit metadata stack consolidation audit must not introduce constraint solver authority;
- fit metadata stack consolidation audit must not introduce constraint satisfaction authority;
- fit metadata stack consolidation audit must not introduce layout solving;
- fit metadata stack consolidation audit must not mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit metadata;
- fit metadata stack consolidation audit must not introduce draw/event/backend authority;
- fit metadata stack consolidation audit must not introduce runtime/verifier/VM/capability authority;
- fit metadata stack consolidation audit must not introduce proof/debugger authority;
- fit metadata stack consolidation audit must not introduce Workbench/Studio integration;
- this roadmap selection does not perform the fit metadata stack consolidation audit.

## 3. Closed Basis

#1028 — roadmap selected size-to-fit boundary
#1029 — layout size-to-fit boundary
#1030 — layout size-to-fit boundary closeout
#1031 — layout size-to-fit boundary ledger audit
#1032 — roadmap selected size-to-fit seed
#1033 — layout size-to-fit seed source
#1034 — layout size-to-fit seed closeout
#1035 — layout size-to-fit seed ledger audit

## 4. Extended Fit Metadata Stack State

The current renderer layout metadata stack is now extended through size-to-fit metadata:

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

The stack remains deterministic, renderer-local, source-reference-preserving, non-mutating, metadata-only, and does not implement executable fit/fill/shrink/grow behavior, intrinsic/content size calculation as executable behavior, real measuring, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Before introducing solver authority, layout solving authority, real fit execution, real measuring, or backend/event/runtime/capability surfaces, the next selected lane is a docs-only consolidation audit across the extended fit metadata stack.

## 5. Project #2 State

Project #2 metadata aligns with the closed basis for #1028, #1029, #1030, #1031, #1032, #1033, #1034, and #1035.
All 8 items are present with 1 instance each (0 duplicates).

## 6. Candidate Lanes

| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Fit Metadata Stack Consolidation Audit | Selected | Consolidation needed before executing solving | Low | Proceed |
| Constraint Solver Boundary | Deferred | Requires fit consolidation first | Medium | Defer |
| Layout Solving Boundary | Deferred / too early | Requires solver boundary first | High | Defer |
| Real Size-to-Fit Implementation | Deferred / too early | Requires layout solving maturity first | High | Defer |
| Real Measuring Implementation | Deferred / forbidden for now | Out of scope for layout metadata | High | Defer |
| Backend Boundary | Deferred / too early | Out of scope for layout metadata | High | Defer |
| Event Boundary | Deferred / high-risk | Out of scope for layout metadata | High | Defer |

## 7. Selection Criteria

1. Must preserve DNA alignment.
2. Must preserve renderer-local layout metadata stack boundaries.
3. Must preserve geometry seed inertness.
4. Must preserve constraints seed inertness.
5. Must preserve sizing seed inertness.
6. Must preserve sizing algorithm seed as metadata derivation substrate only.
7. Must preserve measuring seed as metadata/request substrate only.
8. Must preserve size-to-fit seed as metadata/intent substrate only.
9. Must build on the completed size-to-fit seed ledger audit.
10. Must not perform the fit metadata stack consolidation audit in this roadmap PR.
11. Must not introduce source changes.
12. Must not introduce test changes.
13. Must not introduce executable fit/fill/shrink/grow behavior.
14. Must not introduce intrinsic/content size calculation as executable behavior.
15. Must not introduce real measuring.
16. Must not introduce constraint solver behavior.
17. Must not introduce constraint satisfaction behavior.
18. Must not introduce layout solving.
19. Must not introduce geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit mutation.
20. Must not introduce draw/event/backend.
21. Must not introduce runtime/verifier/VM/capability authority.
22. Must select an audit gate before higher-authority layout work.

## 8. Selected Next Lane

Selected next lane:
R12-UI-RENDERER-LAYOUT-FIT-METADATA-STACK-CONSOLIDATION-AUDIT-PR

## 9. Deferred Lanes

Constraint Solver Boundary — Deferred
Layout Solving Boundary — Deferred / too early
Real Size-to-Fit Implementation — Deferred / too early
Real Measuring Implementation — Deferred / forbidden for now
Backend Boundary — Deferred / too early
Event Boundary — Deferred / high-risk

## 10. Untracked Workspace Artifacts

Untracked workspace artifacts are treated as local-only, non-merged artifacts.
They must not be staged, committed, deleted, or merged by this roadmap selection PR.

## 11. Admission Guard

This selection is planning-only.
This selection does not perform the fit metadata stack consolidation audit.
This selection does not change source.
This selection does not change tests.
This selection does not implement executable fit/fill/shrink/grow behavior.
This selection does not implement intrinsic/content size calculation as executable behavior.
This selection does not implement real measuring.
This selection does not implement font/backend/GPU measurement.
This selection does not implement WGPU/winit/Tauri measurement.
This selection does not implement constraint solver behavior.
This selection does not implement constraint satisfaction.
This selection does not implement layout solving.
This selection does not mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit metadata.
This selection does not introduce draw/event/backend/runtime/capability/proof/Workbench/Studio authority.
This selection only authorizes the next docs-only audit package to be prepared under a separate gate.

## 12. Non-Scope

- no source changes
- no test changes
- no agent skill changes
- no Cargo.toml / Cargo.lock changes
- no consolidation audit performed in this PR
- no executable behavior
- no local untracked artifact deletion

## 13. Final Decision

Final decision:
PASS — POST-UI next lane selected after layout size-to-fit seed audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-FIT-METADATA-STACK-CONSOLIDATION-AUDIT-PR.

This selection is planning-only and does not perform the fit metadata stack consolidation audit, change source, change tests, implement executable fit/fill/shrink/grow behavior, implement intrinsic/content size calculation as executable behavior, implement real measuring, implement font/backend/GPU measurement, implement WGPU/winit/Tauri measurement, implement constraint solver behavior, implement constraint satisfaction, implement layout solving, mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit metadata, or introduce draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
