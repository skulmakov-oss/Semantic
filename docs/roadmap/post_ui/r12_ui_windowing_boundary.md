# R12 UI Windowing Boundary

## 1. Purpose
This boundary document defines the future windowing lifecycle boundary for Semantic UI.

It does not implement native windowing, winit integration, event loop behavior, surface creation, drawing, frame presentation, hit testing, interaction, runtime handoff, action execution, effect execution, or capability admission.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1121 | Backend Native Reality Audit | MERGED |
| #1122 | Backend Native Baseline Ledger | MERGED |
| #1123 | Backend Frame Sink Trait | MERGED |
| #1124 | Offscreen Static Frame Test | MERGED |

The closeout claim maps only to the merged basis above. No unsupported claims are made.

## 3. Boundary Summary
Windowing boundary is the future host boundary that allows a backend-native layer to own native window lifecycle without giving windowing semantic authority.

Future conceptual flow:

```text
UiBackendFrame
  -> UiFrameSink
  -> future WindowHost / WindowLifecycle boundary
  -> future native window / surface
```

This boundary is not a window implementation.
This boundary is not an event loop.
This boundary is not drawing.
This boundary is not frame presentation.
This boundary is not interaction.

## 4. SEMANTIC_UI_DNA Compliance
PASS - UI remains projection/cache, not semantic authority.
PASS - window lifecycle is a host boundary, not semantic authority.
PASS - window lifecycle input does not decide semantic truth.
PASS - frame evidence is preserved as inert evidence.
PASS - renderer/backend/runtime boundaries remain non-authoritative.
PASS - no direct action execution introduced.
PASS - no direct effect execution introduced.
PASS - no runtime/capability bypass introduced.
PASS - Unknown/Conflict semantics were not flattened.

This boundary document is an evidence record only.
It does not elevate windowing into semantic truth or execution authority.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- Boundary language stays within projection/evidence boundaries.
- Boundary does not elevate UI to authority, runtime, backend, or interaction ownership.
- Boundary preserves Unknown/Conflict visibility as required by the DNA.

DNA conflicts detected: none
DNA-driven constraints applied:
- no overclaiming of readiness beyond window lifecycle boundary definition
- no windowing implementation claim
- no backend/runtime/action/effect/capability claim
- no unsupported proof by assertion

## 5. Proposed Future Boundary Shape
This PR does not implement the model, but it defines the future source-shape expectation:

```rust
pub trait UiWindowHost {
    type Error;

    fn open(&mut self, config: UiWindowConfig) -> Result<(), Self::Error>;
    fn submit_frame(&mut self, frame: &UiBackendFrame) -> Result<(), Self::Error>;
    fn close(&mut self) -> Result<(), Self::Error>;
}

pub struct UiWindowConfig {
    title: &'static str,
    width: u32,
    height: u32,
}
```

No code is introduced in this PR.

The proposed shape must preserve:

- window lifecycle ownership in backend/native boundary
- inert `UiBackendFrame` evidence
- typed errors without semantic authority
- separation from `prom-ui` core models

## 6. Allowed Semantics
Allowed future semantics:

- own native window lifecycle inside backend-native boundary;
- receive inert `UiBackendFrame` evidence;
- present frame evidence to a native surface in a later gate;
- keep `prom-ui` core backend-free and windowing-free;
- keep semantic model ownership outside the window layer;
- keep frame evidence immutable from the window host perspective;
- isolate platform-specific behavior inside backend-native crate;
- keep window lifecycle errors typed and non-semantic.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future core source gates:

- no native windowing inside `prom-ui` core;
- no event loop inside `prom-ui` core;
- no direct action execution from window events;
- no direct effect execution from window events;
- no runtime/capability bypass;
- no semantic state mutation;
- no verifier bypass;
- no drawing behavior in this boundary PR;
- no frame presentation in this boundary PR;
- no hit testing in this boundary PR;
- no interaction mapping in this boundary PR;
- no `wgpu` in this boundary PR;
- no new `winit` usage in this boundary PR.

