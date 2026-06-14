# R12 UI Renderer Layout Measuring Boundary

## 1. Purpose
This document defines the R12 UI Renderer Layout Measuring Boundary after the completed and audited renderer layout sizing algorithm seed line.

## 2. DNA Alignment
DNA inspected: YES
DNA source path: docs/dna/SEMANTIC_UI_DNA.md; docs/DNA.md
docs/dna directory present: YES
docs/DNA.md present: YES
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- geometry seed remains inert renderer-local metadata;
- constraints seed remains inert renderer-local metadata declarations;
- sizing seed remains inert renderer-local metadata/result declarations;
- sizing algorithm seed remains deterministic renderer-local metadata derivation substrate;
- measuring boundary remains docs-only;
- measuring boundary must not introduce measuring source;
- measuring boundary must not introduce text/glyph/image/widget measurement;
- measuring boundary must not introduce font/backend/GPU measurement authority;
- measuring boundary must not introduce WGPU/winit/Tauri authority;
- measuring boundary must not introduce size-to-fit authority;
- measuring boundary must not introduce intrinsic/content size calculation as executable behavior;
- measuring boundary must not introduce constraint solver authority;
- measuring boundary must not introduce constraint satisfaction authority;
- measuring boundary must not introduce layout solving;
- measuring boundary must not introduce draw/event/backend authority;
- measuring boundary must not introduce runtime/verifier/VM/capability authority;
- measuring boundary must not introduce proof/debugger authority;
- measuring boundary must not introduce Workbench/Studio integration.

## 3. Closed Basis
- #1013 — roadmap selected sizing algorithm seed
- #1014 — layout sizing algorithm seed source
- #1015 — layout sizing algorithm seed closeout
- #1016 — layout sizing algorithm seed ledger audit
- #1017 — roadmap selected measuring boundary

## 4. Measuring Position in Pipeline
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
Future measuring boundary
  ↓
Future size-to-fit / solver / layout refinement layers, not part of this boundary
  ↓
Future draw/backend/event layers, not part of this boundary

## 5. Measuring Boundary Definition
Measuring Boundary defines where a future deterministic renderer-local measuring layer may sit and what authority it must not receive.

It does not implement measuring source.

It does not implement text/glyph/image/widget measurement.

It does not implement font/backend/GPU measurement.

It does not implement size-to-fit behavior.

It does not implement constraint solver behavior.

It does not implement layout solving.

## 6. Allowed Future Inputs
The boundary may admit future consumption of:

- UiLayoutModel
- UiLayoutGeometryModel
- UiLayoutConstraintsModel
- UiLayoutSizingModel
- UiLayoutSizingAlgorithmModel
- layout nodes
- geometry nodes
- constraint declarations
- sizing entries
- sizing algorithm entries
- stable source references exposed by existing APIs

No measuring input contract is implemented in this PR. This is a boundary-level declaration only.

## 7. Allowed Future Outputs
The boundary may describe future outputs as:

- deterministic measuring model;
- deterministic measuring entries;
- measurement request metadata;
- deferred measurement result metadata;
- unavailable/unknown measurement metadata;
- audit metadata for future source admission;
- source references back to layout/geometry/constraints/sizing/sizing-algorithm metadata.

No measuring structs, IDs, functions, tests, or behavior are implemented in this PR.

## 8. Conceptual Future Measuring Categories
The boundary may name future conceptual categories, strictly as non-implemented concepts:

- unresolved measurement;
- deferred measurement;
- unavailable measurement;
- content measurement request;
- text measurement request;
- glyph measurement request;
- image measurement request;
- widget measurement request;
- audit-only measurement;

These are future conceptual categories only.
They are not implemented by this PR.

## 9. Explicit Separation From Text/Glyph/Image/Widget Measurement
Measuring Boundary does not admit executable text, glyph, image, or widget measurement.

No shaping, font metrics, glyph bounds, image decoding, image probing, widget probing, or backend-dependent measurement is allowed in this boundary.

## 10. Explicit Separation From Font/Backend/GPU Authority
Measuring Boundary does not admit font system, backend, GPU, WGPU, winit, or Tauri authority.

Any future backend-dependent measurement must be separately bounded before source implementation.

## 11. Explicit Separation From Size-to-Fit
Measuring Boundary does not admit size-to-fit authority.

Fit/fill/shrink/grow behavior remains forbidden as executable behavior in this boundary.

If fit behavior is needed later, it must be separately bounded before source implementation.

## 12. Explicit Separation From Constraint Solver
Measuring Boundary does not admit constraint solver authority.

Constraint satisfaction, equation solving, relation solving, or iterative convergence remains forbidden.

If solver authority is needed later, it must be selected as a separate boundary lane.

## 13. Explicit Separation From Layout Solving
Measuring Boundary does not admit layout solving authority.

It may not arrange nodes, place nodes, mutate geometry, rewrite layout, resolve final rectangles, or produce draw-ready layout.

Layout solving remains a later separately bounded lane.

## 14. Explicit Non-Authority Rules
- no measuring source implementation
- no text measurement
- no glyph measurement
- no image measurement
- no widget measurement
- no font integration
- no backend/GPU measurement
- no WGPU/winit/Tauri integration
- no size-to-fit implementation
- no intrinsic/content size calculation as executable behavior
- no constraint solver implementation
- no constraint satisfaction implementation
- no layout solving implementation
- no layout engine rewrite
- no geometry mutation
- no layout mutation
- no sizing metadata mutation
- no constraint mutation
- no draw commands
- no event handling
- no event dispatch
- no runtime/verifier/VM integration
- no capability admission
- no action execution
- no effect authorization
- no proof/debugger authority
- no Workbench/Studio integration

## 15. Deferred Implementation
Deferred to a future source package:
R12-UI-RENDERER-LAYOUT-MEASURING-SEED-LINE-FULL-PACKAGE

That future package must not proceed until this boundary is closed and audited.

## 16. Candidate Future Source Gate
Candidate future source gate:
R12-UI-RENDERER-LAYOUT-MEASURING-SEED-LINE-FULL-PACKAGE

This future package remains deferred until a separate gate admits measuring source implementation.

## 17. Admission Guard
| Surface | Boundary status | Admission classification | Status |
|---|---|---|---|
| future deterministic measuring metadata | allowed only after source gate | ADMITTED FUTURE | PASS |
| current measuring source | absent | DEFERRED | PASS |
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

## 18. Evidence Matrix
| Area | Current state | Classification | Status |
|---|---|---|---|
| Measuring boundary | Documented | ADMITTED | PASS |
| Measuring implementation | Not implemented | DEFERRED | PASS |
| Measuring seed | Future package only | DEFERRED | PASS |
| Text measurement | Not implemented | FORBIDDEN | PASS |
| Glyph measurement | Not implemented | FORBIDDEN | PASS |
| Image measurement | Not implemented | FORBIDDEN | PASS |
| Widget measurement | Not implemented | FORBIDDEN | PASS |
| Font system integration | Not implemented | FORBIDDEN | PASS |
| Backend/GPU measurement | Not implemented | FORBIDDEN | PASS |
| WGPU/winit/Tauri | Not implemented | FORBIDDEN | PASS |
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

## 19. Final Decision
Final decision:
PASS — R12 UI Renderer Layout Measuring Boundary defined.

This boundary admits future measuring work only as a separately gated deterministic renderer-local metadata acquisition layer.

This PR does not implement measuring source, text/glyph/image/widget measurement, font/backend/GPU measurement, size-to-fit behavior, intrinsic/content size calculation as executable behavior, constraint solver behavior, constraint satisfaction, layout solving, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.
