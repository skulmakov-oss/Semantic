# UI-REENTRY-1 Renderer Boundary Verification

## Status

Result: PASS-WITH-WARNINGS

This is an audit-only verification.

No code was changed.
No tests/examples/7hell files were changed.
No runtime/verifier/VM/SemCode files were changed.
No AGENTS.md changes were made.
No PCC/audit residue was touched.

## Purpose

Verify that renderer boundary docs match the current `prom-ui` renderer source reality after the native/WGPU wording reconciliation.

## Source repo state

- branch: `codex/ui-native-wgpu-boundary-wording`
- HEAD: `9f75fa2bcead68f3925514c00e9c35e5f8334618`
- origin/main: `64dcb4b3ea689e1381fec757a863370b4b677455`
- HEAD == origin/main: no
- dirty tree summary: local checkout is not clean enough for implementation readiness; audit-only verification may continue because it does not modify tracked dirty files
- tracked dirty files: `AGENTS.md`
- untracked residue: local PCC / audit residue remains, including existing post-UI audit notes

## DNA inspection

- files inspected: `docs/dna/SEMANTIC_UI_DNA.md`
- alignment: UI remains projection/cache, not semantic authority; renderer owns the presentation contract, not verifier/runtime semantics; renderer must preserve Unknown / Conflict visibility; no hidden host-effect path; no runtime/verifier/VM/SemCode authority transfer; native backend owns the host bridge / presentation path
- conflicts: none in the DNA itself
- constraints applied: no reset --hard; no tracked dirty file edits; no code/test/example/7hell changes; no implementation claim

## Renderer docs inspected

| File | Status | Notes |
|---|---:|---|
| `docs/roadmap/post_ui/r12_ui_renderer_boundary.md` | current | Now states the renderer contract is abstract, with the admitted native/WGPU path living in backend-native behind feature gates. |
| `docs/roadmap/post_ui/r12_ui_draw_backend_selection_boundary.md` | current | Now treats `wgpu` as backend-native reality without implying a backend switch or ownership transfer. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_reality_reconciliation.md` | current | Evidence note that records the docs/reality gap and the split status model. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_renderer_reality_audit.md` | current | Primary audit evidence for the admitted feature-gated native/WGPU path. |
| `docs/roadmap/post_ui/ui_reentry_3_native_wgpu_reality_alignment.md` | current | Historical checkpoint for the re-entry track; its repo-state snapshot predates this branch state, but the audit trail remains useful. |

## Renderer source inspected

| File | Role | Notes |
|---|---|---|
| `crates/prom-ui/src/renderer.rs` | abstract renderer contract / inert presentation model | Defines `UiRenderModel`, `UiRenderMarker`, and read-only presentation helpers such as `render_projection_to_model`, `present_render_diagnostics`, `present_render_trace`, `present_render_markers`, and `present_render_inspection`. The top-level doc explicitly says the renderer seed is inert and does not draw pixels, dispatch events, execute actions, authorize capabilities, or mutate Semantic state. |
| `crates/prom-ui/src/projection.rs` | projection boundary / source-preserving artifact | Defines `UiProjectionArtifact`, `UiProjectedNode`, inert carrier kinds, and trace preservation. Projection keeps source IDs and trace references but does not gain authority. |
| `crates/prom-ui/src/tree_bridge.rs` | inert tree-to-AST bridge | Structural bridge only; resolution metadata `Known`, `Unknown`, and `Conflict` is ignored during bridging, so the bridge does not flatten the source semantics into authority. |
| `crates/prom-ui/src/interaction.rs` | interaction intent classification scaffold | Defines `InteractionIntentKind::Unknown` and `InteractionSource::Unknown` as explicit states; unknown intent remains unclassified and the module does not call VM, Host ABI, or widget/event-loop behavior. |

## Renderer tests inspected

