# POST-UI Roadmap Next Lane Selection

## 1. Purpose

This document selects the next POST-UI roadmap lane after the completed R12 UI Projection Builder line.

It does not implement the selected lane.
It only records the next authorized roadmap direction.

## 2. Closed Basis

Closed basis:
#941 — R12 UI Projection Builder Final Closeout — MERGED

R12 Projection Builder final state:
- projection substrate complete;
- renderer absent;
- layout/draw/event absent;
- runtime/verifier/VM absent;
- capability admission absent;
- Workbench/Studio absent;
- public unchecked projection absent.

## 3. Current State After R12 Projection Builder

The R12 Projection Builder effectively secured projection determinism.
UI projection is now known to be inert, decoupled from renderer semantics, decoupled from event logic, and safely locked under robust diagnostics and traceability guarantees. 

## 4. Candidate Next Lanes

| Candidate lane | Description | Risk | Decision |
|---|---|---:|---|
| R12-UI-RENDERER-BOUNDARY-LINE-FULL-PACKAGE | Define renderer boundary downstream of projection substrate | High | SELECTED |
| R12-UI-LAYOUT-BOUNDARY-LINE-FULL-PACKAGE | Define layout boundary | Medium | DEFERRED |
| R12-UI-EVENT-BOUNDARY-LINE-FULL-PACKAGE | Define event boundary | High | DEFERRED |
| R12-UI-WORKBENCH-INTEGRATION-BOUNDARY-LINE-FULL-PACKAGE | Define Workbench integration boundary | High | DEFERRED |
| R12-UI-RUNTIME-BRIDGE-BOUNDARY-LINE-FULL-PACKAGE | Define runtime bridge boundary | Critical | DEFERRED |

## 5. Selection Criteria

- must follow projection substrate;
- must not require runtime or verifier integration;
- must not imply UI authority over truth;
- must preserve inert projection semantics;
- must define downstream consumer boundary before implementation;
- must reduce risk before layout/event/Workbench work.

## 6. Selected Next Lane

SELECTED NEXT LANE:
R12-UI-RENDERER-BOUNDARY-LINE-FULL-PACKAGE

This selection authorizes only a future renderer boundary package.

It does not authorize renderer implementation.
It does not authorize layout/draw/event implementation.
It does not authorize runtime/verifier/capability integration.

## 7. Why Renderer Boundary Comes Next

Renderer is the first downstream consumer of UiProjectionArtifact.

Before renderer implementation, the project must define:
- what renderer may read from projection artifacts;
- what renderer must not infer;
- how renderer treats PropertyCarrier / ActionCarrier / EffectBoundaryMarker as inert;
- how renderer handles diagnostics and trace references;
- how renderer avoids becoming semantic authority;
- how renderer remains separate from runtime, verifier, capability admission, Workbench, and Studio.

## 8. Explicit Non-Scope

No renderer implementation.
No backend implementation.
No WGPU/winit/Tauri.
No layout engine.
No draw engine.
No event loop.
No event dispatch.
No runtime integration.
No verifier integration.
No VM integration.
No capability admission.
No Workbench/Studio integration.
No source changes.
No dependency additions.

## 9. Admission Guard

| Area | State | Admission Guard classification | Status |
|---|---|---|---|
| next lane selection doc | Present | ADMITTED | PASS |
| R12 Projection Builder | Closed | CLOSED | PASS |
| renderer boundary | Selected for future package | AUTHORIZED_FOR_FUTURE | PASS |
| renderer implementation | Absent | FORBIDDEN | PASS |
| layout/draw/event implementation | Absent | FORBIDDEN | PASS |
| runtime/verifier/VM integration | Absent | FORBIDDEN | PASS |
| capability admission | Absent | FORBIDDEN | PASS |
| Workbench/Studio integration | Absent | FORBIDDEN | PASS |
| source changes | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## 10. Project #2 Tracking

This selection directly drives Project #2 updates ensuring downstream workflows trace correctly back to projection closure.

## 11. Final Decision

Final decision:
SELECTED — The next POST-UI roadmap lane is R12-UI-RENDERER-BOUNDARY-LINE-FULL-PACKAGE.

This selection is based on the completed R12 UI Projection Builder substrate and authorizes only the next renderer boundary package. It does not authorize renderer implementation, backend integration, layout/draw/event, runtime/verifier/VM integration, capability admission, or Workbench/Studio integration.
