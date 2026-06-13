# POST-UI Roadmap Next Lane Selection After Layout Inspection Presentation

## 1. Purpose

This document selects the next POST-UI roadmap lane after the completed and audited R12 UI Renderer Layout Inspection Presentation line.

## 2. DNA Alignment

docs/dna inspected: YES
DNA files inspected: SEMANTIC_UI_DNA.md
DNA conflicts detected: NONE
DNA-driven constraints applied:
- renderer/UI remains downstream;
- layout inspection presentation remains read-only observability;
- geometry boundary must be docs-only;
- no geometry implementation in this PR;
- no coordinates/sizing/constraints/solver;
- no draw/event/backend authority;
- no runtime/verifier/VM/capability authority;
- no Workbench/Studio integration.

## 3. Closed Layout Basis

The layout inspection presentation line required recovery because #978 needed follow-up test fixes in #979 and #980 before final green state.

Accepted corrected layout inspection lineage:

#977 — roadmap selected layout inspection presentation
#978 — initial layout inspection presentation source
#979 — recovery test fix 1
#980 — recovery test fix 2 / final green state
#981 — original layout inspection presentation closeout
#982 — corrective recovery closeout
#983 — layout inspection presentation ledger audit after recovery correction

## 4. Project #2 State

| Item | Status | Track | Wave | Type | Risk | Boundary | Gate | Evidence | Depends on |
|---|---|---|---|---|---|---|---|---|---|
| #977 | Done | POST-UI | R12 | Roadmap | Medium | Renderer | Planning-only | Roadmap doc | #976 |
| #978 | Done | POST-UI | R12 | Code | High | Renderer | PRReady | PR | #977 |
| #979 | Done | POST-UI | R12 | Test | High | Renderer | PRReady | PR | #978 |
| #980 | Done | POST-UI | R12 | Test | High | Renderer | PRReady | PR | #979 |
| #981 | Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | #980 |
| #982 | Done | POST-UI | R12 | Closeout | Medium | Renderer | Release Artifact | Roadmap doc | #981 |
| #983 | Done | POST-UI | R12 | Audit | Medium | Renderer | FullPreflight | Roadmap doc | #982 |

## 5. Candidate Lanes

| Candidate | Classification | Reason | Risk | Decision |
|---|---|---|---|---|
| Layout Geometry Boundary | Selected | Safest next step to define geometry authority limits | Medium | Selected |
| Layout Geometry Seed | Deferred / not yet | Geometry source must wait for boundary | High | Deferred |
| Layout Inspection Public API Lock | Deferred | Risk is currently geometry authority, not deeper layout expansion | Low | Deferred |
| Layout Seed Expansion | Deferred | Wait until geometry boundary is explicit | High | Deferred |
| Event Boundary | Deferred / high-risk | Events require geometry to be defined first | High | Deferred |
| Backend Boundary | Deferred / too early | Backend remains outside substrate | High | Deferred |
| Workbench / Studio Boundary | Deferred / not yet | Requires more mature substrate | High | Deferred |
| Full Layout Consolidation Audit | Deferred | Local audits exist; geometry boundary is next risk control | Low | Deferred |

## 6. Selection Criteria

1. Must preserve DNA alignment.
2. Must preserve layout inspection read-only observability.
3. Must not implement geometry in this roadmap PR.
4. Must not introduce coordinates, sizing, constraints, solver logic, draw, event, backend, runtime, capability, or Workbench/Studio authority.
5. Must define geometry boundaries before geometry source exists.
6. Must provide a safe next docs-only lane focused on risk containment.
7. Must explicitly account for the #978/#979/#980 recovery lineage and #982 corrective closeout.
8. Must be documentable and auditable before source/test work.

## 7. Selected Next Lane

Selected next lane:
R12-UI-RENDERER-LAYOUT-GEOMETRY-BOUNDARY-LINE-FULL-PACKAGE

## 8. Deferred Lanes

- Layout Geometry Seed
- Layout Inspection Public API Lock
- Layout Seed Expansion
- Event Boundary
- Backend Boundary
- Workbench / Studio Boundary
- Full Layout Consolidation Audit

## 9. Admission Guard

This selection is planning-only.
This selection does not implement layout geometry.
This selection does not modify layout.rs.
This selection does not add tests.
This selection does not introduce coordinates, sizing, constraints, or solver behavior.
This selection only authorizes the next docs-only geometry boundary package to be prepared under a separate gate.

## 10. Non-Scope

Allowed future boundary scope:
- docs-only boundary;
- define position in pipeline after UiLayoutModel / layout inspection presentation;
- define allowed geometry inputs;
- define forbidden geometry outputs;
- define non-authority rules;
- define geometry admission guard;
- define future seed prerequisites;
- no source code;
- no tests;
- no coordinates implementation;
- no solver implementation.

Forbidden future boundary scope:
- no geometry structs in source;
- no coordinates/sizing implementation;
- no constraints;
- no solver;
- no layout engine;
- no draw commands;
- no event dispatch;
- no backend rendering;
- no runtime/verifier/VM calls;
- no capability admission;
- no Workbench/Studio integration.

## 11. Final Decision

Final decision:
PASS — POST-UI next lane selected after layout inspection presentation.

The next selected lane is R12-UI-RENDERER-LAYOUT-GEOMETRY-BOUNDARY-LINE-FULL-PACKAGE.

This selection is planning-only and does not implement layout geometry, coordinates, sizing, constraints, solver logic, draw commands, event dispatch, backend rendering, runtime/verifier/VM integration, capability admission, or Workbench/Studio integration.