| Test file | Coverage | Notes |
|---|---|---|
| `crates/prom-ui/tests/renderer_public_api_lock.rs` | public API and entrypoint locks | Locks the renderer entrypoint, public types, and accessor surface. The test suite confirms the renderer surface is intentionally narrow and inert. |
| `crates/prom-ui/tests/ui_render_model_stability.rs` | model determinism and source-reference stability | Verifies render-model stability across repeated runs, sibling ordering, nested ordering, and source reference preservation. |
| `crates/prom-ui/tests/renderer_marker_presentation.rs` | inert marker presentation | Confirms property/action/effect/trace markers are derived deterministically and remain non-executable. |
| `crates/prom-ui/tests/renderer_trace_presentation.rs` | read-only trace presentation | Confirms trace presentation is built from a render model read-only and preserves source links. |
| `crates/prom-ui/tests/renderer_diagnostics_presentation.rs` | read-only diagnostics presentation | Confirms diagnostics presentation is inert, deterministic, and does not introduce backend or capability behavior. |

## Findings

- `prom-ui` still owns the abstract UI model and presentation contract.
- `render_projection_to_model` is an inert downstream consumer of projection artifacts, not a renderer authority transfer.
- `present_render_diagnostics`, `present_render_trace`, `present_render_markers`, and related helpers are read-only presentation helpers, not execution paths.
- `UiProjectionArtifact` and `UiProjectedNode` preserve source identity and traceability.
- `Unknown` and `Conflict` remain explicit states in the tree/projection path and are not flattened by the bridge or renderer contract.
- The renderer docs now match the admitted backend-native WGPU reality recorded in the reconciliation/audit docs.
- No source evidence indicates that the renderer gained semantic authority, runtime authority, or hidden host-effect authority after `#1305`.
- The checkout is not clean enough to claim implementation readiness because `AGENTS.md` is still tracked-dirty and unrelated residue remains.

## Boundary verdict

| Boundary question | Result | Evidence |
|---|---:|---|
| Renderer remains abstract presentation contract | PASS | `crates/prom-ui/src/renderer.rs`, `docs/roadmap/post_ui/r12_ui_renderer_boundary.md` |
| Renderer does not own semantic authority | PASS | `crates/prom-ui/src/renderer.rs`, `crates/prom-ui/src/projection.rs`, `docs/dna/SEMANTIC_UI_DNA.md` |
| Renderer does not own runtime/verifier/VM behavior | PASS | `crates/prom-ui/src/renderer.rs`, `crates/prom-ui/src/interaction.rs`, `docs/dna/SEMANTIC_UI_DNA.md` |
| Unknown / Conflict visibility preserved | PASS | `crates/prom-ui/src/tree_bridge.rs`, `crates/prom-ui/src/interaction.rs`, `docs/dna/SEMANTIC_UI_DNA.md` |
| No hidden host-effect path | PASS | `crates/prom-ui/src/renderer.rs`, `crates/prom-ui/tests/renderer_diagnostics_presentation.rs` |
| Native/WGPU ownership remains backend-native | PASS | `docs/roadmap/post_ui/r12_ui_draw_backend_selection_boundary.md`, `docs/roadmap/post_ui/r12_ui_native_wgpu_renderer_reality_audit.md` |
| Renderer docs match source reality | PASS | `docs/roadmap/post_ui/r12_ui_renderer_boundary.md`, `crates/prom-ui/src/renderer.rs`, `crates/prom-ui/tests/renderer_public_api_lock.rs` |

## Gaps

- No docs/source mismatch was found in the renderer boundary after `#1305`.
- The only readiness gap is local repository hygiene: `HEAD != origin/main` and `AGENTS.md` remains dirty from unrelated work.
- Because this is audit-only, that hygiene gap does not block verification, but it does block any implementation-readiness claim.

## Recommended next step

`UI-REENTRY-2 windowing boundary verification`

This is the next safe audit-only slice because the renderer boundary now matches source reality closely enough, and the remaining local dirty state only blocks implementation claims, not doc verification.

## Non-goals

- no renderer rewrite
- no backend switch
- no runtime/verifier/VM changes
- no PCC/CTF changes
- no tests/examples/7hell changes

## Final verdict

Renderer boundary docs are aligned with current `prom-ui` source reality after `#1305`.

The audit found no semantic-authority regression, no hidden host-effect path, and no flattening of Unknown/Conflict states.
The only warning is repository hygiene: the checkout is not clean enough to claim implementation readiness because `AGENTS.md` remains tracked-dirty and unrelated residue remains uncommitted.
