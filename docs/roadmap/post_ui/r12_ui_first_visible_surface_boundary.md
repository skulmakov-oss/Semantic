# R12 UI First Visible Surface Boundary

## 1. Purpose
This boundary document starts DoD-3: Visible Static UI Ready.

It defines the future visible surface boundary for Semantic UI.

It does not implement a backend, native window, drawing backend, frame sink, hit testing, event loop, interaction, runtime handoff, action/effect execution, or capability admission.

## 2. Closed Basis
DoD-1 Foundation UI Ready is complete through `#1112`.
DoD-2 Minimal Layout Ready is complete through `#1119`.

| PR | Role | Status |
|----|------|--------|
| #1112 | Foundation Capability Closeout | MERGED |
| #1113 | Layout Rect Model Boundary | MERGED |
| #1114 | Layout Rect Model Source | MERGED |
| #1115 | Layout Input Contract Test | MERGED |
| #1116 | Minimal Block Layout Solver Source | MERGED |
| #1117 | Layout Golden Rects Test | MERGED |
| #1118 | Text Measurement Boundary | MERGED |
| #1119 | Minimal Layout Capability Closeout | MERGED |

The closeout claim maps only to the merged basis above. No unsupported claims are made.

## 3. Boundary Summary
First Visible Surface Boundary defines how inert UI evidence can cross from `prom-ui` core into a future backend/native visible surface layer.

Future flow:

```text
UiRenderModel
  + UiLayoutRectModel
  -> UiVisibleSurfaceFrame / UiBackendFrame evidence
  -> future backend/native frame sink
```

The boundary is deterministic, source-linked, layout-linked, backend-free in `prom-ui` core, runtime-free, effect-free, and non-authoritative.

This boundary is not the backend.
This boundary is not drawing.
This boundary is not windowing.
This boundary is not visible UI implementation.
It is the contract that future visible surface work must obey.

## 4. SEMANTIC_UI_DNA Compliance
PASS - UI remains projection/cache, not semantic authority.
PASS - visible surface/frame evidence is inert projection evidence.
PASS - visible surface input does not decide semantic truth.
PASS - source references are preserved.
PASS - renderer/backend/runtime boundaries remain non-authoritative.
PASS - no direct action execution introduced.
PASS - no direct effect execution introduced.
PASS - no runtime/capability bypass introduced.
PASS - Unknown/Conflict semantics were not flattened.

This closeout is an evidence record only.
It does not elevate visible surface evidence into semantic truth or execution authority.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- Closeout language stays within projection/evidence boundaries.
- Closeout does not elevate UI to authority, runtime, backend, or interaction ownership.
- Closeout preserves Unknown/Conflict visibility as required by the DNA.

DNA conflicts detected: none
DNA-driven constraints applied:
- no overclaiming of readiness beyond DoD-3 boundary definition
- no visible UI implementation claim
- no backend/runtime/action/effect/capability claim
- no unsupported proof by assertion

## 5. Proposed Future Boundary Shape
This PR does not implement the model, but it defines the future source-shape expectation:

```rust
pub struct UiVisibleSurfaceInput {
    render_model: UiRenderModel,
    layout_rect_model: UiLayoutRectModel,
}

pub struct UiVisibleSurfaceFrame {
    entries: Vec<UiVisibleSurfaceEntry>,
}

pub struct UiVisibleSurfaceEntry {
    render_node_id: UiRenderNodeId,
    source_projection_node_id: UiProjectedNodeId,
    source_ir_node_id: Option<UiIrNodeId>,
    rect: UiLayoutRect,
    kind: UiRenderNodeKind,
}

pub trait UiFrameSink {
    fn submit_frame(&mut self, frame: UiVisibleSurfaceFrame);
}
```

No code is introduced in this PR.

The proposed shape must preserve:

- render node identity
- projection source evidence
- IR source evidence where available
- layout rect evidence
- render node kind

## 6. Allowed Semantics
Allowed future semantics:

- collect render node evidence and layout rect evidence into a frame-like artifact;
- preserve render/projection/IR/layout source evidence;
- represent frame entries in deterministic order;
- provide an inert frame to a backend/native crate later;
- allow a future backend crate to draw based on frame evidence;
- allow future offscreen frame tests;
- keep `prom-ui` core backend-free;
- keep `prom-ui` core windowing-free;
- keep `prom-ui` core runtime-free.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future core source gates:

- no native window creation inside `prom-ui` core;
- no backend draw commands inside `prom-ui` core;
- no GPU API access inside `prom-ui` core;
- no OS event loop inside `prom-ui` core;
- no `winit` inside `prom-ui` core;
- no `wgpu` inside `prom-ui` core;
- no Vello / tiny-skia / softbuffer dependency inside `prom-ui` core unless a future dependency gate explicitly allows it;
- no hit testing;
- no interaction;
- no action execution;
- no effect execution;
- no runtime handoff;
- no capability admission;
- no semantic truth mutation;
- no direct verifier bypass;
- no platform-dependent nondeterminism in `prom-ui` core.

