# R12 Workbench Separation Check

Status: Draft
Track: R12 / Workbench / POST-UI
Scope type: planning / boundary audit
Implementation status: not authorized by this document

## 1. Purpose

This document records the R12 Workbench separation check.

It verifies Workbench as presentation / orchestration / tooling surface only.

It does not authorize Workbench implementation expansion.

It does not authorize Semantic Studio implementation.

It does not authorize Semantic UI model code.

It does not claim readiness, stability, release readiness, or production readiness.

## 2. Governance Context

See:

- [`docs/roadmap/post_ui/r12_ui_roadmap.md`](./r12_ui_roadmap.md)
- [`docs/roadmap/post_ui/r12_post_ui_milestone_map.md`](./r12_post_ui_milestone_map.md)
- [`docs/roadmap/post_ui/r12_workbench_studio_pause_guard.md`](./r12_workbench_studio_pause_guard.md)
- [`docs/roadmap/post_ui/r12_studio_00_anchor_map.md`](./r12_studio_00_anchor_map.md)

`#675` remains active.

`#595` remains future-facing.

Workbench separation check does not close `#675`.

Workbench separation check does not unblock Studio implementation.

Workbench separation check does not unblock model seed by itself.

Workbench separation check is one docs gate before model seed planning can be considered complete.

## 3. Workbench Current Posture

Workbench is:

- existing tooling shell
- presentation and orchestration layer
- docs / spec navigator
- local report / command / snapshot surface
- not source of truth
- not semantic authority
- not release authority
- not Semantic UI owner

## 4. Inspected Surfaces

| Surface | Files / docs inspected | Current role | Boundary assessment | Risk | Notes |
| --- | --- | --- | --- | --- | --- |
| Workbench frontend shell | `apps/workbench/src/App.tsx`, `App.css`, `index.css`, `main.tsx`, `diagnostics.ts`, `workbench-api.ts`, `workbench-state.ts` | presentation / orchestration / tooling shell | allowed; remains downstream from canonical docs, jobs, and release docs | medium | includes read-only docs navigator, local command display, release visibility, and cached UI state; should remain non-authoritative |
| Workbench Tauri backend | `apps/workbench/src-tauri/src/adapter.rs`, `docs.rs`, `lib.rs`, `snapshot.rs`, `reports.rs`, `workspace_files.rs`, `lsp_bridge.rs`, `scaffold.rs` | bridge / snapshot / report / workspace helper layer | allowed within current shell boundaries | medium | canonical navigator, workspace helpers, snapshot/report export, and bridge surfaces are useful but must not become semantic truth or release authority |
| docs/spec navigator | `apps/workbench/src/App.tsx`, `apps/workbench/src-tauri/src/docs.rs` | read-only docs navigation | allowed; read-only navigation over canonical repository docs | low | navigator points to canonical docs; it does not fork them |
| release / report surfaces | `apps/workbench/src/App.tsx`, `apps/workbench/src-tauri/src/reports.rs`, `apps/workbench/src-tauri/src/snapshot.rs` | visibility-only release/report panels | allowed with strict read-only positioning | medium | release docs and report exports are visibility surfaces, not release authority |
| diagnostics surfaces | `apps/workbench/src/App.tsx`, `apps/workbench/src/diagnostics.ts` | diagnostics presentation | allowed when tied to canonical source docs and outputs | low | diagnostics are emitted truth presentation, not duplicated parser/verifier semantics |
| workspace file helpers | `apps/workbench/src-tauri/src/workspace_files.rs` | workspace path resolution / file helper | allowed within repository-scoped helper boundaries | medium | path helpers are useful but should remain helpers, not ownership transfer |
| LSP bridge / scaffold helpers | `apps/workbench/src-tauri/src/lsp_bridge.rs`, `apps/workbench/src-tauri/src/scaffold.rs` | integration helper surfaces | allowed only as bounded helper surfaces | medium | watch for any creep toward semantic authority or hidden execution semantics |
| Workbench docs | `docs/workbench/*.md` | documentation of current Workbench scope | allowed and important for boundary clarity | low | docs consistently describe Workbench as presentation/orchestration and caution against truth/authority widening |
| R12 governance docs | `docs/roadmap/post_ui/r12_ui_roadmap.md`, `docs/roadmap/post_ui/r12_post_ui_milestone_map.md`, `docs/roadmap/post_ui/r12_workbench_studio_pause_guard.md`, `docs/roadmap/post_ui/r12_studio_00_anchor_map.md`, `docs/roadmap/post_ui/r12_ui_ownership_project_seed.md` | POST-UI governance chain | allowed; these docs define the pause, milestone, and future anchors | low | these docs show the intended governance hierarchy and future docs gates |

