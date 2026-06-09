# R12 UI Ownership Project Seed

Status: Draft
Track: R12 / POST-UI
Purpose: planning seed for ownership, project, and roadmap setup
Scope type: documentation only

## 1. Status

This is a planning seed.

It is not an implementation roadmap yet.

It does not authorize Workbench implementation.

It does not authorize Semantic Studio implementation.

It does not widen public release scope.

It exists to connect current doctrine, governance blockers, code surfaces, and candidate project items before any implementation work begins.

## 2. Governance Anchors

### Issue #675 - DIR-UI-PAUSE

- Role: governance / scope-control pause for Workbench and Semantic Studio.
- Planning implication: UI implementation must remain paused until explicit readiness review.
- Implementation restriction: no Workbench or Semantic Studio implementation while this directive remains active.

### Milestone #25 - POST-UI: Semantic UI Application Boundary

- Role: open milestone for the future Semantic UI application boundary track.
- Planning implication: POST-UI UI work should be organized as a separate boundary track, not merged into the published baseline.
- Implementation restriction: this milestone does not itself authorize implementation or release widening.

### Issue #595 - STUDIO-00

- Role: future control-environment anchor for Semantic Studio.
- Planning implication: Semantic Studio is a later planning target that must stay foundation-first and verifier-first.
- Implementation restriction: the issue is architectural and planning-only; it does not authorize immediate UI implementation.

## 3. Doctrine Anchors

### `docs/dna/SEMANTIC_UI_DNA.md`

- Role: architecture doctrine.
- Owner: Semantic UI architecture.
- Key boundary rule: Semantic UI owns its own UI Tree, UI AST, UI IR, state/update/event model, capability/effect discipline, diagnostics and fault model, and renderer adapter contract.
- What it does not authorize: browser/DOM ownership, WebView ownership, widget framework adoption, renderer ownership by third-party UI systems, or release widening.

### `docs/architecture/ui_ownership_map.md`

- Role: ownership and boundary map.
- Owner: POST-UI UI ownership layer.
- Key boundary rule: `prom-ui`, `prom-ui-runtime`, platform backend, demo, and Workbench are split into separate owner roles with explicit must-not-own boundaries.
- What it does not authorize: compiler, VM, parser, typechecker, or hidden runtime authority in the UI layer.

### `docs/roadmap/language_maturity/ui_application_boundary_scope.md`

- Role: completed first-wave scope reading for the UI application boundary.
- Owner: POST-UI first-wave owner split.
- Key boundary rule: first-wave runtime ownership stays narrow and explicit, and release-facing docs keep widened main distinct from published `v1.1.1`.
- What it does not authorize: silent widening of the published baseline or generalized widget/layout ownership.

## 4. Code Surface Map

| Surface | Files / crate | Current owner | Current role | Current status | Must not own | Planning implication |
| --- | --- | --- | --- | --- | --- | --- |
| prom-ui | `crates/prom-ui` | Semantic UI boundary types | Capability taxonomy and admitted UI operation identity | Contract-heavy scaffold | widget/layout framework, VM policy, renderer ownership | Seed future UI model contracts without runtime widening |
| prom-ui-runtime | `crates/prom-ui-runtime` | UI runtime boundary | Session lifecycle, event polling, frame token, adapter seam | Runtime scaffold with tests | compiler, parser, verifier, Workbench UI, platform backend ownership outside seam | Treat as runtime boundary owner, not compiler owner |
| prom-ui-backend-native | `crates/prom-ui-backend-native` | Native backend boundary | Feature-gated platform backend skeleton | Skeleton with `winit` placeholder | renderer authority, UI model ownership, compiler ownership | Plan adapter/back-end work separately from model and runtime docs |
| prom-ui-demo | `crates/prom-ui-demo` | Demo consumer | Canonical consumer demo over `NullBackend` | Reference/demo app | semantic ownership, backend authority, release authority | Keep as consumer proof surface only |
| Workbench src | `apps/workbench/src` | Workbench presentation layer | Orchestration shell, navigation, local presentation state | Live UI shell | parser, typechecker, verifier, VM, canonical source truth | Keep Workbench as presentation/orchestration only |
| Workbench src-tauri | `apps/workbench/src-tauri` | Workbench bridge layer | Docs catalog, command bridge, workspace I/O, report export | Live Tauri bridge | semantic authority, renderer authority, release authority | Use only as tooling surface and read-only catalog bridge |
| Legal / third-party registers | `docs/legal/third_party_influence.md`, `docs/legal/third_party_dependencies.md` | Legal/influence tracking | Separate inspiration from dependency tracking | Influence register present; dependency register placeholder | runtime ownership, semantic model ownership | Populate actual dependencies before making wider UI claims |
| Future UiTree / UiAst / UiIr | doctrine-defined future model | Semantic UI architecture | Future core model for UI semantics | Not present in code yet | Workbench ownership, foreign UI lifecycle ownership | Seed as future model track, not implementation track |

