# R12 UI Frame Presentation Boundary

## 1. Purpose
This boundary document defines the architectural rules for presenting semantic frame instructions from the `prom-ui` core layer to the physical `wgpu` layer within `prom-ui-backend-native`.

It establishes the bridge between abstract rendering metadata (`UiBackendFrame`) and the hardware rendering queue.

It does not implement any new source code.
It does not change tests, Cargo features, dependencies, or Admission Guard.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1123 | Backend Frame Sink Trait | MERGED |
| #1133 | Winit Run Loop Integration Boundary | MERGED |
| #1134 | Winit Run Loop Integration Source | MERGED |
| #1135 | Draw Backend Selection Boundary | MERGED |
| #1136 | Minimal Draw Backend Source Integration | MERGED |

## 3. Boundary Summary
The native backend (`prom-ui-backend-native`) has fully instantiated its drawing apparatus (`NativeBackendWgpuContext` and `NativeBackendPresentationSurface`). Currently, it issues static clear passes independent of `prom-ui-runtime`.

To complete the pipeline, the `UiBackendAdapter::present_frame` interface must be activated so the core `RuntimeStateUpdater` can hand over semantic frames dynamically.

Future boundary under review:
- The core runtime remains agnostic to the rendering API, generating only abstract `DrawFrame` outputs.
- `NativeBackend` maps the incoming `DrawFrame` into specific `wgpu` RenderPass instructions.
- All drawing operations originating from semantic user data are driven by this interface.

## 4. SEMANTIC_UI_DNA Compliance
PASS - Core UI logic stays isolated.
PASS - Backend frame parsing correctly transforms semantic intent into native pixel instructions without backwards dependency.

DNA conflicts detected: none.

## 5. Existing Baseline Facts
Existing repository facts from the accepted baseline:
- `prom-ui-backend-native` implements `UiBackendAdapter`.
- `UiBackendAdapter` possesses `present_frame(&mut self, frame: DrawFrame)`.
- The `wgpu-backend` effectively configures and manages swapchain rendering.

## 6. Ownership Boundary
This selection designates that:
- `prom-ui-runtime` owns deciding *when* a frame should be rendered and *what* it semantically contains.
- `prom-ui-backend-native` owns decoding the semantic instructions into a batch of hardware commands.

## 7. Allowed Future Semantics
Allowed future semantics, if admitted by a later source PR:
- Implementing `UiBackendAdapter::present_frame` within `NativeBackend` to intercept `DrawFrame` and trigger `present_minimal_clear` or a future generalized `present_semantic_frame` sequence.
- Dispatching the received instructions directly into the `wgpu` command queue using the `NativeBackendPresentationSurface`.

## 8. Forbidden Semantics
Forbidden by this boundary:
- Creating hardware dependencies inside `prom-ui-runtime`.
- Mutating semantic structures back from the native layer (the pipeline is strictly uni-directional at this phase).
- Generating layout positions physically inside the backend renderer (layout remains abstract).

## 9. Future-Gated Work
Future gates:
- `R12-UI-FRAME-PRESENTATION-SOURCE-PR`
  - Instantiates `present_frame` using the initialized `wgpu` context.

## 10. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- Admission Guard changed: NO
- GitHub CI used: NO

## 11. Final Decision
PASS - R12 UI Frame Presentation Boundary defined.

This document formally clears the path to connect the abstract `prom-ui` rendering pipeline to the actual hardware renderer via `UiBackendAdapter::present_frame`.

## 12. Recommended Next Lane
`R12-UI-FRAME-PRESENTATION-SOURCE-PR`

Do not start it in this PR.
