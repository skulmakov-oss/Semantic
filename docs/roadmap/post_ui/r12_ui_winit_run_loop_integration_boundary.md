# R12 UI Winit Run Loop Integration Boundary

## 1. Purpose
This boundary document defines the future integration boundary between the
existing crate-local `winit-backend` scaffolding and normal
`NativeBackend::run_event_loop(...)` lifecycle.

It does not implement run-loop integration.
It does not change source code, tests, Cargo, dependencies, DNA, or Admission
Guard.

It introduces no drawing, frame presentation, hit testing, interaction,
runtime handoff, action execution, effect execution, or capability admission.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1122 | Backend Native Baseline Ledger | MERGED |
| #1123 | Backend Frame Sink Trait | MERGED |
| #1124 | Offscreen Static Frame Test | MERGED |
| #1125 | Windowing Boundary | MERGED |
| #1132 | Winit Window Seed Reality Ledger | MERGED |

## 3. Boundary Summary
The current backend-native baseline already contains manual, feature-gated
`winit` scaffolding.

The remaining architectural gap is not whether `winit` exists. It is whether
and how the existing manual paths may become normal backend lifecycle
integration.

Current state:

```text
NativeBackend::run_event_loop(...)
  -> staged pending-event drain only
  -> no winit EventLoop ownership
  -> no native Window ownership
  -> no run_app integration
```

Future boundary under review:

```text
NativeBackend
  -> future WinitRunLoopHost / NativeWindowLifecycle owner
  -> winit EventLoop::run_app(...)
  -> staged window events as inert metadata
  -> no semantic execution
```

## 4. SEMANTIC_UI_DNA Compliance
PASS - UI remains projection/cache, not semantic authority.
PASS - native backend remains a boundary crate.
PASS - `prom-ui` core remains backend-free and windowing-free.
PASS - run-loop ownership does not imply semantic truth.
PASS - native events are not Semantic actions.
PASS - native close/key/window events are not effects.
PASS - runtime/capability admission is not bypassed.
PASS - Unknown/Conflict semantics are not flattened.

docs/dna inspected: YES

DNA files inspected:

- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:

- Semantic UI defines model and contracts; renderer/native backends adapt.
- Native backend may host platform lifecycle but must not become Semantic UI
  owner, verifier, runtime, action, effect, capability, or audit authority.
- UI state and native backend state remain projection/staging state, not
  semantic state.

DNA conflicts detected: none.

## 5. Existing Baseline Facts
Existing repository facts from the accepted baseline:

- `winit-backend` feature exists.
- optional `winit` dependency exists.
- event-loop creation helper exists.
- window config to `WindowAttributes` translation exists.
- native window creation scaffold exists.
- manual/ignored `run_app(...)` smoke paths exist.
- `NativeBackendWinitApp` facade exists.
- `NativeBackendWinitAppState` exists.
- run-loop integration plan records
  `integrates_with_backend_run_event_loop: false`.
- renderer/frame presentation flags remain false in the current plan.

## 6. Ownership Boundary
A future run-loop integration source gate must explicitly define ownership of:

- the `winit::event_loop::EventLoop`;
- the native `Window`;
- the `ApplicationHandler` state;
- staged `NativeBackend` state;
- translated inert input events;
- close-request lifecycle;
- error/reporting surface.

The source gate must also define what is consumed, borrowed, retained, or
returned.

No hidden global event loop ownership is admitted by this boundary.
No `prom-ui` core type may own a native `winit` handle.

## 7. Allowed Future Semantics
Allowed future semantics, if admitted by a later source PR:

- create a native `EventLoop` inside `prom-ui-backend-native`;
- create a native `Window` inside a valid `winit` lifecycle callback;
- run a crate-local `ApplicationHandler`;
- stage close/key/window events as inert input metadata;
- return explicit typed errors or run summaries;
- keep `prom-ui-runtime` platform-neutral;
- keep `prom-ui` core windowing-free;
- preserve existing frame evidence without rewriting it.

