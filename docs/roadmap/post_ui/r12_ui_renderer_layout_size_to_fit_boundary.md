# R12 UI Renderer Layout Size-to-Fit Boundary

## 1. Purpose
This document defines the R12 UI Renderer Layout Size-to-Fit Boundary after the completed and audited renderer layout metadata stack consolidation.

## 2. DNA Alignment
- DNA inspected: YES
- DNA source path: `docs/dna/SEMANTIC_UI_DNA.md`
- docs/dna directory present: YES
- docs/DNA.md present: NO
- DNA conflicts detected: NONE
- DNA-driven constraints applied:
  - renderer/UI remains downstream;
  - layout metadata stack remains renderer-local;
  - geometry seed remains inert renderer-local metadata;
  - constraints seed remains inert renderer-local metadata declarations;
  - sizing seed remains inert renderer-local metadata/result declarations;
  - sizing algorithm seed remains deterministic renderer-local metadata derivation substrate;
  - measuring boundary remains docs-only and audited;
  - measuring seed remains deterministic renderer-local measurement metadata/request substrate;
  - metadata stack consolidation audit is complete;
  - size-to-fit boundary remains planning/documentation only;
  - no size-to-fit source implementation;
  - no fit/fill/shrink/grow behavior;
  - no intrinsic/content size calculation as executable behavior;
  - no real text/glyph/image/widget measurement;
  - no font/backend/GPU measurement;
  - no WGPU/winit/Tauri measurement;
  - no constraint solver implementation;
  - no constraint satisfaction implementation;
  - no layout solving implementation;
  - no geometry/layout/sizing/constraints/measuring mutation;
  - no draw/event/backend authority;
  - no runtime/verifier/VM/capability authority;
  - no proof/debugger authority;
  - no Workbench/Studio integration.

## 3. Closed Basis
- #1022 — roadmap selected measuring seed
- #1023 — layout measuring seed source
- #1024 — layout measuring seed closeout
- #1025 — layout measuring seed ledger audit
- #1026 — roadmap selected metadata stack consolidation audit
- #1027 — layout metadata stack consolidation audit
- #1028 — roadmap selected size-to-fit boundary

## 4. Size-to-Fit Position in Pipeline
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
UiLayoutSizingAlgorithmModel
  ↓
UiLayoutMeasuringModel
  ↓
Future size-to-fit boundary
  ↓
Future constraint solver / layout refinement layers, not part of this boundary
  ↓
Future draw/backend/event layers, not part of this boundary

## 5. Size-to-Fit Boundary Definition
Size-to-Fit Boundary defines where a future deterministic renderer-local size-to-fit layer may sit and what authority it must not receive.

It does not implement size-to-fit source.
It does not implement fit/fill/shrink/grow behavior.
It does not implement intrinsic/content size calculation as executable behavior.
It does not implement real measuring.
It does not implement constraint solver behavior.
It does not implement layout solving.
It does not mutate layout, geometry, sizing, constraints, sizing algorithm, or measuring metadata.

## 6. Allowed Future Inputs
The boundary may admit future consumption of:

- UiLayoutModel
- UiLayoutGeometryModel
- UiLayoutConstraintsModel
- UiLayoutSizingModel
- UiLayoutSizingAlgorithmModel
- UiLayoutMeasuringModel
- layout nodes
- geometry nodes
- constraint declarations
- sizing entries
- sizing algorithm entries
- measuring entries
- stable source references exposed by existing APIs

No size-to-fit input contract is implemented in this PR.
This is a boundary-level declaration only.

## 7. Allowed Future Outputs
The boundary may describe future outputs as:

- deterministic size-to-fit model;
- deterministic size-to-fit entries;
- fit/fill/shrink/grow intent metadata;
- deferred fit result metadata;
- unavailable/unknown fit metadata;
- audit metadata for future source admission;
- source references back to layout/geometry/constraints/sizing/sizing-algorithm/measuring metadata.

No size-to-fit structs, IDs, functions, tests, or behavior are implemented in this PR.

## 8. Conceptual Future Size-to-Fit Categories
The boundary may name future conceptual categories, strictly as non-implemented concepts:

- unresolved fit intent;
- deferred fit intent;
- unavailable fit result;
- audit-only fit declaration;
- fill request;
- shrink request;
- grow request;
- clamp request;

These are future conceptual categories only.
They are not implemented by this PR.

## 9. Explicit Separation From Real Measuring
Size-to-Fit Boundary does not admit real measuring.

It may not measure text, glyphs, images, widgets, fonts, backend surfaces, GPU surfaces, or rendered content.

Future size-to-fit may consume measuring metadata only after a separate source gate, but it may not create measuring authority by itself.

## 10. Explicit Separation From Intrinsic/Content Size Calculation
Size-to-Fit Boundary does not admit intrinsic or content size calculation as executable behavior.

Intrinsic/content calculation remains separately bounded and cannot be smuggled into fit/fill/shrink/grow semantics.

## 11. Explicit Separation From Constraint Solver
Size-to-Fit Boundary does not admit constraint solver authority.

Constraint satisfaction, equation solving, relation solving, or iterative convergence remains forbidden.

If solver authority is needed later, it must be selected as a separate boundary lane.

## 12. Explicit Separation From Layout Solving
Size-to-Fit Boundary does not admit layout solving authority.

It may not arrange nodes, place nodes, mutate geometry, rewrite layout, resolve final rectangles, or produce draw-ready layout.

Layout solving remains a later separately bounded lane.