## 8. Window Lifecycle Rules
A future window host may own open/close lifecycle.
A future window host may own platform-specific native handles.
A future window host must remain outside `prom-ui` core.
A future window host must not mutate `UiTree`, `UiAst`, `UiIr`, `UiProjectionArtifact`, `UiRenderModel`, `UiLayoutRectModel`, or `UiBackendFrame`.
A future window host must not become semantic authority.

## 9. Frame Submission / Presentation Rules
`UiBackendFrame` remains inert frame evidence.

Submitting a frame to a future window host is not semantic execution.
Presenting a frame is future-gated.
Drawing a frame is future-gated.
No frame presentation is introduced by this PR.
No draw backend is selected by this PR.

## 10. Event Loop Rules
Event loop behavior is future-gated.
Native events must not execute Semantic actions directly.
Native events must not execute effects directly.
Native events must become inert event metadata before any admission path.
Input/event handling remains out of scope until interaction gates.

## 11. Dependency Boundary Rules
`prom-ui` core must not depend on `winit`.
`prom-ui` core must not depend on `prom-ui-backend-native`.
`prom-ui` core must not depend on GPU/windowing/drawing crates.
Existing `winit` usage remains crate-local to `prom-ui-backend-native` baseline.
New `winit` expansion requires `R12-UI-WINIT-WINDOW-SEED-PR`.
`wgpu` is not approved by this boundary.
Draw backend selection remains future-gated.

## 12. Source Evidence Rules
Window lifecycle must not erase frame evidence.
Window lifecycle must not rewrite render/projection/IR/layout evidence.
Window lifecycle must not infer semantic truth from geometry.
Missing evidence must remain explicit in frame-level contracts.

## 13. Future-Gated Work
- `R12-UI-WINIT-WINDOW-SEED-PR`
  - seeds minimal window lifecycle inside `prom-ui-backend-native` only
- `R12-UI-DRAW-BACKEND-SELECTION-PR`
  - selects draw backend through explicit dependency decision
- `R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`
  - implements minimal draw behavior in backend-native layer
- `R12-UI-STATIC-VISIBLE-DEMO-PR`
  - runs a static visible demo without interaction
- `R12-UI-RAW-EVENT-CAPTURE-BOUNDARY-PR`
  - starts event capture boundary later, not in DoD-3 static phase

## 14. Explicit Non-Capabilities
This boundary does not mean:

- native windowing is implemented;
- `winit` window seed is complete;
- event loop exists;
- frame presentation exists;
- drawing backend is selected;
- `wgpu` is approved;
- visible UI is complete;
- hit testing exists;
- interaction exists;
- action request admission exists;
- runtime handoff exists;
- effect handoff exists;
- capability admission exists.

## 15. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- `Admission Guard` changed: NO
- GitHub CI used: NO

## 16. Validation
- `cargo fmt --check`: PASS
- `cargo test -p prom-ui --lib`: PASS
- `cargo test -p prom-ui`: PASS
- `cargo test -p prom-ui-backend-native`: PASS
- `cargo test -p prom-ui-backend-native --test offscreen_static_frame`: PASS
- `git diff --check`: PASS
- tracked `pr_body` files: NO
- Admission Guard executed: YES
- Admission Guard result: FAIL - ENVIRONMENT PATHING
- Admission Guard changed: NO

## 17. Remaining Warnings
Admission Guard still fails locally due environment pathing.
This is unchanged and does not reflect a repository regression.

Cargo test emits unrelated existing warnings from other test files, but no failures.

## 18. Final Decision
PASS WITH WARNINGS — R12 UI Windowing boundary defined.

This PR defines the future window lifecycle boundary only.

It introduces no source code, no tests, no Cargo changes, no new `winit` usage, no `wgpu`, no drawing, no frame presentation, no event loop behavior, no hit testing, no interaction, no runtime/action/effect behavior, and no capability admission.

The next recommended lane is `R12-UI-WINIT-WINDOW-SEED-PR`.

## 19. Recommended Next Lane
`R12-UI-WINIT-WINDOW-SEED-PR`

Do not start it in this PR.