## 8. Forbidden Semantics
Forbidden by this boundary:

- no native windowing inside `prom-ui` core;
- no `winit` dependency in `prom-ui` core;
- no `NativeBackend::run_event_loop(...)` source change in this PR;
- no drawing;
- no frame presentation;
- no renderer integration;
- no `wgpu`;
- no draw backend selection;
- no hit testing;
- no interaction mapping;
- no direct Semantic action execution from native events;
- no direct effect execution from native events;
- no runtime handoff;
- no capability admission;
- no semantic state mutation;
- no verifier bypass.

## 9. Event Rules
Native events are platform facts.

Native events must not directly become Semantic actions.
Native events must not directly execute effects.
Native events must first become inert event metadata before any future
admission path.

Close request handling may request lifecycle exit from the native event loop,
but that does not grant Semantic action/effect authority.

## 10. Frame Rules
`UiBackendFrame` remains inert evidence.

Run-loop integration does not imply drawing.
Run-loop integration does not imply frame presentation.
Run-loop integration does not select a draw backend.
Run-loop integration must not rewrite render/projection/IR/layout evidence.

## 11. Error And Transcript Rules
Future run-loop integration must expose typed errors or explicit transcripts.

Required distinctions:

- missing staged window config;
- event-loop creation failure;
- native window creation failure;
- close requested;
- run completed;
- run failed;
- renderer not used;
- frame not presented.

Errors and transcripts are evidence, not semantic truth.

## 12. Future-Gated Work
Future gates:

- `R12-UI-WINIT-RUN-LOOP-INTEGRATION-SOURCE-PR`
  - may wire existing `winit` app state into normal backend lifecycle if this
    boundary is accepted.
- `R12-UI-WINIT-RUN-LOOP-GOLDEN-TRANSCRIPT-TEST-PR`
  - proves deterministic transcript behavior without drawing.
- `R12-UI-DRAW-BACKEND-SELECTION-PR`
  - selects draw backend through explicit dependency decision.
- `R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`
  - implements minimal draw behavior in backend-native layer only.
- `R12-UI-FRAME-PRESENTATION-BOUNDARY-PR`
  - defines when presentation becomes admitted behavior.

## 13. Explicit Non-Capabilities
This boundary does not mean:

- `NativeBackend::run_event_loop(...)` is integrated with `winit`;
- visible UI is complete;
- drawing exists;
- frame presentation exists;
- draw backend is selected;
- `wgpu` is approved;
- hit testing exists;
- interaction exists;
- native events are Semantic actions;
- action request admission exists;
- runtime handoff exists;
- effect handoff exists;
- capability admission exists.

## 14. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- Admission Guard changed: NO
- GitHub CI used: NO

## 15. Validation
- `pwsh -File scripts/local_ci.ps1`: PASS
- `cargo test -p prom-ui-backend-native --features winit-backend`: PASS
- `git diff --check`: PASS
- tracked `pr_body` files: NO

## 16. Remaining Warnings
Existing manual/ignored native window smoke paths remain baseline facts.

This boundary does not approve expanding them into normal lifecycle behavior
without a future explicit source gate.

## 17. Final Decision
PASS WITH WARNINGS - R12 UI Winit run-loop integration boundary defined.

This PR defines the future boundary for integrating existing crate-local
`winit` scaffolding with normal backend lifecycle.

It introduces no source code, tests, Cargo changes, new dependencies, drawing,
frame presentation, hit testing, interaction, runtime/action/effect behavior,
or capability admission.

The next recommended lane is
`R12-UI-WINIT-RUN-LOOP-INTEGRATION-SOURCE-PR`.

## 18. Recommended Next Lane
`R12-UI-WINIT-RUN-LOOP-INTEGRATION-SOURCE-PR`

Do not start it in this PR.
