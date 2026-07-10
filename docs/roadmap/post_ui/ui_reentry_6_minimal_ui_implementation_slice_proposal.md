# UI-REENTRY-6 Minimal UI Implementation Slice Proposal

## Status

Result: READY-WITH-WARNINGS

This is a proposal-only document.

No code was changed.
No tests/examples/7hell files were changed.
No runtime/verifier/VM/SemCode files were changed.
No PCC/CTF files were changed.
No untracked residue was touched.

## Purpose

Select the first minimal UI implementation slice after UI re-entry boundary verification.

## Source repo state

- branch: `main`
- HEAD: `fca5e55108fd47246679bc19e32a0579e2dce835`
- origin/main: `fca5e55108fd47246679bc19e32a0579e2dce835`
- HEAD == origin/main: yes
- AGENTS.md state: clean
- dirty tree summary: tracked state is clean; only local untracked residue remains
- untracked residue: 29 local files/groups remain, including existing post-UI audit notes and PCC residue; none were touched

## DNA inspection

- files inspected: `docs/dna/SEMANTIC_UI_DNA.md`
- alignment: UI remains projection/cache, not semantic authority; renderer owns the presentation contract, not verifier/runtime semantics; windowing remains a host/presentation lifecycle boundary; native backend owns the host bridge / presentation path; Unknown / Conflict states must not be flattened; no hidden host-effect path; no runtime/verifier/VM/SemCode authority transfer
- conflicts: none in the DNA itself
- constraints applied: no implementation code; no tests/examples/7hell; no PCC/CTF touch; no backend switch; no renderer rewrite; no reliance on untracked residue

## Re-entry evidence

| Re-entry item | Status | Notes |
|---|---:|---|
| native/WGPU wording | verified | The boundary wording was reconciled to acknowledge the admitted feature-gated backend-native WGPU reality. |
| renderer boundary | verified | Renderer contract docs now match `prom-ui` source reality and keep authority in the projection/presentation layer. |
| windowing boundary | verified | Windowing docs match the runtime/native host-bridge split and do not claim semantic authority. |
| AGENTS.md hygiene | clean | `AGENTS.md` is no longer tracked-dirty and does not block a new implementation branch. |

## Source inventory

| Area | Owning crate | Candidate files | Notes |
|---|---|---|---|
| renderer | `prom-ui` | `crates/prom-ui/src/renderer.rs`, `crates/prom-ui/tests/renderer_public_api_lock.rs`, `crates/prom-ui/tests/renderer_marker_presentation.rs`, `crates/prom-ui/tests/renderer_trace_presentation.rs`, `crates/prom-ui/tests/ui_render_model_stability.rs` | Abstract presentation contract is already stable; the narrowest first slice is to tighten the contract tests around the exported renderer surface. |
| runtime/admission | `prom-ui-runtime` | `crates/prom-ui-runtime/src/adapter_boundary.rs`, `crates/prom-ui-runtime/src/admission_facade.rs`, `crates/prom-ui-runtime/src/interaction_pipeline.rs`, `crates/prom-ui-runtime/src/intent_admission.rs`, `crates/prom-ui-runtime/src/intent_dispatch.rs`, `crates/prom-ui-runtime/tests/*.rs` | Ownership is clear, but the source-closeout work is already done; not the smallest first slice. |
| native backend | `prom-ui-backend-native` | `crates/prom-ui-backend-native/src/lib.rs`, `crates/prom-ui-backend-native/src/session_hook.rs`, `crates/prom-ui-backend-native/src/frame_sink.rs`, `crates/prom-ui-backend-native/src/draw_generation.rs`, `crates/prom-ui-backend-native/tests/native_backend_winit_*.rs`, `crates/prom-ui-backend-native/tests/backend_run_loop_smoke.rs`, `crates/prom-ui-backend-native/tests/static_visible_demo_smoke.rs` | Host bridge and native/WGPU reality live here; useful for a later implementation slice, but slightly heavier than the renderer contract test tightening. |
| tests | all three UI crates | `crates/prom-ui/tests/*.rs`, `crates/prom-ui-runtime/tests/*.rs`, `crates/prom-ui-backend-native/tests/*.rs` | Contract coverage already exists across the stack; the first implementation slice should stay inside one crate and one seam. |

## Candidate slice matrix

