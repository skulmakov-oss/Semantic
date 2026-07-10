# UI-REENTRY-2 Windowing Boundary Verification

## Status

Result: PASS-WITH-WARNINGS

This is an audit-only verification.

No code was changed.
No tests/examples/7hell files were changed.
No runtime/verifier/VM/SemCode files were changed.
No AGENTS.md changes were made.
No PCC/audit residue was touched.

## Purpose

Verify that windowing boundary docs match the current `prom-ui-runtime` and `prom-ui-backend-native` source reality.

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
- alignment: UI remains projection/cache, not semantic authority; windowing is a host boundary / presentation lifecycle, not semantic authority; native backend owns the host bridge / presentation path; no hidden host-effect path; no runtime/verifier/VM/SemCode authority transfer; Unknown / Conflict states must not be flattened
- conflicts: none in the DNA itself
- constraints applied: no `reset --hard`; no tracked dirty file edits; no code/test/example/7hell changes; no implementation claim

## Windowing docs inspected

| File | Status | Notes |
|---|---:|---|
| `docs/roadmap/post_ui/r12_ui_windowing_boundary.md` | current | Boundary language stays at the contract layer while explicitly noting the feature-gated native windowing/WGPU path and the dedicated reality audit. |
| `docs/roadmap/post_ui/r12_ui_draw_backend_selection_boundary.md` | current | Draw-backend wording now reflects admitted backend-native WGPU reality without implying a backend switch or ownership collapse. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_reality_reconciliation.md` | current | Records the docs/reality gap and the split status model for native WGPU capabilities. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_renderer_reality_audit.md` | current | Primary audit evidence for admitted backend-native WGPU reality. |
| `docs/roadmap/post_ui/ui_reentry_3_native_wgpu_reality_alignment.md` | current | Earlier checkpoint for the WGPU reality reconciliation track; still useful as a trail marker. |
| `docs/roadmap/post_ui/ui_reentry_1_renderer_boundary_verification.md` | current | Confirms the renderer boundary is already aligned with source reality after `#1305`. |

## Windowing source inspected

| File | Role | Notes |
|---|---|---|
| `crates/prom-ui-runtime/src/adapter_boundary.rs` | logical runtime adapter seam | Defines logical `WindowId`, `FrameId`, and `DrawBatchId`, keeps the adapter seam narrow, and explicitly excludes OS handles, renderer internals, ABI details, and capability policy. |
| `crates/prom-ui-runtime/src/admission_facade.rs` | local target-shape validation | Admits only shape checks and forwards shaped requests to the adapter boundary; it does not own lifecycle, capability, audit, or platform execution. |
| `crates/prom-ui-runtime/src/interaction_pipeline.rs` | semantic interaction coordination | Captures the `capture -> route -> map -> admit -> dispatch` path and keeps windowing out of semantic authority. |
| `crates/prom-ui-runtime/src/intent_admission.rs` | execution admission gate | Keeps semantic intent distinct from execution authority and audits the evaluation before dispatch. |
| `crates/prom-ui-runtime/src/intent_dispatch.rs` | execution dispatcher | Dispatches admitted actions to state update; it is not a windowing owner. |
| `crates/prom-ui-backend-native/src/lib.rs` | native host bridge / windowing scaffold | Owns the feature-gated native windowing path, event-loop creation, window creation scaffolds, and raw host-event translation while keeping the native backend crate local to host/platform behavior. |
| `crates/prom-ui-backend-native/src/session_hook.rs` | transport hook into runtime pipeline | Transports raw backend evidence into the runtime interaction pipeline without taking semantic ownership. |
| `crates/prom-ui-backend-native/src/frame_sink.rs` | inert frame evidence boundary | Defines `UiBackendFrame` and `UiFrameSink` as inert frame evidence and a submission seam. |
| `crates/prom-ui-backend-native/src/draw_generation.rs` | draw-frame staging | Converts layout placement into a `DrawFrame`; this is draw staging, not semantic authority. |

## Windowing tests inspected

