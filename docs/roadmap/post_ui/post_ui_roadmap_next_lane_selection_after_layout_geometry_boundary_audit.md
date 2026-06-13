# POST-UI Roadmap Next Lane Selection After Layout Geometry Boundary Audit

## 1. Purpose
This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Geometry Boundary line and the layout seed test hygiene cleanup.

## 2. DNA Alignment
docs/dna inspected: YES
DNA files inspected: SEMANTIC_UI_DNA.md
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- geometry seed may only be inert renderer-local structural metadata;
- geometry seed must not introduce draw/event/backend authority;
- geometry seed must not introduce runtime/verifier/VM/capability authority;
- geometry seed must not introduce Workbench/Studio integration;
- this roadmap selection does not implement geometry source.

## 3. Closed Basis
#984 — roadmap selected geometry boundary
#985 — layout geometry boundary
#986 — layout geometry boundary closeout
#987 — layout geometry boundary ledger audit
#988 — layout seed test hygiene cleanup

## 4. Geometry Boundary State
Geometry boundary is closed as docs-only boundary work. It documents future geometry authority without implementing geometry source, coordinates, sizing, constraints, solver behavior, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.

## 5. Hygiene Cleanup State
The non-blocking unused_mut warning in crates/prom-ui/tests/renderer_layout_seed.rs was removed by #988. The cleanup did not change production source, docs, manifests, dependencies, or architecture.

## 6. Project #2 State
| Item | Status | Track | Wave | Type | Risk | Boundary | Gate | Evidence | Depends on |
|---|---|---|---|---|---|---|---|---|---|
| #984 | Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | #983 |
| #985 | Done | POST-UI | R12 | Docs | High | Renderer | Docs-only | Roadmap doc | #984 |
| #986 | Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | #985 |
| #987 | Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | #986 |
| #988 | Done | POST-UI | R12 | Test | Low | Renderer | PRReady | PR | #987 |

## 7. Candidate Lanes
| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Geometry Seed | Selected | The geometry boundary has been selected, documented, closed out, audited, and the non-blocking layout seed warning has been cleaned. The next structurally valid step is a small source seed for geometry metadata under strict constraints. | Medium | Selected |
| Geometry Solver Boundary | Deferred / too early | A solver boundary is premature until minimal geometry metadata exists. | High | Deferred |
| Constraints Boundary | Deferred | Constraints should wait until geometry seed establishes the minimal shape and reference model. | High | Deferred |
| Event Boundary | Deferred / high-risk | Events remain close to action/effect/capability semantics and should not be introduced before geometry/layout layers are cleanly seeded and audited. | High | Deferred |
| Backend Boundary | Deferred / too early | Backend/WGPU/winit/Tauri remains outside the current scope. | High | Deferred |
| Layout Geometry Consolidation Audit | Deferred | Useful after geometry seed, not before it. | Low | Deferred |

## 8. Selection Criteria
1. Must preserve DNA alignment.
2. Must preserve renderer/layout inertness.
3. Must not implement geometry in the roadmap selection PR.
4. Must not introduce solver/constraints prematurely.
5. Must not introduce draw/event/backend.
6. Must not introduce runtime/verifier/VM/capability authority.
7. Must build naturally on closed geometry boundary and audit.
8. Must be source-gated separately before implementation.

## 9. Selected Next Lane
Selected next lane:
R12-UI-RENDERER-LAYOUT-GEOMETRY-SEED-LINE-FULL-PACKAGE

This selection is planning-only.
This selection does not implement geometry source.
This selection only authorizes the next source package to be prepared under a separate gate.

## 10. Deferred Lanes
- Geometry Solver Boundary
- Constraints Boundary
- Event Boundary
- Backend Boundary
- Layout Geometry Consolidation Audit

## 11. Admission Guard
This selection is planning-only.
This selection does not modify layout.rs.
This selection does not add tests.
This selection does not introduce coordinates, sizing, constraints, or solver behavior.
This selection only authorizes the next source package to be prepared under a separate gate.

## 12. Non-Scope
Allowed future geometry seed scope:
- minimal geometry metadata seed;
- deterministic geometry identity policy, if applicable;
- geometry model as inert renderer-local structural metadata;
- read-only consumption of existing layout/render metadata if already exposed;
- no draw/event/backend/runtime/capability authority;
- focused tests proving inertness and deterministic behavior.

Forbidden future geometry seed scope:
- no full geometry solver;
- no constraint solver;
- no sizing algorithm;
- no layout engine rewrite;
- no draw commands;
- no event handling;
- no backend/WGPU/winit/Tauri;
- no runtime/verifier/VM calls;
- no capability admission;
- no Workbench/Studio integration.

## 13. Final Decision
Final decision:
PASS — POST-UI next lane selected after layout geometry boundary audit.

The next selected lane is R12-UI-RENDERER-LAYOUT-GEOMETRY-SEED-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement geometry source, coordinates, sizing, constraints, solver behavior, draw/event/backend systems, runtime/verifier/VM integration, capability admission, proof/debugger authority, or Workbench/Studio integration.
