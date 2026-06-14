# POST-UI Roadmap Next Lane Selection After Layout Fit Metadata Stack Consolidation Audit

## 1. Purpose

This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Fit Metadata Stack Consolidation Audit.

## 2. DNA Alignment

DNA inspected: YES
DNA source path: docs/dna
docs/dna directory present: YES
docs/DNA.md present: NO
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout fit metadata stack remains renderer-local;
- metadata layers remain deterministic;
- metadata layers remain source-reference-preserving;
- metadata layers remain non-mutating;
- constraint solver boundary selection remains docs-only;
- constraint solver boundary selection does not implement solver behavior;
- constraint solver boundary selection does not implement constraint satisfaction;
- constraint solver boundary selection does not implement layout solving;
- constraint solver boundary selection does not mutate layout/geometry/sizing/sizing-algorithm/measuring/size-to-fit metadata;
- constraint solver boundary selection does not introduce executable fit/fill/shrink/grow behavior;
- constraint solver boundary selection does not introduce intrinsic/content size calculation;
- constraint solver boundary selection does not introduce real measuring;
- constraint solver boundary selection does not introduce draw/event/backend authority;
- constraint solver boundary selection does not introduce runtime/verifier/VM/capability authority;
- constraint solver boundary selection does not introduce proof/debugger authority;
- constraint solver boundary selection does not introduce Workbench/Studio integration.

## 3. Closed Basis

#1032 — roadmap selected size-to-fit seed
#1033 — layout size-to-fit seed source
#1034 — layout size-to-fit seed closeout
#1035 — layout size-to-fit seed ledger audit
#1036 — roadmap selected fit metadata stack consolidation audit
#1037 — fit metadata stack consolidation audit

## 4. Fit Metadata Stack State

The current renderer layout fit metadata stack is consolidated as deterministic renderer-local metadata from layout through geometry, constraints, sizing, sizing algorithm, measuring, and size-to-fit.

The stack remains source-reference-preserving, non-mutating, metadata-only, and does not implement executable fit/fill/shrink/grow behavior, intrinsic/content size calculation as executable behavior, real measuring, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

After this consolidation, the next selected lane is a docs-only constraint solver boundary. That boundary must define solver authority before any future solver source package is admitted.

## 5. Project #2 State

Project #2 metadata aligns with the closed basis for #1032, #1033, #1034, #1035, #1036, and #1037.
All items are present with 1 instance each (0 duplicates).

## 6. Candidate Lanes

| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Constraint Solver Boundary | Selected | Next structurally safe step to define solver authority as a docs-only boundary | Low | Proceed |
| Constraint Solver Seed / Source | Deferred / too early | Boundary must exist before source | High | Defer |
| Layout Solving Boundary | Deferred / too early | Requires constraint solver boundary first | High | Defer |
| Real Size-to-Fit Implementation | Deferred / too early | Executable fitting remains outside current envelope | High | Defer |
| Real Measuring Implementation | Deferred / forbidden for now | Real measuring remains outside current envelope | High | Defer |
| Backend Boundary | Deferred / too early | Outside current envelope | High | Defer |
| Event Boundary | Deferred / high-risk | Should wait until layout authority is mature | High | Defer |

## 7. Selection Criteria

1. Must preserve DNA alignment.
2. Must preserve renderer-local layout metadata stack boundaries.
3. Must define constraint solver authority before implementing it.
4. Must not start the constraint solver boundary package.
5. Must not implement solver source.
6. Must not implement constraint satisfaction.
7. Must not implement equation solving.
8. Must not implement relation solving.
9. Must not implement iterative convergence.
10. Must not implement layout solving.
11. Must not mutate geometry/layout/sizing/sizing-algorithm/measuring/size-to-fit metadata.
12. Must not introduce executable fit/fill/shrink/grow behavior.
13. Must not introduce intrinsic/content size calculation.
14. Must not introduce real measuring.
15. Must not introduce draw/event/backend authority.
16. Must not introduce runtime/verifier/VM/capability authority.
17. Must not introduce proof/debugger authority.
18. Must not introduce Workbench/Studio integration.

## 8. Selected Next Lane

Selected next lane:
R12-UI-RENDERER-LAYOUT-CONSTRAINT-SOLVER-BOUNDARY-LINE-FULL-PACKAGE

## 9. Deferred Lanes

Constraint Solver Seed / Source
Layout Solving Boundary
Real Size-to-Fit Implementation
Real Measuring Implementation
Backend Boundary
Event Boundary

## 10. Untracked Workspace Artifacts

Untracked workspace artifacts are treated as local-only, non-merged artifacts.
They must not be staged, committed, deleted, or merged by this roadmap selection PR.

## 11. Admission Guard

This selection is planning-only.
This selection does not start the constraint solver boundary package.
This selection does not change source.
This selection does not change tests.
This selection does not implement constraint solver behavior.
This selection does not implement constraint satisfaction.
This selection does not implement equation solving.
This selection does not implement relation solving.
This selection does not implement iterative convergence.
This selection does not implement layout solving.
This selection does not mutate geometry/layout/sizing/sizing-algorithm/measuring/size-to-fit metadata.
This selection does not introduce executable fit/fill/shrink/grow behavior.
This selection does not introduce intrinsic/content size calculation.
This selection does not introduce real measuring.
This selection does not introduce draw/event/backend/runtime/capability/proof/Workbench/Studio authority.

## 12. Non-Scope

- no source changes
- no test changes
- no agent skill changes
- no Cargo.toml / Cargo.lock changes
- no constraint solver boundary document in this PR
- no executable behavior
- no local untracked artifact deletion

## 13. Final Decision

Final decision:
PASS — POST-UI next lane selected after layout fit metadata stack consolidation audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-CONSTRAINT-SOLVER-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not start the constraint solver boundary package, change source, change tests, implement constraint solver behavior, implement constraint satisfaction, implement equation solving, implement relation solving, implement iterative convergence, implement layout solving, mutate geometry/layout/sizing/sizing-algorithm/constraints/measuring/size-to-fit metadata, or introduce executable fit/fill/shrink/grow behavior, intrinsic/content size calculation, real measuring, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

Tracked repository state remains clean for this roadmap selection. Pre-existing untracked local workspace artifacts are not staged, not committed, not deleted, and not merged.
