# R12 Semantic UI First App Shell Rule

Status: Draft
Track: R12 / POST-UI / Governance
Scope type: governance / architecture rule
Implementation status: not authorized by this document

## 1. Purpose

This document records the Semantic UI First / self-hosted app shell rule.

It prevents Workbench and Semantic Studio from becoming standalone product applications before Semantic can create application UI shells itself.

It strengthens the existing `#675` pause.

It does not close `#675`.

It does not authorize implementation.

It does not rename existing Workbench code paths.

## 2. Rule Statement

Workbench and Semantic Studio must not be developed as standalone product-level applications until Semantic can define and drive application UI shells through its own Semantic UI model, UI AST / IR, admission rules, and renderer adapter contract.

Temporary Workbench surfaces may remain only as bounded presentation, orchestration, diagnostics, reports, and documentation tooling.

Temporary Workbench surfaces must not become Semantic UI owners, release authorities, semantic authorities, or Studio implementation substitutes.

Semantic Studio remains future-facing until Semantic-native UI shell capability exists.

## 3. Architectural Rationale

Workbench and Studio must be results of Semantic capability, not external centers of gravity.

React and Tauri may exist as current tooling shell dependencies, but they must not become the strategic Semantic UI architecture.

Semantic UI must define the model first.

Renderer adapts to Semantic, not Semantic to a foreign UI lifecycle.

This rule protects Semantic from UI-driven authority drift.

## 4. What Is Frozen

- Workbench as a standalone product application.
- Semantic Studio as a standalone product application.
- Workbench product expansion beyond bounded tooling / presentation.
- Semantic Studio product shell implementation.
- Studio as unified control environment implementation.
- React / Tauri shell expansion as the strategic Semantic UI layer.
- Any UI implementation that bypasses Semantic UI model / AST / IR / admission gates.
- Any claim that Workbench or Studio is the current product center.

## 5. What Remains Allowed

- docs-only planning.
- audits and boundary checks.
- existing bounded Workbench tooling / presentation surfaces.
- Semantic UI model seed planning.
- future inert prom-ui-local model seed after explicit owner approval.
- Semantic UI Tree / AST / IR foundation work.
- capability / effect admission planning.
- renderer adapter contract planning.
- self-hosted app shell capability planning.
- dependency / influence register maintenance.

Allowed work must not imply Workbench or Studio implementation approval.

Allowed work must not imply release readiness.

Allowed work must remain subordinate to Local Admission Guard.

## 6. Relationship To Existing R12 Docs Gates

See:

- [`r12_ui_roadmap.md`](./r12_ui_roadmap.md)
- [`r12_post_ui_milestone_map.md`](./r12_post_ui_milestone_map.md)
- [`r12_workbench_studio_pause_guard.md`](./r12_workbench_studio_pause_guard.md)
- [`r12_studio_00_anchor_map.md`](./r12_studio_00_anchor_map.md)
- [`r12_workbench_separation_check.md`](./r12_workbench_separation_check.md)
- [`r12_ui_model_seed_plan.md`](./r12_ui_model_seed_plan.md)

This rule does not invalidate the docs-gate chain.

It refines the meaning of future Workbench / Studio work.

Model seed may proceed only as Semantic UI foundation, not as Workbench / Studio product work.

Workbench and Studio remain blocked as applications until Semantic-native UI shell capability exists.

## 7. Relationship To `#675` and `#595`

`#675` remains active.

This rule strengthens `#675`.

This rule does not close, weaken, or supersede `#675`.

`#595` remains a future Studio planning anchor.

`#595` does not override `#675`.

`#595` does not authorize Studio implementation.

Future Studio work requires both explicit unpause governance and Semantic-native UI shell capability.

## 8. New Naming Meaning

GitHub Project `#2` was renamed from `Workbench` to `Semantic UI Foundation Roadmap`.

The new title reflects the foundation-first direction.

Existing paths such as `apps/workbench` and `docs/workbench` remain historical / current tooling paths and must not be renamed in this task.

Workbench remains a bounded tooling shell, not the roadmap owner.

## 9. Semantic UI Foundation Order

The intended order is:

1. Semantic UI model.
2. UI Tree / UI AST / UI IR.
3. capability / effect admission.
4. renderer adapter contract.
5. Semantic-authored application shell capability.
6. Workbench or Semantic Studio only later as Semantic UI applications.

Workbench / Studio product work before step 5 is forbidden unless explicitly superseded by governance.

## 10. Non-Goals

- no code changes.
- no Workbench implementation.
- no Semantic Studio implementation.
- no UI implementation.
- no model seed code.
- no renderer/backend dependency admission.
- no browser/WebView ownership.
- no widget framework adoption.
- no release widening.
- no stable / production-ready / public-release-ready claim.
- no final API commitment.
- no compiler / verifier / VM / runtime change.
- no dependency addition.
- no closure or weakening of `#675`.
- no physical rename of `apps/workbench`.
- no physical rename of `docs/workbench`.

## 11. Future Impact

Future `R12-UI-MODEL-SEED` code, if approved, is allowed only as Semantic UI foundation.

Future Workbench / Studio PRs must prove they do not violate this rule.

Any future app shell work must show whether it is Semantic-authored or temporary tooling.

Any attempt to make Workbench or Studio product-level before Semantic-native shell capability requires manual governance review.

## 12. Final Decision

Final decision:

READY — CONTINUE SEMANTIC UI FOUNDATION WORK WITHOUT WORKBENCH OR STUDIO PRODUCT EXPANSION
