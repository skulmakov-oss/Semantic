# R12 UI Draw Backend Selection Boundary

## 1. Purpose
This boundary document defines the selection of `wgpu` as the sole authorized minimal draw backend for the UI framework inside the `prom-ui-backend-native` crate.

Status note:
the repository now contains a feature-gated native WGPU path and a dedicated reality audit at
[r12_ui_native_wgpu_renderer_reality_audit.md](./r12_ui_native_wgpu_renderer_reality_audit.md).
Treat this document as the boundary that governs the future draw contract, not as evidence that WGPU is absent.

It does not implement any new source code.
It does not change tests, Cargo features, dependencies, or Admission Guard.
It does not introduce draw execution, frame presentation, or visual output logic.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1123 | Backend Frame Sink Trait | MERGED |
| #1132 | Winit Window Seed Reality Ledger | MERGED |
| #1133 | Winit Run Loop Integration Boundary | MERGED |
| #1134 | Winit Run Loop Integration Source | MERGED |

## 3. Boundary Summary
The native backend layer (`prom-ui-backend-native`) has fully integrated windowing (`winit`). To draw actual pixels on the screen, a rendering backend must be selected.

Current state:
- The `prom-ui-backend-native` crate possesses an inert `wgpu-backend` feature and baseline initialization context (`NativeBackendWgpuContext`).
- No draw logic uses it.
- `prom-ui` core has no rendering backend.

Future boundary under review:
- `wgpu` is confirmed as the draw backend.
- `prom-ui` core remains fully abstracted and isolated from `wgpu`.
- Draw execution logic will be restricted exclusively to `prom-ui-backend-native`.
- All drawing instructions from the core will be sent via neutral boundary representations (`UiBackendFrame`).

## 4. SEMANTIC_UI_DNA Compliance
PASS - UI core remains entirely separated from native graphics layers.
PASS - `wgpu` is confined to the backend layer.
PASS - Backend frame evidence handles abstraction properly.

DNA conflicts detected: none.

## 5. Existing Baseline Facts
Existing repository facts from the accepted baseline:
- `wgpu-backend` feature exists in `prom-ui-backend-native`.
- `wgpu` dependency exists.
- `NativeBackendWgpuContext` skeleton exists.
- `prom-ui` core has zero `wgpu` or `winit` dependencies.
- The feature-gated native WGPU path is already present in the backend-native crate and is documented separately by the native WGPU reality audit.

## 6. Ownership Boundary
This selection designates that:
- `prom-ui-backend-native` owns `wgpu` lifecycle (Instance, Adapter, Device, Queue, Surface).
- `prom-ui` core owns neutral visual representation metadata.
- `prom-ui` core MUST NOT import or export `wgpu` types.

## 7. Allowed Future Semantics
Allowed future semantics, if admitted by a later source PR:
- Implementing an actual render pass executing neutral frame instructions.
- Writing to a swapchain surface.
- Submitting command buffers to a queue.

## 8. Forbidden Semantics
Forbidden by this boundary:
- No `wgpu` inside `prom-ui` core.
- No direct source code changes inside this PR.
- No drawing output inside this PR.
- No frame presentation.

## 9. Future-Gated Work
Future gates:
- `R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`
  - Implements minimal wgpu-based rendering inside `prom-ui-backend-native` to draw clear colors or basic rectangles.
- `R12-UI-FRAME-PRESENTATION-BOUNDARY-PR`
  - Defines rules for presenting final frames to the OS surface.

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
PASS - R12 UI Draw Backend Selection Boundary defined.

This document confirms `wgpu` as the rendering choice for `prom-ui-backend-native` while guaranteeing the pure, abstract nature of `prom-ui` core is preserved.

## 12. Recommended Next Lane
`R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`

Do not start it in this PR.