| Test file | Coverage | Notes |
|---|---|---|
| `crates/prom-ui-runtime/tests/runtime_intent_dispatch_contract.rs` | admission/dispatch contract | Confirms default runtime admission denies execution by default and preserves the intent/admission split. |
| `crates/prom-ui-runtime/tests/interaction_pipeline_tick_frame_smoke.rs` | interaction pipeline tick flow | Verifies the capture/route/map/admit/dispatch flow and that denied intents do not dispatch. |
| `crates/prom-ui-backend-native/tests/native_backend_winit_feature_contract.rs` | winit feature gate | Confirms the `winit-backend` gate behavior and feature-gated placeholder availability. |
| `crates/prom-ui-backend-native/tests/native_backend_winit_window_creation_contract.rs` | window creation scaffold | Confirms the scaffold exists, starts empty, and exposes the expected `ApplicationHandler` shape. |
| `crates/prom-ui-backend-native/tests/native_backend_winit_run_loop_plan_contract.rs` | run-loop plan and readiness | Confirms the run-loop plan stays out of renderer/presentation ownership and keeps runtime boundaries clean. |
| `crates/prom-ui-backend-native/tests/native_backend_winit_app_facade_contract.rs` | app facade lifecycle | Confirms the facade requires staged config, preserves backend staging, and does not claim platform wiring before run. |
| `crates/prom-ui-backend-native/tests/native_backend_winit_app_facade_draw_run_transcript_contract.rs` | draw/run transcript split | Confirms draw staging and run transcripts remain separated from renderer/presentation authority. |
| `crates/prom-ui-backend-native/tests/backend_run_loop_smoke.rs` | native run-loop smoke | Confirms the backend run-loop seam exists as an integration point. |
| `crates/prom-ui-backend-native/tests/static_visible_demo_smoke.rs` | visible demo smoke | Confirms the feature-gated static visible demo path can create a surface and present a minimal clear in a manual session. |

## Findings

- `prom-ui-runtime` owns the logical runtime adapter seam, target-shape validation, and the semantic interaction pipeline.
- `prom-ui-backend-native` owns the native host bridge and the feature-gated windowing path.
- `UiBackendFrame` remains inert frame evidence, not semantic authority.
- `UiRuntimeEffectRequest` / `UiAdmissionFacade` preserve the split between shape validation and adapter submission.
- `InteractionPipeline` keeps admission and dispatch on the runtime side, not the windowing side.
- Native backend source shows real winit scaffolding, run-loop planning, event translation, and manual smoke/demo entry points behind feature gates.
- The windowing docs do not grant semantic authority to the host/window lifecycle and do not collapse the backend-native split into the renderer boundary.
- The docs still speak at the contract layer, which is consistent with the current repo shape, but implementation readiness is still blocked by local hygiene.

## Boundary verdict

| Boundary question | Result | Evidence |
|---|---:|---|
| Windowing remains host/presentation boundary | PASS | `docs/roadmap/post_ui/r12_ui_windowing_boundary.md`, `crates/prom-ui-backend-native/src/lib.rs`, `crates/prom-ui-backend-native/src/frame_sink.rs` |
| Windowing does not own semantic authority | PASS | `docs/dna/SEMANTIC_UI_DNA.md`, `crates/prom-ui-runtime/src/adapter_boundary.rs`, `crates/prom-ui-runtime/src/interaction_pipeline.rs` |
| Windowing does not own runtime/verifier/VM behavior | PASS | `crates/prom-ui-runtime/src/admission_facade.rs`, `crates/prom-ui-runtime/src/intent_admission.rs`, `crates/prom-ui-runtime/src/intent_dispatch.rs` |
| Native backend owns host bridge | PASS | `crates/prom-ui-backend-native/src/lib.rs`, `crates/prom-ui-backend-native/src/session_hook.rs`, `crates/prom-ui-backend-native/src/frame_sink.rs` |
| No hidden host-effect path | PASS | `crates/prom-ui-runtime/src/interaction_pipeline.rs`, `crates/prom-ui-backend-native/src/session_hook.rs`, `crates/prom-ui-backend-native/src/draw_generation.rs` |
| Unknown / Conflict visibility preserved | PASS | `docs/dna/SEMANTIC_UI_DNA.md`, `crates/prom-ui-runtime/src/adapter_boundary.rs`, `crates/prom-ui-runtime/src/interaction_pipeline.rs` |
| Windowing docs match source reality | PASS | `docs/roadmap/post_ui/r12_ui_windowing_boundary.md`, `crates/prom-ui-backend-native/src/lib.rs`, `crates/prom-ui-backend-native/tests/native_backend_winit_run_loop_plan_contract.rs` |

## Gaps

- No docs/source mismatch was found that requires an immediate windowing docs patch.
- The checkout is not clean enough to claim implementation readiness because `HEAD != origin/main` and `AGENTS.md` remains tracked-dirty from unrelated work.
- Untracked PCC/audit residue is still present locally, but it is outside this slice.

## Recommended next step

`BLOCKED until local dirty tracked files are resolved`

Implementation may be recommended only if:

- docs and source align;
- DNA boundaries are clear;
- no dirty tracked files block a clean implementation branch;
- validation path exists.

## Non-goals

- no windowing implementation
- no renderer rewrite
- no backend switch
- no runtime/verifier/VM changes
- no PCC/CTF changes
- no tests/examples/7hell changes

## Final verdict

The windowing boundary docs are aligned with the current source reality at the contract layer.

The audit found no semantic-authority transfer, no hidden host-effect path, and no flattening of Unknown/Conflict states.
The only warning is repository hygiene: this checkout is not ready for implementation because `AGENTS.md` is still tracked-dirty and the branch is not synced to `origin/main`.
