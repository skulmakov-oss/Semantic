# R12 UI Renderer Public API Lock Closeout

## Line
R12-UI-RENDERER-PUBLIC-API-LOCK-LINE-FULL-PACKAGE

## Goal
Complete the Renderer Public API Lock line for the inert R12 renderer seed.

## Boundary State
The renderer module now exposes a locked public API via integration tests without expanding behavior or adding rendering authority.

### Locked Signatures
```rust
pub fn render_projection_to_model(projection: &UiProjectionArtifact) -> Result<UiRenderModel, UiRenderError>
```

### Locked Types
- `UiRenderModel`
- `UiRenderNode`
- `UiRenderModelId`
- `UiRenderNodeId`
- `UiRenderMarker`
- `UiRenderNodeKind`
- `UiRenderError`

### Asserted Constraints
- No backend (WGPU, winit, Tauri)
- No layout/draw/event code
- No event dispatch mechanism
- No runtime/verifier/VM capability admission
- `UiRenderMarker` remains inert without execution/handling functions
- Models and Nodes are immutable/read-only and structurally deterministic

## Closed PRs
- #947 — test(ui): lock renderer public api

## Conclusion
The R12 UI Renderer Public API Lock line is complete.
The inert downstream seed is structurally locked.
