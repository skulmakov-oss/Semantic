# R12 UI Frame Presentation Boundary

## 1. Purpose
This boundary document officially defines the rules and lifecycle expectations for integrating physical pixel presentation (via `wgpu::Surface` swapchains) into the `prom-ui-backend-native` layer.

It does not implement surface creation, swapchain configuration, or frame presentation.
It introduces no source code, tests, Cargo changes, dependencies, or runtime mutations.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1122 | Backend Native Baseline Ledger | MERGED |
| #1123 | Backend Frame Sink Trait | MERGED |
| #1124 | Offscreen Static Frame Test | MERGED |
| #1125 | Windowing Boundary | MERGED |
| #1132 | Winit Window Seed Reality Ledger | MERGED |
| #1133 | Winit Run Loop Integration Boundary | MERGED |
| #1134 | Draw Backend Selection | MERGED |
| #1135 | Draw Backend Minimal Source | MERGED |

## 3. Boundary Summary
With the minimal `wgpu` initialization primitives and the manual `winit` event loop scaffolding successfully merged, the next architectural threshold is configuring a `wgpu::Surface` to target a real native `Window` and presenting rendered pixels.

This boundary dictates that frame presentation belongs entirely to the `prom-ui-backend-native` layer. The core semantic model remains entirely decoupled from presentation surfaces, swapchains, and physical pixel constraints.

## 4. SEMANTIC_UI_DNA Compliance
PASS - UI remains projection/cache, not semantic authority.
PASS - `prom-ui` core remains oblivious to swapchain and physical surfaces.
PASS - `prom-ui-runtime` remains platform-neutral.
PASS - Presentation lifecycle does not mutate semantic state.
PASS - Draw and surface errors are treated as evidence, not semantic faults.
PASS - Unknown/Conflict semantics are not flattened.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- Presentation and surface boundaries remain purely inert output sinks.
- Surface/swapchain errors (e.g. Outdated/Lost) must be gracefully handled (reconfigured) without asserting semantic failure or crashing the Semantic UI runtime.

## 5. Proposed Future Source Shape
The future source gate must implement:
- Safe creation of `wgpu::Surface<'static>` from a winit `Window`.
- Configuration of the swapchain using window extents and presentation modes.
- `surface.get_current_texture()` acquisition in the draw loop.
- Presentation of the texture back to the physical screen.
- Graceful recovery from `wgpu::SurfaceError` (e.g., `Outdated`, `Lost`).

This logic must be contained strictly within the backend-native crate.

## 6. Allowed Semantics
Allowed future semantics, if admitted by a later source PR:
- Create a `wgpu::Surface` from a winit `Window`.
- Call `surface.configure(...)` to set up swapchains.
- Render actual pixels to the acquired surface texture.
- Gracefully recreate the swapchain upon window resize.
- Log or record presentation faults as diagnostic evidence.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future source gates:
- No surface configuration or presentation logic is implemented in this PR.
- No `wgpu::Surface` types may leak into `prom-ui` core.
- No semantic execution or action routing during frame presentation.
- `SurfaceError::OutOfMemory` or other fatal rendering faults must not corrupt or rewrite the `UiAst` or `UiTree`.
- No interaction mapping or hit testing yet.

## 8. Surface Error and Resilience Rules
Presentation faults are physical, not semantic. 
A failed frame presentation (due to a minimized window or lost surface) is not a semantic failure. 
The backend must gracefully skip rendering, reconfigure the surface, or record the fault without halting the Semantic UI runtime or corrupting the inert `UiBackendFrame` evidence.

## 9. Dependency Boundary Rules
- `wgpu` usage remains localized to the `prom-ui-backend-native` crate.
- `prom-ui` core continues to treat the backend solely via `UiFrameSink`.

## 10. Future-Gated Work
- `R12-UI-FRAME-PRESENTATION-SOURCE-PR`
  - Implements the surface creation and swapchain logic in `prom-ui-backend-native`.
- `R12-UI-STATIC-VISIBLE-DEMO-PR`
  - Runs a static visible demo using the newly integrated presentation surface without interaction.
- `R12-UI-WINIT-RUN-LOOP-INTEGRATION-SOURCE-PR`
  - Integrates the presentation logic directly into the normal event loop lifecycle.

## 11. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- Admission Guard changed: NO
- GitHub CI used: NO

## 12. Final Decision
PASS — R12 UI Frame Presentation Boundary defined.

This PR defines the future boundary for physical pixel presentation.
It introduces no source code, tests, Cargo changes, drawing implementation, interaction mapping, or capability admission.

## 13. Recommended Next Lane
`R12-UI-FRAME-PRESENTATION-SOURCE-PR`

Do not start it in this PR.
