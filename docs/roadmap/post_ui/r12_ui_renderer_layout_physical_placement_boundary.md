# R12 UI Renderer Layout Physical Placement Boundary

## 1. Purpose
This document defines the docs-only boundary for future renderer-local layout physical placement after the audited layout solving result metadata stack.

## 2. DNA Alignment
DNA inspected: YES
DNA source path: `docs/dna/SEMANTIC_UI_DNA.md`
docs/dna directory present: YES
docs/DNA.md present: NO
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout metadata remains renderer-local;
- layout solving result metadata remains renderer-local;
- physical placement boundary remains docs-only;
- physical placement source is not admitted;
- final physical layout is not produced;
- pixel/screen/viewport placement is not admitted;
- backend/event/runtime/capability authority is not admitted;
- Workbench/Studio remains out of scope.

## 3. Basis
#1079 — layout solving implementation source ledger audit
#1080 — selected layout solving implementation metadata stack consolidation audit
#1081 — layout solving implementation metadata stack consolidation audit
#1082 — selected physical placement boundary lane

## 4. Boundary Position
Physical placement is positioned after UiLayoutSolvingResultModel and before any backend rendering, event dispatch, runtime action, capability admission, or Workbench/Studio integration.

## 5. Future Allowed Inputs
Future physical placement may consume:
- UiLayoutSolvingResultModel;
- upstream layout metadata references;
- deterministic renderer-local placement policy metadata;
- separately gated viewport/container descriptors only if admitted by a future source gate.

## 6. Future Allowed Outputs
Future physical placement may produce renderer-local placement metadata.

It must not be treated as backend draw commands.
It must not be treated as event targets.
It must not be treated as runtime authority.
It must not be treated as capability admission.
It must not be treated as Workbench/Studio authority.

## 7. Conceptual Future Categories
Conceptual future categories only:
- placement coordinate metadata;
- parent-relative placement metadata;
- deterministic placement policy metadata;
- placement diagnostics metadata;
- deferred physical rectangle metadata;
- viewport/container relationship metadata.

These are future conceptual categories only.
They are not implemented by this PR.

## 8. Explicit Non-Authority Rules
This boundary does not implement physical placement.
This boundary does not produce final physical layout.
This boundary does not produce backend rectangles.
This boundary does not produce draw commands.
This boundary does not introduce pixel/screen/viewport placement.
This boundary does not introduce backend rendering.
This boundary does not introduce event dispatch.
This boundary does not introduce runtime/verifier/VM integration.
This boundary does not introduce capability admission.
This boundary does not introduce proof/debugger authority.
This boundary does not introduce Workbench/Studio integration.

## 9. Separation From Existing Layers
This boundary is separate from:
- UiLayoutSolvingResultModel;
- UiLayoutConstraintSolverModel;
- UiLayoutSizeToFitModel;
- UiLayoutMeasuringModel;
- backend rendering;
- event dispatch;
- runtime/verifier/VM;
- capability admission;
- Workbench/Studio.

## 10. Deferred Source Gate
Physical placement source remains deferred to a separate explicitly selected source gate.

No source implementation is admitted by this boundary document.

## 11. Forbidden Surface
- physical placement source;
- final physical layout;
- pixel/screen/viewport placement;
- backend rendering;
- event dispatch;
- runtime/verifier/VM integration;
- capability admission;
- proof/debugger authority;
- Workbench/Studio integration.

## 12. Admission Guard
This boundary is docs-only.
This boundary does not implement physical placement.
This boundary does not define source APIs.
This boundary does not add tests.
This boundary does not add backend/rendering/runtime/capability authority.
This boundary does not add Workbench/Studio authority.

## 13. Final Decision
Final decision:
BOUNDARY DEFINED — R12 UI Renderer Layout Physical Placement Boundary is defined as a docs-only future boundary after the audited layout solving result metadata stack.

This boundary does not implement physical placement, final physical layout, pixel/screen/viewport placement, backend rendering, event dispatch, runtime/verifier/VM integration, capability admission, proof/debugger authority, WGPU/winit/Tauri integration, or Workbench/Studio integration.