## 5. Track Split

### Track A - Governance / Pause Control

- Covers: issue #675.
- Intent: keep Workbench and Semantic Studio paused until readiness.
- Implementation: none.

### Track B - POST-UI Semantic UI Boundary

- Covers: milestone #25.
- Intent: grow the admitted Semantic UI boundary through `prom-ui`, `prom-ui-runtime`, and backend-adapter tracks.
- Implementation: separate from Workbench and separate from published baseline claims.

### Track C - Future Semantic Studio

- Covers: issue #595.
- Intent: define the future control environment after foundation readiness.
- Implementation: planning-only until the foundation is explicitly ready.

## 6. Dependency / Influence Register Gap

`docs/legal/third_party_influence.md` exists.

`docs/legal/third_party_dependencies.md` exists, but it is currently a placeholder and does not yet enumerate the actual admitted dependency posture for the UI-adjacent code surfaces.

Actual dependencies from Workbench, Tauri, React, and optional `winit` must be registered before any wider UI claims are made.

Inspiration, dependency, and derivative must remain separate categories.

## 7. Candidate GitHub Project Items

| Title | Track | Type | Risk | Boundary | Evidence | Depends on | First allowed PR | Non-goals |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R12-UI-FILE-AUDIT closeout | R12 | Closeout | Low | Audit summary | Current audit across docs and code surfaces | none | none; project closeout only | no implementation |
| R12-UI-PROJECT-SETUP | R12 | Audit / Setup | Low | Project planning | #675, #25, #595, UI doctrine, ownership map | this seed | docs-only planning update if needed | no implementation |
| R12-UI-ROADMAP-DRAFT | R12 | Docs | Medium | POST-UI roadmap | Ownership map, milestone #25, issue #595 | project setup | docs/roadmap update only | no implementation |
| R12-THIRD-PARTY-DEPENDENCY-REGISTER | R12 | Docs | Medium | legal / dependency tracking | placeholder dependency register and actual workspace deps | audit of package manifests | docs/legal update only | no ownership widening |
| R12-WORKBENCH-STUDIO-PAUSE-GUARD | R12 | Governance | High | Workbench / Studio pause | issue #675 | governance decision | none; issue/guardrail only | no UI implementation |
| R12-POST-UI-MILESTONE-MAP | R12 | Docs | Medium | milestone mapping | milestone #25 and POST-UI docs | project setup | docs-only mapping update | no release widening |
| R12-UI-MODEL-SEED | R12 | Code / Docs | High | future UiTree / UiAst / UiIr model | doctrine anchor and missing code model | roadmap draft and governance clarity | future docs-only or code seed PR after readiness | no renderer or Workbench ownership |
| R12-WORKBENCH-SEPARATION-CHECK | R12 | Audit | Medium | Workbench separation | Workbench source shell and docs navigator behavior | current code audit | docs-only check/update if needed | no authority shift |
| R12-STUDIO-00-ANCHOR-MAP | R12 | Docs | Medium | Semantic Studio planning | issue #595 and doctrine anchors | project setup | docs-only planning update | no Studio implementation |

## 8. Recommended Ordering

1. Project setup and item registration.
2. Dependency register update.
3. R12 UI roadmap draft.
4. Only then R12-UI-MODEL-SEED.
5. No Workbench or Semantic Studio implementation while #675 remains active.

## 9. Explicit Non-Goals

- no UI implementation
- no Workbench implementation
- no Semantic Studio implementation
- no renderer dependency admission
- no browser ownership
- no WebView ownership
- no widget framework scope
- no release widening
- no stable or production-ready claims

## 10. Final Decision

Final decision:

READY — CREATE PROJECT ITEMS BEFORE ROADMAP OR IMPLEMENTATION
