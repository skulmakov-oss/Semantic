# R12 UI Draw Backend Selection

## 1. Purpose
This boundary document officially selects the draw backend for Semantic UI.
It resolves the previously deferred draw backend dependency gate.

It does not implement actual drawing, frame presentation, rendering logic, or runtime mutations.

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
| TBD  | Winit Run Loop Integration Source | MERGED |

## 3. Selected Draw Backend
The selected draw backend is **`wgpu`**.

Reasoning:
- `wgpu` provides the foundation for the cross-platform Rust GPU-accelerated graphics API. It is the canonical R12 draw backend foundation.
- `softbuffer` is deferred. It remains a possible future deterministic CPU/reference backend, not the canonical R12 draw backend, as it is a framebuffer path, not a scalable renderer foundation.
- `vello` is deferred. It remains a later 2D renderer candidate built on top of `wgpu`. Including it now would mix the backend selection boundary with renderer architecture.

## 4. SEMANTIC_UI_DNA Compliance
PASS - UI remains projection/cache, not semantic authority.
PASS - The renderer is treated as an inert output sink.
PASS - `wgpu` integration does not leak into `prom-ui` core.
PASS - `wgpu` does not mutate the Semantic AST, IR, or Runtime.
PASS - Action/Effect boundaries remain strictly unaffected.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- The backend crate may securely encapsulate `wgpu` types.
- Semantic projection contracts do not depend on `wgpu`.

## 5. Dependency Boundary Rules
- `prom-ui-backend-native` adds `wgpu` as an optional dependency behind the `wgpu-backend` feature.
- `prom-ui` core must **not** depend on `wgpu`.
- `prom-ui-runtime` must **not** depend on `wgpu`.

## 6. Forbidden Semantics
Forbidden in this boundary and immediate future core source gates:
- No rendering or drawing logic is implemented in this PR.
- No frame presentation or swapchain configuration is implemented in this PR.
- No semantic state mutation.
- No hit testing or interaction mapping.
- No execution of actions or effects.

## 7. Future-Gated Work
- `R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`
  - Implements the minimal `wgpu` draw behavior in the backend-native layer.
- `R12-UI-FRAME-PRESENTATION-BOUNDARY-PR`
  - Defines when and how physical pixels are swapped/presented.
- `R12-UI-STATIC-VISIBLE-DEMO-PR`
  - Runs a static visible demo without interaction.

## 8. Repository Scope
- source files changed: YES (Cargo.toml, lib.rs minimal scaffolds)
- test files changed: YES (new wgpu feature contract)
- docs changed: YES
- `Cargo.lock` changed: YES (wgpu dependency graph added)
- `docs/dna` changed: NO
- Admission Guard changed: NO

## 9. Validation
- `pwsh -File scripts/local_ci.ps1`: PASS
- `cargo test -p prom-ui-backend-native --features wgpu-backend`: PASS

## 10. Final Decision
PASS — R12 UI Draw Backend selected (`wgpu`).
The `wgpu-backend` feature and placeholder module establish the dependency boundary.

## 11. Recommended Next Lane
`R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`
