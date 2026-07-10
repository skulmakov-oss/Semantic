# UI Re-entry 3: Native/WGPU Reality Alignment

## Status

Result: PASS-WITH-WARNINGS

This is an audit-only slice.

No code was changed.
No tests/examples/7hell files were changed.
No runtime/verifier/VM/SemCode files were changed.
No PCC/CTF port trail files were changed.

## Purpose

This slice aligns the current UI boundary docs with the repository's actual native/WGPU reality after the closed port / housekeeping cycle.
It does not introduce implementation work.

## Source repo state

- branch: `main`
- HEAD: `0db9eb8316431181efc255b5be4a0dd40dab8b60`
- origin/main: `0db9eb8316431181efc255b5be4a0dd40dab8b60`
- main == origin/main: yes
- dirty tree summary: untracked local audit docs remain under `docs/roadmap/pcc/` plus the existing `docs/roadmap/post_ui/ui_reentry_checkpoint.md`

## DNA inspection

- files inspected: `docs/dna/SEMANTIC_UI_DNA.md`
- alignment: UI remains projection/cache and not semantic authority; renderer backend ownership stays separated; native backend owns the host bridge; UI must not flatten Unknown/Conflict states
- conflicts: none in the DNA itself
- constraints applied: no UI authority transfer; no runtime/verifier/VM authority transfer; no hidden host-effect path; no renderer/backend ownership collapse

## UI docs inventory

| File | Exists | Status | Notes |
|---|---:|---|---|
| `docs/roadmap/post_ui/r12_ui_renderer_boundary.md` | yes | needs-review | Still describes the renderer as a future boundary; current repository reality already includes feature-gated native WGPU paths and related audit evidence. |
| `docs/roadmap/post_ui/r12_ui_windowing_boundary.md` | yes | current | Continues to model windowing as a host boundary, which is consistent with the UI DNA. |
| `docs/roadmap/post_ui/r12_ui_draw_backend_selection_boundary.md` | yes | needs-review | Boundary language is older than the admitted WGPU reality recorded elsewhere. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_reality_reconciliation.md` | yes | current | Explicitly records the gap between older docs and actual backend-native WGPU reality. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_renderer_reality_audit.md` | yes | current | Primary evidence record for native WGPU reality in `prom-ui-backend-native`. |
| `docs/roadmap/post_ui/r12_ui_interaction_pipeline_integration_source_closeout.md` | yes | current | Closed source-closeout; interaction pipeline integration is already complete. |
| `docs/roadmap/post_ui/r12_ui_intent_admission_and_dispatch_source_closeout.md` | yes | current | Closed source-closeout; intent admission/dispatch source work is already complete. |

## UI source inventory

| Area | Observed files/modules | Notes |
|---|---|---|
| renderer | `crates/prom-ui/src/renderer.rs`, `crates/prom-ui/src/projection.rs`, `crates/prom-ui/src/tree_bridge.rs`, `crates/prom-ui/tests/renderer_*.rs`, `crates/prom-ui/tests/ui_render_model_stability.rs` | `prom-ui` owns the abstract render/presentation contract; tests lock its API and stability properties. |
| windowing | `crates/prom-ui-runtime/src/adapter_boundary.rs`, `crates/prom-ui-runtime/src/admission_facade.rs`, `crates/prom-ui-backend-native/src/lib.rs`, `crates/prom-ui-backend-native/src/session_hook.rs` | Window lifecycle is split between runtime boundary orchestration and backend-native integration. |
| native/wgpu | `crates/prom-ui-backend-native/src/lib.rs`, `crates/prom-ui-backend-native/src/draw_generation.rs`, `crates/prom-ui-backend-native/src/frame_sink.rs`, `crates/prom-ui-backend-native/tests/native_backend_wgpu_feature_*.rs`, `crates/prom-ui-backend-native/tests/native_backend_winit_*.rs` | Native WGPU support is present behind feature gates and is not merely hypothetical. |
| interaction pipeline | `crates/prom-ui-runtime/src/interaction_pipeline.rs`, `crates/prom-ui-runtime/src/intent_admission.rs`, `crates/prom-ui-runtime/src/intent_dispatch.rs`, `crates/prom-ui-backend-native/src/session_hook.rs` | The capture -> route -> map -> admit -> dispatch flow is already source-closed and hooked from backend-native. |
| intent admission / dispatch | `crates/prom-ui-runtime/src/intent_admission.rs`, `crates/prom-ui-runtime/src/intent_dispatch.rs`, `crates/prom-ui-runtime/src/intent_capability.rs`, `crates/prom-ui-runtime/src/intent_audit.rs` | The admission/dispatch split is explicit; intent is non-authoritative by design. |
| tests | `crates/prom-ui/tests/*.rs`, `crates/prom-ui-runtime/tests/*.rs`, `crates/prom-ui-backend-native/tests/*.rs` | Contract coverage exists across renderer, runtime, native bridge, and UI boundary seams. |

## Findings

- The checkout is now synchronized: `main == origin/main`.
- `prom-ui` remains the abstract UI source-of-truth for the UI model and presentation contract.
- `prom-ui-runtime` owns orchestration and boundary enforcement around interaction and admission.
- `prom-ui-backend-native` owns the host bridge and the admitted native/WGPU reality.
- Older renderer/draw-backend boundary wording is now behind the source reality and should be treated as needing review, not as the primary truth.
- The interaction pipeline and intent admission/dispatch source work are already closed out and should not be reopened for implementation.

## Gaps

- Renderer and draw-backend docs still lag the admitted backend-native WGPU reality.
- The cleanest next move is another audit/doc reconciliation pass, not code.
- Untracked PCC/port residue is still present locally, but it is outside the requested UI scope.

## Candidate next slices

| Slice | Result | Reason | Suggested next action |
|---|---|---|---|
| UI-REENTRY-1 renderer boundary verification | READY-WITH-WARNINGS | Renderer boundary exists, but the wording must be reconciled with current WGPU reality first. | Re-read the renderer boundary docs against `prom-ui` renderer/projection source. |
| UI-REENTRY-2 windowing boundary verification | READY | Windowing is already a clear host boundary and can be revalidated against source. | Reconfirm the window lifecycle split in `prom-ui-runtime` and `prom-ui-backend-native`. |
| UI-REENTRY-3 native/wgpu reality alignment | READY | This is the safest and most direct doc/source reconciliation slice after sync. | Update or replace stale boundary wording so it matches the admitted native WGPU path. |
| UI-REENTRY-4 interaction pipeline source audit | NOT-RECOMMENDED | Source closeout already exists. | Leave closed unless evidence of regression appears. |
| UI-REENTRY-5 intent admission / dispatch source audit | NOT-RECOMMENDED | Source closeout already exists. | Leave closed unless evidence of regression appears. |
| UI-REENTRY-6 minimal UI implementation slice | BLOCKED | Boundary reconciliation still comes first. | Defer implementation until docs and source are aligned. |

## Recommended next step

Choose exactly one next step:

`UI-REENTRY-3 native/wgpu reality alignment`

This remains the smallest safe next slice because it is doc/audit-only, it stays within the UI boundary, and it fixes the actual mismatch that the checkpoint surfaced.

## Non-goals

- no PCC Practical Core port
- no CTF sync claim
- no runtime/verifier/VM changes
- no renderer rewrite
- no native/wgpu backend switch
- no broad refactor

## Final verdict

PASS-WITH-WARNINGS is correct for this slice.

The repository is now on synchronized `main`, but the UI docs still contain older boundary wording that trails the actual native/WGPU backend reality.
The next step should be a documentation reconciliation slice, not implementation.
