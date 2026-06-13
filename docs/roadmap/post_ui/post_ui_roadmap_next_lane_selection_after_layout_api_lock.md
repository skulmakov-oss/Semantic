# POST-UI Roadmap Next Lane Selection After Layout Public API Lock

## 1. Purpose

This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Public API Lock line.

## 2. DNA Alignment

docs/dna inspected: YES
DNA files inspected:
- docs/dna/SEMANTIC_UI_DNA.md
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout remains inert structural metadata;
- layout inspection presentation must remain read-only;
- no layout behavior expansion;
- no geometry solver;
- no draw/event/backend authority;
- no runtime/verifier/VM/capability authority;
- no Workbench/Studio integration.

## 3. Closed Layout Basis

The layout seed line required recovery because #969 was merged before the final corrected source PR #970.

Accepted corrected layout lineage:

#968 — layout boundary ledger audit
#970 — actual layout seed source implementation
#969 — premature original layout seed closeout
#971 — corrective recovery closeout
#972 — layout seed ledger audit after recovery correction
#973 — roadmap selection after layout seed
#974 — layout public API lock tests
#975 — layout public API lock closeout
#976 — layout public API lock ledger audit after Project #2 backfill correction

## 4. Project #2 State

Project #2 metadata for layout API lock:
- #974 is Done
- #975 is Done
- #976 is Done

## 5. Candidate Lanes

| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Inspection Presentation | Selected | After the layout seed exists and its public API is locked and audited, the safest next step is to add an inert inspection/presentation layer over the layout model. | Low | Proceed |
| Layout Geometry Boundary | Deferred / high-risk | Geometry introduces coordinate and sizing semantics. It should wait until layout inspection/presentation gives observability over layout structure. | High | Deferred |
| Layout Seed Expansion | Deferred | The base layout API is locked. Expansion should wait until a read-only inspection layer exists, so future changes can be audited through presentation metadata. | Medium | Deferred |
| Event Boundary | Deferred / high-risk | Events are close to action/effect/capability semantics and should not proceed while layout remains in structural/presentation phases. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside the current renderer/layout substrate. | High | Deferred |
| Workbench / Studio Boundary | Deferred / not yet | Workbench/Studio is a product-level orchestration layer and must wait until renderer/layout/event/capability boundaries are more mature. | High | Deferred |
| Full Layout Consolidation Audit | Deferred | Useful later, but not the best immediate next step. Layout boundary, seed, and API lock already have local ledger audits. The next useful surface is inert layout inspection. | Low | Deferred |

## 6. Selection Criteria

1. Must preserve DNA alignment.
2. Must preserve layout seed inertness.
3. Must build on the locked public layout API.
4. Must not implement new layout behavior in this roadmap PR.
5. Must not introduce geometry, draw, event, backend, runtime, capability, or Workbench/Studio authority.
6. Must provide a safe next source lane focused on read-only observability.
7. Must explicitly account for the #969/#970 recovery lineage and #974/#975 Project #2 backfill correction.
8. Must be documentable and auditable before source/test work.

## 7. Selected Next Lane

Selected next lane:
R12-UI-RENDERER-LAYOUT-INSPECTION-PRESENTATION-LINE-FULL-PACKAGE

## 8. Deferred Lanes

- Layout Geometry Boundary
- Layout Seed Expansion
- Event Boundary
- Backend Boundary
- Workbench / Studio Boundary
- Full Layout Consolidation Audit

## 9. Admission Guard

This selection is planning-only.
This selection does not implement layout inspection presentation.
This selection does not modify layout.rs.
This selection does not add tests.
This selection only authorizes the next inspection/presentation package to be prepared under a separate gate.

## 10. Non-Scope

- no layout behavior expansion
- no geometry solver
- no coordinates/sizing engine
- no draw commands
- no event handling
- no event dispatch
- no backend/WGPU/winit/Tauri
- no runtime/verifier/VM integration
- no capability admission
- no Workbench/Studio integration

## 11. Final Decision

Final decision:
PASS — POST-UI next lane selected after layout public API lock.

The next selected lane is R12-UI-RENDERER-LAYOUT-INSPECTION-PRESENTATION-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement layout inspection presentation, layout behavior expansion, geometry solving, draw commands, event dispatch, backend rendering, runtime/verifier/VM integration, capability admission, or Workbench/Studio integration.