## 5. Separation Matrix

| Boundary | Workbench allowed role | Workbench forbidden role | Evidence | Result |
| --- | --- | --- | --- | --- |
| Semantic UI model ownership | display/planning references only | UI Tree / UI AST / UI IR ownership | `docs/roadmap/post_ui/r12_ui_ownership_project_seed.md`, `docs/roadmap/post_ui/r12_ui_roadmap.md` | pass |
| Semantic authority | present repository truth | define semantic truth | `apps/workbench/src/App.tsx`, `docs/workbench/view_models.md` | pass |
| Release authority | display canonical release docs/reports | decide release readiness | `apps/workbench/src/App.tsx`, `apps/workbench/src-tauri/src/reports.rs`, `docs/workbench/remaining_readiness_surface.md` | watch |
| Verifier / VM / runtime authority | present outputs/results | change verifier/VM/runtime behavior | `apps/workbench/src/App.tsx`, `apps/workbench/src/diagnostics.ts`, `apps/workbench/src-tauri/src/docs.rs` | pass |
| Local Admission Guard | reference as authoritative | replace with GitHub CI or Workbench logic | `apps/workbench/src/App.tsx`, `docs/roadmap/post_ui/r12_workbench_studio_pause_guard.md` | pass |
| Semantic Studio implementation | future anchor references | implement Studio shell/control environment | `docs/roadmap/post_ui/r12_studio_00_anchor_map.md` | pass |
| Renderer/backend ownership | adapter boundary references | renderer ownership of Semantic UI model | `apps/workbench/src-tauri/src/adapter.rs`, `docs/roadmap/post_ui/r12_workbench_studio_pause_guard.md` | pass |
| Browser/WebView ownership | Workbench shell implementation dependency | browser/WebView ownership of Semantic state | `apps/workbench/src/App.tsx`, `apps/workbench/src-tauri/src/lib.rs` | pass |

## 6. Allowed Workbench Work Under `#675`

- docs-only Workbench planning
- audits
- separation checks
- read-only navigator refinement within existing scope
- presentation of repository docs
- presentation of local reports
- view-model cleanup that does not alter authority
- command / report boundary documentation

Any code work must require separate explicit authorization.

This document does not provide that authorization.

## 7. Forbidden Workbench Work Under `#675`

- Workbench becoming Semantic UI model owner
- Workbench defining UI Tree / UI AST / UI IR
- Workbench owning semantic truth
- Workbench owning release truth
- Workbench replacing Local Admission Guard
- Workbench using GitHub CI as authoritative gate
- Workbench becoming Semantic Studio implementation
- Workbench adding renderer/backend ownership
- Workbench adding browser/WebView ownership of Semantic state
- Workbench adding widget framework scope
- Workbench expanding compiler / verifier / VM / runtime behavior

## 8. Risk / Watch Items

- command / report surfaces must remain presentation-only
- docs/spec navigator must remain read-only and canonical-doc backed
- release / report UI must not become release authority
- workspace / scaffold helpers must not become semantic authority
- future Studio integration must remain blocked by `#675`
- future model seed must avoid Workbench coupling

## 9. Model Seed Impact

`R12-UI-MODEL-SEED` remains blocked behind docs gates.

Workbench separation check is one required docs gate.

Model seed must not touch Workbench.

Model seed must not touch Semantic Studio.

Model seed must remain prom-ui-local and inert when later authorized.

Model seed must not add renderer/backend dependencies.

Model seed must not add parser / lowering / VM / runtime integration.

Model seed must not create widget / layout framework scope.

## 10. Non-Goals

- no Workbench implementation
- no Semantic Studio implementation
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

- `R12-UI-MODEL-SEED-PLAN`
- `R12-UI-MODEL-SEED` only after docs gates

Optional:

- future command/effect boundary audit if ambiguity remains
- future release/report authority audit if needed

## 12. Final Decision

Final decision:

READY — CONTINUE WITH DOCS-ONLY MODEL SEED PLANNING
