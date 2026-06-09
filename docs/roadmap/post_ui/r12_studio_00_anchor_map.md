# R12 Studio-00 Anchor Map

Status: Draft
Track: R12 / Semantic Studio / POST-UI
Scope type: planning / anchor map
Implementation status: not authorized by this document

## 1. Purpose

This document maps Issue #595 as the future Semantic Studio planning anchor.

It is not an implementation authorization.

It does not start Semantic Studio implementation.

It does not start Workbench implementation.

It does not authorize a product shell, unified control environment, or runtime control surface.

It does not claim readiness, stability, release readiness, or production readiness.

## 2. Studio Anchor

Issue #595:
STUDIO-00 — define Semantic Studio as the unified Semantic control environment

Issue #595 is a future planning anchor.

`#595` does not override `#675`.

`#595` does not authorize implementation while `#675` remains active.

Semantic Studio remains future-facing until explicit readiness.

This document does not close, amend, or weaken `#675`.

This document does not implement Semantic Studio.

Issue metadata re-read in this pass:

- state: open
- labels: none
- URL: <https://github.com/skulmakov-oss/Semantic/issues/595>
- short summary: Semantic Studio is a future unified control environment; the issue is intentionally architectural and planning-only and should be implemented only after the foundation is stable enough

## 3. Relationship To `#675` Pause Guard

See:

- [`docs/roadmap/post_ui/r12_workbench_studio_pause_guard.md`](./r12_workbench_studio_pause_guard.md)

`#675` is stronger than `#595` for current implementation decisions.

`#595` may define future Studio intent, but `#675` blocks implementation.

Any future Studio implementation requires explicit unpause governance.

GitHub CI is not enough to unblock Studio.

A PR merge is not enough to unblock Studio unless governance status is explicitly changed.

This anchor map is docs-only and does not change governance state.

## 4. Relationship To R12 Roadmap

See:

- [`docs/roadmap/post_ui/r12_ui_roadmap.md`](./r12_ui_roadmap.md)
- [`docs/roadmap/post_ui/r12_post_ui_milestone_map.md`](./r12_post_ui_milestone_map.md)

R12 roadmap places Studio anchor map before Workbench separation check, model seed plan, and model seed.

This Studio anchor map is one docs gate.

This map does not unblock model seed by itself.

This map does not unblock Semantic Studio implementation.

## 5. Semantic Studio Future Role

Potential future role:

- unified Semantic control environment
- project navigation shell
- verifier / admission visibility surface
- diagnostics and reports surface
- documentation and spec navigation surface
- Workbench-like lower-level console may be a mode or component later

These are future planning categories, not active implementation commitments.

Studio must not own Semantic UI model.

Studio must not own compiler, verifier, VM, or runtime.

Studio must not own release authority by default.

Studio must not bypass Local Admission Guard.

Studio must not become hidden UI implementation under planning labels.

## 6. Studio / Workbench Boundary

| Area | Workbench posture | Studio posture | Allowed now | Forbidden now | Future evidence needed |
| --- | --- | --- | --- | --- | --- |
| Tooling shell | existing presentation / orchestration shell | future unified control environment concept | docs / audit / planning | Studio shell implementation | `#675` unpause, Studio boundary doc |
| Docs/spec navigation | existing read-only navigator surface | future integrated documentation surface | docs-only mapping | new Studio docs UI implementation | Workbench separation proof, Studio scope doc |
| Command/report surfaces | existing local command / report boundaries | future control / visibility surface | audit and boundary docs | expanding command authority | command/effect boundary audit |
| Semantic UI model | not owner | not owner | Semantic UI model planning under POST-UI docs gates | Studio or Workbench taking ownership | model seed plan, ownership map |
| Release/readiness authority | not authority | not authority by default | docs-only readiness mapping | release gating claims | explicit release governance decision |

## 7. Studio Non-Ownership Rules

Semantic Studio does not own:

- UI Tree
- UI AST
- UI IR
- Semantic UI state/update/event model
- capability/effect discipline
- diagnostics/fault model
- renderer adapter contract
- compiler
- verifier
- VM
- runtime
- release authority
- Local Admission Guard

## 8. Future Studio Entry Criteria

Before any future Semantic Studio implementation PR:

- `#675` must be closed, superseded, or explicitly amended.
- Studio boundary document must be merged.
- Workbench separation check must be completed.
- command/effect boundary audit must be completed.
- dependency register must be updated.
- license verification status must be understood.
- release scope must be explicitly reviewed.
- Local Admission Guard must remain authoritative.
- no hidden renderer/backend ownership.
- no browser/WebView ownership of Semantic state.
- no widget framework ownership of Semantic UI model.

This document does not satisfy those criteria by itself.

## 9. Model Seed Impact

`R12-UI-MODEL-SEED` remains blocked behind docs gates.

Studio anchor map is one required docs gate.

Model seed must not touch Semantic Studio.

Model seed must not touch Workbench.

Model seed must remain prom-ui-local and inert when later authorized.

Model seed must not add renderer/backend dependencies.

Model seed must not add parser / lowering / VM / runtime integration.

Model seed must not create widget / layout framework scope.

## 10. Non-Goals

- no Semantic Studio implementation
- no Workbench implementation
- no UI implementation
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
- no closure or weakening of `#675`

## 11. Follow-Up Items

- `R12-WORKBENCH-SEPARATION-CHECK`
- `R12-UI-MODEL-SEED-PLAN`
- `R12-UI-MODEL-SEED` only after docs gates

## 12. Final Decision

Final decision:

READY — CONTINUE WITH DOCS-ONLY WORKBENCH SEPARATION MAPPING