## 8. Determinism Requirements
Same `UiRenderModel` + `UiLayoutRectModel` input must produce the same future frame evidence.

Frame entry order must be deterministic.
Frame entry order should follow render model order unless a future source gate explicitly documents another stable order.
Missing layout evidence must be explicit.
No platform-dependent frame construction may leak into `prom-ui` core.

## 9. Source Evidence Requirements
Future frame evidence must preserve:

- `render_node_id`
- `source_projection_node_id`
- `source_ir_node_id` where available
- layout rect evidence
- render node kind

It must not drop source evidence.

If source evidence or layout evidence is unavailable, frame evidence must represent that explicitly, not silently erase it.

## 10. Backend Boundary Rules
`prom-ui` core may define inert frame evidence in a future source gate.

`prom-ui` core must not depend on native windowing crates.
`prom-ui` core must not depend on GPU/backend draw crates.
`prom-ui` core must not submit OS windows or GPU commands.

backend/native crates may later implement frame sinks outside `prom-ui` core.
backend/native crates must not mutate `UiTree`, `UiAst`, `UiIr`, `UiProjectionArtifact`, `UiRenderModel`, or `UiLayoutRectModel`.
backend/native crates receive frame evidence; they do not become semantic authority.

## 11. Frame Evidence Rules
A future `UiVisibleSurfaceFrame` is evidence, not execution.

A future frame may describe what could be drawn.
A future frame must not itself draw.
A future frame must not dispatch actions.
A future frame must not execute effects.
A future frame must not perform runtime admission.
A future frame must not mutate semantic state.

## 12. Future-Gated Work
Future gates remain separate and unchanged:

- `R12-UI-BACKEND-NATIVE-CRATE-SEED-PR`
  - creates backend/native crate scaffold only
- `R12-UI-BACKEND-FRAME-SINK-TRAIT-PR`
  - defines inert frame sink contract
- `R12-UI-OFFSCREEN-STATIC-FRAME-TEST-PR`
  - proves frame construction offscreen without native window
- `R12-UI-WINDOWING-BOUNDARY-PR`
  - defines window lifecycle boundary only
- `R12-UI-WINIT-WINDOW-SEED-PR`
  - seeds minimal windowing outside `prom-ui` core
- `R12-UI-DRAW-BACKEND-SELECTION-PR`
  - selects drawing backend through explicit dependency gate
- `R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`
  - implements minimal drawing in backend/native layer
- `R12-UI-STATIC-VISIBLE-DEMO-PR`
  - runs a static visible demo without interaction

This PR does not implement any of them.

## 13. Non-Capabilities
This boundary does not mean:

- visible UI exists yet;
- backend/native crate exists yet;
- frame sink trait exists yet;
- offscreen frame tests exist yet;
- native windowing exists yet;
- `winit` integration exists yet;
- drawing backend exists yet;
- GPU drawing exists yet;
- software rasterization exists yet;
- hit testing exists yet;
- interaction exists yet;
- event loop exists yet;
- action request admission exists yet;
- runtime handoff exists yet;
- effect handoff exists yet;
- capability admission exists yet.

## 14. Semantic Test Position
Current gate uses documentation because the visible surface boundary is not implemented yet.

No Semantic executable tests are added in this gate.

Future-gated Semantic contract work should define meaning-level invariants such as:

- visible surface evidence is not semantic truth;
- frame evidence does not draw by itself;
- backend/native drawing must not mutate semantic models;
- rendering must not execute effects;
- visible UI must not bypass verifier/runtime/capability gates;
- interaction must remain inert until admitted by runtime boundary.

This section prevents forgetting the Semantic layer, but does not create fake Semantic tests.

## 15. Repository Scope
source files changed: NO
test files changed: NO
docs changed: YES
Cargo.toml changed: NO
Cargo.lock changed: NO
docs/dna changed: NO
Admission Guard changed: NO
GitHub CI used: NO

## 16. Validation
Local validation:

- `cargo fmt --check`: PASS
- `cargo test -p prom-ui --lib`: PASS
- `cargo test -p prom-ui`: PASS
- `git diff --check`: PASS
- tracked `pr_body` files: NO
- Admission Guard executed: YES
- Admission Guard result: FAIL - ENVIRONMENT PATHING
- Admission Guard changed: NO

## 17. Final Decision
PASS WITH WARNINGS — R12 UI First Visible Surface boundary defined.

This PR starts DoD-3 Visible Static UI Ready by defining the future boundary between inert UI/layout evidence and a future visible surface/backend layer.

It introduces no source code, no tests, no backend crate, no windowing, no drawing backend, no hit testing, no interaction, no runtime behavior, no action/effect behavior, and no capability admission.

## 18. Recommended Next Lane
R12-UI-BACKEND-NATIVE-CRATE-SEED-PR
