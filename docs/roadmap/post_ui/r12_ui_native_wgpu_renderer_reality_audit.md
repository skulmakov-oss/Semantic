# R12 UI Native WGPU Renderer Reality Audit

## 1. Purpose
This audit records the actual native renderer reality in `crates/prom-ui-backend-native` so the roadmap can distinguish admitted renderer infrastructure from older boundary language that still describes WGPU as merely future work.

## 2. Closed Basis
Closed basis for this audit:

| PR | Title / Gate | Status |
|----|--------------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1122 | Backend Native Baseline Ledger | MERGED |
| #1123 | Backend Frame Sink Trait | MERGED |
| #1132 | Winit Window Seed Reality Ledger | MERGED |
| #1133 | Winit Run Loop Integration Boundary | MERGED |
| #1134 | Winit Run Loop Integration Source | MERGED |
| #1150 | Native Pipeline Session Hook Source | MERGED |
| #1153 | Render Backend Integration Source | MERGED |
| #1154 | Demo Stabilization and Input Feedback | MERGED |
| #1158 | Demo Interaction Model Extraction | MERGED |
| #1159 | First Native UI Demo Milestone | MERGED |

## 3. Reality Snapshot
Current repository reality includes:

- optional `wgpu-backend` support in `crates/prom-ui-backend-native`;
- optional `winit-backend` support in `crates/prom-ui-backend-native`;
- `NativeBackendWgpuContext` scaffolding;
- `NativeBackendPresentationSurface` scaffolding behind `winit-backend`;
- `selected_draw_backend_name()` returning `"wgpu"` when the feature is enabled;
- minimal offscreen render-pass execution using `wgpu`;
- native surface presentation paths behind feature gates;
- backend-native session hook integration for the full interaction pipeline.

Current code is therefore past the “no WGPU at all” stage.

## 4. Classification
This audit classifies the native renderer posture as:

- `wgpu` dependency/foundation: admitted behind feature gate
- minimal offscreen draw: admitted
- native window surface/presentation: admitted for demo/baseline scope only
- renderer transcript: not yet complete as a public stable contract
- `prom-ui` semantic authority: unchanged, not granted

## 5. Non-Authority Invariants
The following remain true:

- frame presented != semantic success
- render succeeded != action admitted
- draw staged != frame visible
- UI local renderer state != VM state
- renderer transcript != audit authority

## 6. Docs Reality Gap
Older boundary documents still use language that implies WGPU is only future work.
That is now too weak as the primary description of repository reality.

This audit does not claim the renderer is fully production-ready.
It does claim that the repository already contains a real, feature-gated native WGPU path and that the docs should stop describing it as absent.

## 7. Current Recommendation
The safest current status is split:

```text
WGPU foundation: admitted behind feature gate
native surface/presentation: admitted for demo/baseline scope only
renderer transcript: still requires explicit completion if incomplete
UI semantic authority: unchanged, not granted
```

## 8. Validation Surface
Relevant evidence already present in-tree:

- `crates/prom-ui-backend-native/src/lib.rs`
- `crates/prom-ui-backend-native/src/session_hook.rs`
- `crates/prom-ui-backend-native/tests/native_pipeline_session_hook.rs`
- `crates/prom-ui-backend-native/tests/interaction_pipeline_native_hook_smoke.rs`
- `crates/prom-ui-backend-native/tests/backend_run_loop_smoke.rs`

## 9. Recommended Next Lane
Recommended next lane:

`R12-UI-WGPU-STATUS-CLOSEOUT-PR`

If the roadmap does not want a dedicated closeout lane, the next smallest safe step is a docs-only supersession update to the older draw-backend selection / renderer boundary documents.

## 10. Final Decision
PASS WITH WARNINGS — native WGPU renderer reality is admitted in the repository, but the public contract is still split and must not be widened beyond the feature-gated baseline.
