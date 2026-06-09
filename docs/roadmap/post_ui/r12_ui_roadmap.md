# R12 Semantic UI Roadmap

Status: Draft
Track: R12 / POST-UI
Scope type: planning / roadmap
Implementation status: not authorized by this document

## 1. Purpose

This document defines the R12 Semantic UI planning roadmap.

It is not an implementation authorization.

It is not a public release readiness claim.

It does not start Workbench or Semantic Studio implementation.

It sequences work needed before future Semantic UI model code.

## 2. Current Evidence Base

- UI file audit completed.
- R12 ownership project seed merged via [`#875`](https://github.com/skulmakov-oss/Semantic/pull/875).
- GitHub Project #2 is configured with canonical R12 items, fields, and views.
- Duplicate retry project items were cleaned.
- Third-party dependency register merged via [`#877`](https://github.com/skulmakov-oss/Semantic/pull/877).
- Semantic UI DNA doctrine exists.
- Issue [`#675`](https://github.com/skulmakov-oss/Semantic/issues/675) pause remains active.
- Milestone [`#25`](https://github.com/skulmakov-oss/Semantic/milestone/25) exists as the POST-UI boundary track.
- Issue [`#595`](https://github.com/skulmakov-oss/Semantic/issues/595) exists as the future Semantic Studio anchor.

## 3. Governance Constraints

- `#675` pauses Workbench and Semantic Studio implementation until explicit readiness.
- Workbench remains a presentation / orchestration / tooling surface.
- Semantic Studio remains a future planning anchor.
- POST-UI Semantic UI boundary work must not silently become Workbench or Studio implementation.
- No stable, production-ready, public-release-ready, or release-ready claim is made.

## 4. Ownership Boundaries

Semantic UI owns:

- UI Tree
- UI AST
- UI IR
- state/update/event model
- capability/effect discipline
- diagnostics/fault model
- renderer adapter contract

Workbench owns:

- tooling shell
- presentation / orchestration
- docs/spec navigator
- local command / report surfaces, within existing boundaries only

Renderer / backend owns:

- adapter implementation only
- no Semantic UI model ownership

External projects:

- inspiration or dependency only as registered
- no silent derivative / fork code

## 5. R12 Track Split

### Track A - Governance / Pause Control

- Owns: `#675` guard.
- No implementation.

### Track B - Legal / Dependency Posture

- Owns: third-party dependency register follow-up.
- License verification remains pending.

### Track C - POST-UI Semantic UI Boundary

- Owns: future Semantic UI model planning.
- Maps Milestone `#25`.

### Track D - Workbench Separation

- Ensures Workbench remains tooling / presentation only.

### Track E - Future Semantic Studio

- Maps `#595`.
- Planning-only until readiness.

## 6. Ordered Work Plan

| Order | Item | Project item | Type | Risk | Boundary | Gate | Depends on | Allowed output | Explicit non-goals |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | R12-UI-PROJECT-SETUP | completed | Setup | Low | Workbench | Planning-only | R12-UI-OWNERSHIP-PROJECT-SEED | Project board configured | no implementation |
| 2 | R12-THIRD-PARTY-DEPENDENCY-REGISTER | completed / merged via `#877` | Docs | Medium | Legal | Docs-only | R12-UI-FILE-AUDIT closeout | Dependency register inventory; license pending verification | no ownership widening |
| 3 | R12-POST-UI-MILESTONE-MAP | planned | Docs | Medium | Semantic UI | Docs-only | R12-UI-PROJECT-SETUP, Milestone `#25` | docs-only mapping of Milestone `#25` to POST-UI tracks | no release widening |
| 4 | R12-WORKBENCH-STUDIO-PAUSE-GUARD | planned | Governance | High | Workbench | Planning-only | `#675` | docs-only confirmation of `#675` pause implications | no UI implementation |
| 5 | R12-STUDIO-00-ANCHOR-MAP | planned | Docs | Medium | Semantic Studio | Planning-only | `#595` | docs-only mapping of `#595` as future Studio anchor | no Studio implementation |
| 6 | R12-WORKBENCH-SEPARATION-CHECK | planned | Audit | Medium | Workbench | Planning-only | R12-UI-PROJECT-SETUP | docs-only check that Workbench remains presentation / orchestration | no authority shift |
| 7 | R12-UI-MODEL-SEED-PLAN | planned | Docs | High | Semantic UI | Planning-only | R12-POST-UI-MILESTONE-MAP, R12-WORKBENCH-STUDIO-PAUSE-GUARD, R12-WORKBENCH-SEPARATION-CHECK, R12-THIRD-PARTY-DEPENDENCY-REGISTER | docs-only model seed plan, not code | no renderer/backend dependency admission |
| 8 | R12-UI-MODEL-SEED | future | Code / Docs | High | Semantic UI | PRReady | R12-UI-MODEL-SEED-PLAN | future minimal code seed only after prior docs gates are closed | no Workbench / Semantic Studio implementation |

R12-UI-MODEL-SEED is blocked until the roadmap, pause guard, and model seed plan are complete.

R12-UI-MODEL-SEED must remain prom-ui-local and inert.

No renderer/backend dependency admission is allowed in the model seed.

No Workbench or Semantic Studio implementation is allowed in the model seed.

## 7. Model Seed Entry Criteria

Before any `R12-UI-MODEL-SEED` code PR, require:

- roadmap merged
- `#675` pause implications documented
- Milestone `#25` map documented
- Workbench separation checked
- dependency register updated
- model seed plan written
- `PRReady` required
- no `FullPreflight` unless explicitly scoped later

## 8. Expected Future Model Shape

This section is planning-only.

Possible future inert types:

- `UiNodeId`
- `UiTreeId`
- `UiNodeKind`
- `UiNode`
- `UiTree`
- `UiAstNode`
- `UiAst`
- `UiIrNode`
- `UiIr`

These names are planning candidates, not final API commitments.

No parser integration.

No lowering integration.

No VM integration.

No renderer dependency.

No event loop.

No widget / layout framework.

No external UI dependency.

## 9. Dependency / Influence Posture

Influence register and dependency register are separate.

The dependency register now inventories actual manifest dependencies.

License verification remains pending.

No derivative / fork evidence was found.

React, Tauri, and `winit` do not own the Semantic UI model.

External projects remain inspiration or dependency, not owners.

## 10. Release / Readiness Non-Claims

- no public release widening
- no stable claim
- no production-ready claim
- no Semantic Studio readiness claim
- no Workbench readiness claim
- no final legal clearance claim
- no renderer/backend readiness claim

## 11. Roadmap Exit Criteria

This roadmap can be considered ready for the next planning step when:

- it is merged
- Project item `R12-UI-ROADMAP-DRAFT` is marked complete manually or by later task
- `R12-POST-UI-MILESTONE-MAP` is next or explicitly deferred
- no implementation has been started by this roadmap

## 12. Final Decision

Final decision:

READY — CONTINUE WITH DOCS-ONLY POST-UI PLANNING BEFORE MODEL SEED