## 13. Explicit Separation From Metadata Mutation
Size-to-Fit Boundary does not admit mutation of layout, geometry, sizing, constraints, sizing algorithm, or measuring metadata.

Future size-to-fit source, if admitted, must preserve input models unless a later explicit mutation/refinement boundary is selected.

## 14. Explicit Non-Authority Rules
- no draw commands
- no event handling
- no event dispatch
- no backend/WGPU/winit/Tauri
- no runtime/verifier/VM integration
- no capability admission
- no action execution
- no effect authorization
- no proof/debugger authority
- no Workbench/Studio integration

## 15. Deferred Implementation
Deferred to a future source package:
`R12-UI-RENDERER-LAYOUT-SIZE-TO-FIT-SEED-LINE-FULL-PACKAGE`

That future package must not proceed until this boundary is closed and audited.

## 16. Candidate Future Source Gate
The only future gate implied by this boundary is a separately selected source package for size-to-fit metadata interpretation.

It remains outside this PR and outside this boundary.

## 17. Admission Guard
| Surface | Boundary status | Admission classification | Status |
|---|---|---|---|
| future deterministic size-to-fit metadata | allowed only after source gate | ADMITTED FUTURE | PASS |
| current size-to-fit source | absent | DEFERRED | PASS |
| fit/fill/shrink/grow behavior | forbidden | FORBIDDEN | PASS |
| intrinsic/content size calculation | forbidden | FORBIDDEN | PASS |
| real text/glyph/image/widget measurement | forbidden | FORBIDDEN | PASS |
| font/backend/GPU measurement | forbidden | FORBIDDEN | PASS |
| WGPU/winit/Tauri measurement | forbidden | FORBIDDEN | PASS |
| constraint solver | forbidden | FORBIDDEN | PASS |
| constraint satisfaction | forbidden | FORBIDDEN | PASS |
| layout solving | forbidden | FORBIDDEN | PASS |
| layout engine rewrite | forbidden | FORBIDDEN | PASS |
| geometry/layout/sizing/constraints/measuring mutation | forbidden | FORBIDDEN | PASS |
| draw commands | forbidden | FORBIDDEN | PASS |
| backend API | forbidden | FORBIDDEN | PASS |
| event handlers | forbidden | FORBIDDEN | PASS |
| runtime/verifier/VM | forbidden | FORBIDDEN | PASS |
| capability admission | forbidden | FORBIDDEN | PASS |
| semantic truth authority | forbidden | FORBIDDEN | PASS |
| proof/debugger authority | forbidden | FORBIDDEN | PASS |

## 18. Evidence Matrix
| Area | Current state | Classification | Status |
|---|---|---|---|
| Size-to-fit boundary | Documented | ADMITTED | PASS |
| Size-to-fit implementation | Not implemented | DEFERRED | PASS |
| Size-to-fit seed | Future package only | DEFERRED | PASS |
| Fit/fill/shrink/grow behavior | Not implemented | FORBIDDEN | PASS |
| Intrinsic/content size calculation | Not implemented | FORBIDDEN | PASS |
| Real text measurement | Not implemented | FORBIDDEN | PASS |
| Real glyph measurement | Not implemented | FORBIDDEN | PASS |
| Real image measurement | Not implemented | FORBIDDEN | PASS |
| Real widget measurement | Not implemented | FORBIDDEN | PASS |
| Font system integration | Not implemented | FORBIDDEN | PASS |
| Backend/GPU measurement | Not implemented | FORBIDDEN | PASS |
| WGPU/winit/Tauri | Not implemented | FORBIDDEN | PASS |
| Constraint solver | Not implemented | FORBIDDEN | PASS |
| Constraint satisfaction | Not implemented | FORBIDDEN | PASS |
| Layout solving | Not implemented | FORBIDDEN | PASS |
| Layout engine rewrite | Not implemented | FORBIDDEN | PASS |
| Geometry mutation | Not implemented | FORBIDDEN | PASS |
| Layout mutation | Not implemented | FORBIDDEN | PASS |
| Sizing metadata mutation | Not implemented | FORBIDDEN | PASS |
| Constraint mutation | Not implemented | FORBIDDEN | PASS |
| Measuring mutation | Not implemented | FORBIDDEN | PASS |
| Draw/render backend | Not implemented | FORBIDDEN | PASS |
| Event dispatch | Not implemented | FORBIDDEN | PASS |
| Action execution | Not implemented | FORBIDDEN | PASS |
| Effect authorization | Not implemented | FORBIDDEN | PASS |
| Runtime/verifier/VM | Not implemented | FORBIDDEN | PASS |
| Capability admission | Not implemented | FORBIDDEN | PASS |
| Proof/debugger authority | Not implemented | FORBIDDEN | PASS |
| Workbench/Studio | Not implemented | FORBIDDEN | PASS |
| Dependency additions | None | FORBIDDEN | PASS |

## 19. Final Decision
Final decision:
PASS — R12 UI Renderer Layout Size-to-Fit Boundary defined.

This boundary admits future size-to-fit work only as a separately gated deterministic renderer-local metadata interpretation layer.

This PR does not implement size-to-fit source, fit/fill/shrink/grow behavior, intrinsic/content size calculation as executable behavior, real text/glyph/image/widget measurement, font/backend/GPU measurement, WGPU/winit/Tauri measurement, constraint solver behavior, constraint satisfaction, layout solving, layout engine rewrite, geometry/layout/sizing/constraints/measuring mutation, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.
