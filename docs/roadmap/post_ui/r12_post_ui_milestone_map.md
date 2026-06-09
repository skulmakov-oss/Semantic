# R12 POST-UI Milestone Map

Status: Draft
Track: R12 / POST-UI
Scope type: planning / milestone map
Implementation status: not authorized by this document

## 1. Purpose

This document maps Milestone #25 to the R12 POST-UI planning tracks.

It is not an implementation authorization.

It does not start Workbench or Semantic Studio implementation.

It does not authorize renderer/backend dependency admission.

It does not claim stable, production-ready, public-release-ready, or release-ready status.

## 2. Milestone Anchor

Milestone #25:
POST-UI: Semantic UI Application Boundary

Milestone metadata re-read in this pass:

- state: open
- open issues: 0
- closed issues: 0
- description summary: this milestone remains open and is not started in the Semantic-program sense; it is about a future Semantic-source UI boundary, not Workbench tooling or product UI

Milestone #25 is the planning anchor for Semantic UI Application Boundary work.

It is separate from Workbench product UI.

It is separate from Semantic Studio implementation.

It must remain aligned with Semantic UI DNA.

It must remain verifier-first and boundary-first in future planning.

## 3. Relationship To R12 Roadmap

See:

- [`docs/roadmap/post_ui/r12_ui_roadmap.md`](./r12_ui_roadmap.md)

R12 roadmap places this milestone map before pause guard, Studio anchor map, Workbench separation check, model seed plan, and model seed.

This map does not unblock model seed by itself.

This map is one required planning gate before future model seed.

## 4. POST-UI Boundary Definition

POST-UI Semantic UI Application Boundary means:

- Semantic-owned UI model boundary
- not Workbench UI implementation
- not Semantic Studio implementation
- not renderer/backend ownership
- not browser/WebView ownership
- not widget framework adoption

Semantic UI owns:

- UI Tree
- UI AST
- UI IR
- state/update/event model
- capability/effect discipline
- diagnostics/fault model
- renderer adapter contract

## 5. Track Mapping

| Track | Milestone role | Owner layer | Current status | Allowed work | Forbidden work | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| Semantic UI DNA / Doctrine | doctrine anchor for POST-UI boundary | Semantic UI architecture | present and authoritative | docs-only doctrine alignment | implementation claims | [`docs/dna/SEMANTIC_UI_DNA.md`](../../dna/SEMANTIC_UI_DNA.md) |
| UI Ownership / Boundary Maps | owner split and must-not-own boundaries | POST-UI boundary map | present and authoritative | docs-only boundary mapping | authority widening | [`docs/architecture/ui_ownership_map.md`](../../architecture/ui_ownership_map.md) |
| UI Specs / Admission Contracts | admitted UI surface and verifier-admission planning | spec layer | present as docs/spec anchors | docs/spec mapping and consistency checks | verifier/runtime behavior changes | [`docs/spec/ui/*`](../../spec/ui/) and [`docs/spec/ui_abi_capability_admission.md`](../../spec/ui_abi_capability_admission.md) |
| prom-ui Contract Layer | future inert model planning | UI boundary types | scaffold present | future inert model planning only | renderer/event loop/widget framework | [`crates/prom-ui`](../../../crates/prom-ui) |
| prom-ui-runtime Boundary Layer | runtime boundary planning | UI runtime boundary | scaffold present | future runtime boundary planning | hidden renderer ownership or host lifecycle takeover | [`crates/prom-ui-runtime`](../../../crates/prom-ui-runtime) |
| prom-ui-backend-native Adapter Layer | adapter planning | native backend adapter boundary | scaffold present | optional adapter planning | Semantic UI model ownership | [`crates/prom-ui-backend-native`](../../../crates/prom-ui-backend-native) |
| Workbench Separation | governance while `#675` is active | Workbench tooling/presentation | paused / guarded | audit/docs only while `#675` active | Workbench implementation widening | [`apps/workbench`](../../../apps/workbench) and `#675` |
| Semantic Studio Future Anchor | future control-environment planning | Semantic Studio planning | future anchor only | planning docs only | Studio implementation | `#595` |
| Legal / Dependency Posture | dependency and influence inventory | legal/control docs | dependency register updated; license verification pending | inventory / license verification planning | dependency admission by implication | [`docs/legal/third_party_dependencies.md`](../../legal/third_party_dependencies.md) and [`docs/legal/third_party_influence.md`](../../legal/third_party_influence.md) |

## 6. Milestone Work Buckets

### Bucket A — Doctrine and Ownership

- maintain Semantic UI DNA
- maintain ownership map
- maintain non-adoption boundaries

### Bucket B — Specs and Admission

- UI ABI capability admission
- UI contract map
- verifier admission metadata
- event/effect envelopes

### Bucket C — Runtime / Adapter Boundary Planning

- prom-ui-runtime contract planning
- adapter boundary planning
- no backend ownership of model

### Bucket D — Workbench / Studio Governance

- `#675` pause guard
- Workbench separation check
- `#595` Studio anchor map

### Bucket E — Future Model Seed Preparation

- model seed plan
- inert prom-ui-local model candidates
- no code until docs gates are closed

## 7. Entry Criteria For Future Milestone Code Work

Before code work under Milestone #25:

- R12 roadmap merged
- this milestone map merged
- `#675` pause guard documented
- Workbench separation checked
- Studio anchor mapped
- model seed plan merged
- dependency register updated
- `PRReady` required
- no `FullPreflight` unless explicitly scoped later

## 8. Non-Goals

- no UI implementation
- no Workbench implementation
- no Semantic Studio implementation
- no renderer/backend dependency admission
- no browser/WebView ownership
- no widget framework
- no release widening
- no stable / production-ready / public-release-ready claim
- no final API commitment
- no compiler/verifier/VM/runtime change
- no dependency addition
- no final legal clearance claim

## 9. Follow-Up Items

- `R12-WORKBENCH-STUDIO-PAUSE-GUARD`
- `R12-STUDIO-00-ANCHOR-MAP`
- `R12-WORKBENCH-SEPARATION-CHECK`
- `R12-UI-MODEL-SEED-PLAN`
- `R12-UI-MODEL-SEED` only after docs gates

## 10. Final Decision

Final decision:

READY — CONTINUE WITH DOCS-ONLY POST-UI GOVERNANCE MAPPING
