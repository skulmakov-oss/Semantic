# R12 Workbench / Semantic Studio Pause Guard

Status: Draft
Track: R12 / Governance / POST-UI
Scope type: planning / governance guard
Implementation status: not authorized by this document

## 1. Purpose

This document records the governance implications of Issue #675.

It defines what remains paused and what is allowed under planning, audit, and docs work.

It does not authorize Workbench implementation.

It does not authorize Semantic Studio implementation.

It does not authorize UI model code by itself.

It does not claim readiness, stability, release readiness, or production readiness.

## 2. Governance Anchor

Issue #675:
DIR-UI-PAUSE — pause Workbench and Semantic Studio until project readiness

Issue #675 is the active pause guard for Workbench and Semantic Studio implementation.

The pause remains active unless explicitly closed or superseded by a later governance decision.

This document does not close `#675`.

This document does not weaken `#675`.

This document makes `#675` operational for R12 planning.

Issue metadata re-read in this pass:

- state: open
- labels: governance, scope-control
- URL: <https://github.com/skulmakov-oss/Semantic/issues/675>
- short summary: Workbench and Semantic Studio are officially paused until the project reaches sufficient readiness; only minimal planning preservation is allowed while implementation remains blocked

## 3. What Is Paused

Paused:

- Workbench product expansion beyond existing presentation / orchestration boundaries
- Workbench semantic authority expansion
- Workbench release authority expansion
- Workbench becoming Semantic UI owner
- Semantic Studio implementation
- Semantic Studio product shell implementation
- Semantic Studio as unified control environment implementation
- any UI implementation that silently bypasses POST-UI docs gates
- any renderer/backend ownership expansion under Workbench or Studio naming
- any claim that UI is stable, production-ready, release-ready, or public-release-ready

## 4. What Remains Allowed

Allowed under `#675`:

- docs-only planning
- audit documents
- ownership maps
- roadmap documents
- milestone maps
- dependency / influence registers
- pause guard documents
- Workbench separation checks
- Semantic Studio anchor mapping
- inert model seed planning
- future inert prom-ui-local code only after docs gates are closed and explicitly authorized

Allowed work must remain bounded.

Allowed work must not imply implementation approval.

Allowed work must not imply readiness.

## 5. Boundary Effects

| Area | Current posture | Allowed work | Forbidden work | Required evidence before unpause |
| --- | --- | --- | --- | --- |
| Workbench | presentation / orchestration / tooling surface | docs / audit / separation checks | semantic authority, release authority, UI model ownership | readiness decision, separation proof, command/effect boundary audit |
| Semantic Studio | future planning anchor | anchor map and product boundary docs | implementation / product shell / control environment launch | Studio boundary document, readiness decision, Workbench separation proof |
| Semantic UI / POST-UI | architecture boundary track | doctrine / spec / model planning | hidden Workbench / Studio implementation | roadmap, milestone map, pause guard, model seed plan |
| Renderer / backend | adapter boundary only | docs-only adapter planning | renderer ownership of Semantic UI model | adapter contract, capability/effect boundary, dependency review |
| Legal / dependency posture | dependency register updated, license pending verification | license verification planning | final legal clearance claim | verified license table, dependency update process |

## 6. Relation To R12 Roadmap

See:

- [`docs/roadmap/post_ui/r12_ui_roadmap.md`](./r12_ui_roadmap.md)
- [`docs/roadmap/post_ui/r12_post_ui_milestone_map.md`](./r12_post_ui_milestone_map.md)

R12 roadmap places this pause guard before Studio anchor map, Workbench separation check, model seed plan, and model seed.

This pause guard is required before model seed planning can be considered complete.

This pause guard does not unblock implementation by itself.

## 7. Unpause Criteria

Minimum criteria before any future Workbench or Semantic Studio implementation unpause:

- explicit governance decision
- `#675` closed, superseded, or explicitly amended
- Workbench separation check completed
- Studio anchor map completed
- R12 roadmap merged
- R12 milestone map merged
- dependency register updated
- license verification status understood
- Local Admission Guard remains authoritative
- release scope explicitly reviewed
- no hidden renderer/backend ownership
- no browser/WebView ownership of Semantic state
- no widget framework ownership of Semantic UI model

GitHub CI pass is not sufficient to unpause.

A PR merge is not sufficient to unpause unless the PR explicitly changes governance status.

## 8. Model Seed Impact

`R12-UI-MODEL-SEED` remains blocked behind docs gates.

This pause guard is one required docs gate.

Model seed, when later authorized, must be inert and prom-ui-local.

Model seed must not touch Workbench.

Model seed must not touch Semantic Studio.

Model seed must not add renderer/backend dependencies.

Model seed must not add parser / lowering / VM / runtime integration.

Model seed must not create widget / layout framework scope.

## 9. Non-Goals

- no UI implementation
- no Workbench implementation
- no Semantic Studio implementation
- no model seed code
- no renderer/backend dependency admission
- no browser/WebView ownership
- no widget framework adoption
- no release widening
- no stable / production-ready / public-release-ready claim
- no final API commitment
- no compiler / verifier / VM / runtime change
- no dependency addition
- no final legal clearance claim
- no closure of `#675`

## 10. Follow-Up Items

- `R12-STUDIO-00-ANCHOR-MAP`
- `R12-WORKBENCH-SEPARATION-CHECK`
- `R12-UI-MODEL-SEED-PLAN`
- `R12-UI-MODEL-SEED` only after docs gates

## 11. Final Decision

Final decision:

READY — CONTINUE WITH DOCS-ONLY STUDIO ANCHOR AND WORKBENCH SEPARATION MAPPING