| Slice | Status | Owning crate | Expected files | Validation | Risk |
|---|---:|---|---|---|---|
| UI-IMPL-1 renderer contract test tightening | READY-WITH-WARNINGS | `prom-ui` | `crates/prom-ui/tests/renderer_public_api_lock.rs` | `cargo test -p prom-ui --test renderer_public_api_lock`; optionally run the neighboring renderer presentation tests | Low; touches only the abstract renderer contract seam and keeps runtime/backend ownership untouched. |
| UI-IMPL-2 native backend smoke validation improvement | READY-WITH-WARNINGS | `prom-ui-backend-native` | `crates/prom-ui-backend-native/tests/native_backend_winit_run_loop_plan_contract.rs`, `crates/prom-ui-backend-native/tests/backend_run_loop_smoke.rs`, `crates/prom-ui-backend-native/tests/static_visible_demo_smoke.rs` | `cargo test -p prom-ui-backend-native --test native_backend_winit_run_loop_plan_contract`; `cargo test -p prom-ui-backend-native --test backend_run_loop_smoke` | Medium; still narrow, but the validation surface is more integration-heavy. |
| UI-IMPL-3 frame sink / draw generation small hardening | READY-WITH-WARNINGS | `prom-ui-backend-native` | `crates/prom-ui-backend-native/src/frame_sink.rs`, `crates/prom-ui-backend-native/src/draw_generation.rs`, plus focused backend-native tests | Targeted `cargo test -p prom-ui-backend-native` coverage around frame staging and draw staging | Medium; still isolated to one crate, but closer to the native host bridge and draw path. |
| UI-IMPL-4 interaction pipeline regression test | NOT-RECOMMENDED | `prom-ui-runtime` | `crates/prom-ui-runtime/tests/interaction_pipeline_tick_frame_smoke.rs`, `crates/prom-ui-runtime/tests/runtime_intent_dispatch_contract.rs` | Runtime tests only | Medium; source-closeout already exists, so this is not the first implementation slice unless a regression is found. |
| UI-IMPL-5 static visible demo smoke clarification | NEEDS-DESIGN-DECISION | `prom-ui-backend-native` | `crates/prom-ui-backend-native/tests/static_visible_demo_smoke.rs` | Manual desktop session plus backend-native smoke test | Medium/high; the manual demo path is useful, but it is not the cleanest first implementation branch. |
| UI-IMPL-6 documentation-only closeout, no implementation yet | NOT-RECOMMENDED | docs only | `docs/roadmap/post_ui/*.md` | N/A | Low, but it is not an implementation slice. |

## Recommended first slice

**UI-IMPL-1 — renderer contract test tightening**

- Owning crate: `prom-ui`
- Expected files: `crates/prom-ui/tests/renderer_public_api_lock.rs`
- Expected tests: `cargo test -p prom-ui --test renderer_public_api_lock`
- Non-goals:
  - no renderer rewrite
  - no backend switch
  - no runtime/verifier/VM changes
  - no PCC/CTF changes
  - no untracked residue cleanup
  - no new native/WGPU work
- Acceptance criteria:
  - renderer public surface remains locked to the existing abstract presentation contract
  - no semantic authority transfer is introduced
  - no runtime/verifier/VM/SemCode ownership moves into the renderer seam
  - the test diff stays minimal and local to `prom-ui`
- Rollback criteria:
  - if the contract tightening forces broader API changes outside `renderer_public_api_lock.rs`
  - if the change starts pulling in backend/runtime code
  - if the slice cannot be validated with a narrow `prom-ui` test run

## Non-goals

- no renderer rewrite
- no backend switch
- no runtime/verifier/VM changes
- no PCC/CTF changes
- no broad refactor
- no untracked residue cleanup

## Required gates before implementation

1. Start from clean `main == origin/main`.
2. Create a dedicated feature branch.
3. Stage only approved files.
4. Run targeted tests.
5. Keep PR small.
6. Do not include local audit residue.

## Final verdict

Implementation may start, but only as a narrow `prom-ui` renderer-contract test-tightening slice.

The repository is aligned enough for a first implementation branch, while untracked local residue remains intentionally isolated and must stay out of the slice. The first safe step is a minimal contract-hardening PR in `prom-ui`, not a runtime or backend expansion.
