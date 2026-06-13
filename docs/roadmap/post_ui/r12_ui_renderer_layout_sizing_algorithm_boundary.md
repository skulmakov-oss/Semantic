# R12 UI Renderer Layout Sizing Algorithm Boundary

## 1. Purpose
This document defines the R12 UI Renderer Layout Sizing Algorithm Boundary after the completed and audited renderer layout sizing seed line.

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
- sizing algorithm boundary remains planning/documentation only;
- no sizing algorithm implementation;
- no measuring algorithm implementation;
- no size-to-fit implementation;
- no intrinsic/content size calculation;
- no constraint solver implementation;
- no constraint satisfaction implementation;
- no layout solving implementation;
- no draw/event/backend authority;
- no runtime/verifier/VM/capability authority;
- no proof/debugger authority;
- no Workbench/Studio integration.

## 3. Closed Basis
- #1005 — roadmap selected sizing seed
- #1006 — layout sizing seed source
- #1007 — layout sizing seed closeout
- #1008 — layout sizing seed ledger audit
- #1009 — roadmap selected sizing algorithm boundary

## 4. Algorithm Position in Pipeline
UiProjectionArtifact
  ↓
UiRenderModel
  ↓
Renderer presentation models
  ↓
UiLayoutModel
  ↓
UiLayoutGeometryModel
  ↓
UiLayoutConstraintsModel
  ↓
UiLayoutSizingModel
  ↓
Future sizing algorithm boundary
  ↓
Future measuring / fitting / solver / layout refinement layers, not part of this boundary
  ↓
Future draw/backend/event layers, not part of this boundary

## 5. Sizing Algorithm Boundary Definition
Sizing Algorithm Boundary defines where a future deterministic sizing derivation layer may sit and what authority it must not receive.

It does not implement sizing algorithm source.

It does not implement measuring algorithm source.

It does not implement size-to-fit behavior.

It does not implement constraint solver behavior.

It does not implement layout solving.

## 6. Allowed Future Inputs
The boundary may admit future consumption of:

- UiLayoutModel
- UiLayoutGeometryModel
- UiLayoutConstraintsModel
- UiLayoutSizingModel
- layout nodes
- geometry nodes
- constraint declarations
- sizing entries
- source layout/geometry/constraints/sizing references exposed by existing APIs

No source code input contract is implemented in this PR.
This is a boundary-level declaration only.

## 7. Allowed Future Outputs
The boundary may describe future outputs as:

- deterministic sizing algorithm model;
- deterministic sizing algorithm entries;
- derived inert sizing result metadata;
- unresolved/deferred sizing result metadata;
- audit metadata for future source admission;
- source references back to layout/geometry/constraints/sizing metadata.

No algorithm structs, IDs, functions, tests, or behavior are implemented in this PR.

## 8. Conceptual Future Algorithm Categories
The boundary may name future conceptual categories, strictly as non-implemented concepts:

- unresolved derivation;
- deterministic pass-through derivation;
- preferred-size derivation;
- min/max bound derivation;
- available-space derivation;
- deferred-fit derivation;
- deferred-measure derivation;
- audit-only derivation.

These are future conceptual categories only.
They are not implemented by this PR.

## 9. Explicit Separation From Measuring
Sizing algorithm boundary does not admit measuring authority.

Measuring content, glyphs, text, images, widgets, GPU surfaces, or backend-dependent objects remains forbidden in this boundary.

If measuring is needed later, it must be selected as a separate boundary lane.

## 10. Explicit Separation From Size-to-Fit
Sizing algorithm boundary does not admit size-to-fit authority.

Fit/fill/shrink/grow behavior remains forbidden as executable behavior in this boundary.

If fit behavior is needed later, it must be separately bounded before source implementation.

## 11. Explicit Separation From Constraint Solver
Sizing algorithm boundary does not admit constraint solver authority.

Constraint satisfaction, equation solving, relation solving, or iterative convergence remains forbidden.

If solver authority is needed later, it must be selected as a separate boundary lane.

## 12. Explicit Separation From Layout Solving
Sizing algorithm boundary does not admit layout solving authority.

It may not arrange nodes, place nodes, mutate geometry, rewrite layout, resolve final rectangles, or produce draw-ready layout.

Layout solving remains a later separately bounded lane.

## 13. Explicit Non-Authority Rules
- no sizing algorithm source;
- no measuring algorithm source;
- no size-to-fit source;
- no intrinsic size calculation;
- no content measurement;
- no constraint solver;
- no constraint satisfaction algorithm;
- no layout solving;
- no layout engine rewrite;
- no geometry mutation;
- no sizing metadata mutation;
- no constraint mutation;
- no draw commands;
- no event handling;
- no event dispatch;
- no backend/WGPU/winit/Tauri;
- no runtime/verifier/VM integration;
- no capability admission;
- no action execution;
- no effect authorization;
- no proof/debugger authority;
- no Workbench/Studio integration.

## 14. Deferred Implementation
Deferred to a future source package:
R12-UI-RENDERER-LAYOUT-SIZING-ALGORITHM-SEED-LINE-FULL-PACKAGE

That future package must not proceed until this boundary is closed and audited.

## 15. Candidate Future Source Gate
The future source gate is intentionally separate from this boundary document.

It may define a deterministic renderer-local sizing derivation layer only after this boundary is accepted, closed, and audited.

## 16. Admission Guard
Sizing algorithm boundary is planning-only. It authorizes only the next boundary package to be prepared under a separate gate. It does not implement sizing algorithm source, measuring algorithm source, size-to-fit behavior, intrinsic/content size calculation, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 17. Evidence Matrix
| Area | Current state | Classification | Status |
|---|---|---|---|
| Sizing algorithm boundary | Documented | ADMITTED | PASS |
| Sizing algorithm implementation | Not implemented | DEFERRED | PASS |
| Sizing algorithm seed | Future package only | DEFERRED | PASS |
| Measuring algorithm | Not implemented | FORBIDDEN | PASS |
| Size-to-fit behavior | Not implemented | FORBIDDEN | PASS |
| Intrinsic/content size calculation | Not implemented | FORBIDDEN | PASS |
| Constraint solver | Not implemented | FORBIDDEN | PASS |
| Constraint satisfaction | Not implemented | FORBIDDEN | PASS |
| Layout solving | Not implemented | FORBIDDEN | PASS |
| Layout engine rewrite | Not implemented | FORBIDDEN | PASS |
| Geometry mutation | Not implemented | FORBIDDEN | PASS |
| Sizing metadata mutation | Not implemented | FORBIDDEN | PASS |
| Constraint mutation | Not implemented | FORBIDDEN | PASS |
| Draw/render backend | Not implemented | FORBIDDEN | PASS |
| Event dispatch | Not implemented | FORBIDDEN | PASS |
| Action execution | Not implemented | FORBIDDEN | PASS |
| Effect authorization | Not implemented | FORBIDDEN | PASS |
| Runtime/verifier/VM | Not implemented | FORBIDDEN | PASS |
| Capability admission | Not implemented | FORBIDDEN | PASS |
| Proof/debugger authority | Not implemented | FORBIDDEN | PASS |
| Workbench/Studio | Not implemented | FORBIDDEN | PASS |
| Dependency additions | None | FORBIDDEN | PASS |

## 18. Final Decision
Final decision:
PASS — R12 UI Renderer Layout Sizing Algorithm Boundary defined.

This boundary admits future sizing algorithm work only as a separately gated deterministic renderer-local metadata derivation layer.

This PR does not implement sizing algorithm source, measuring algorithm source, size-to-fit behavior, intrinsic/content size calculation, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.
