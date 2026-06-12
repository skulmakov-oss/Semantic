## Summary

Adds the R12 UI Renderer Diagnostics Presentation seed.

This introduces inert renderer-local diagnostics presentation structures over `UiRenderModel`.

It does not rewrite verifier diagnostics, call runtime/verifier/VM systems, execute actions, authorize effects, dispatch events, or implement backend rendering.

## Closed basis

- #943 — R12 UI Renderer Boundary
- #944 — R12 UI Renderer Boundary Closeout
- #945 — R12 UI Renderer Seed
- #946 — R12 UI Renderer Seed Closeout
- #947 — R12 UI Renderer Public API Lock
- #948 — R12 UI Renderer Public API Lock Closeout
- #949 — R12 UI Renderer Full-Line Ledger Audit

## Scope

Implemented:

- inert diagnostics presentation model
- diagnostics presentation item model
- deterministic presentation identity
- deterministic item identity
- read-only `UiRenderModel` consumption
- source render model/projection preservation
- inert marker-to-presentation mapping
- tests and API signature locks

Preserved:

- renderer remains downstream and inert
- no backend
- no WGPU/winit/Tauri
- no layout/draw/event
- no event dispatch
- no runtime/verifier/VM
- no capability admission
- no Workbench/Studio

## Explicit non-scope

- no projection.rs changes
- no validation.rs changes
- no Cargo.toml / Cargo.lock changes
- no dependency additions
- no backend rendering
- no draw/layout/event implementation
- no verifier diagnostic rewriting
- no semantic truth authority

## Project #2 metadata

```text
Track: POST-UI
Wave: R12
Type: Code
Risk: High
Boundary: Renderer
Gate: PRReady
Evidence: PR
Depends on: #949
```

## Validation

Local only:

```text
cargo fmt --check: PASS
cargo test -p prom-ui --lib: PASS
cargo test -p prom-ui: PASS
git diff --check: PASS
GitHub CI used as evidence: NO
```

## Admission Guard

| Area                                | Observed state | Classification | Status |
| ----------------------------------- | -------------- | -------------- | ------ |
| diagnostics presentation model      | Implemented    | ADMITTED       | PASS   |
| read-only UiRenderModel consumption | Implemented    | ADMITTED       | PASS   |
| diagnostic authority                | Absent         | FORBIDDEN      | PASS   |
| verifier diagnostic rewriting       | Absent         | FORBIDDEN      | PASS   |
| backend/WGPU/winit/Tauri            | Absent         | FORBIDDEN      | PASS   |
| layout/draw/event                   | Absent         | FORBIDDEN      | PASS   |
| event dispatch                      | Absent         | FORBIDDEN      | PASS   |
| runtime/verifier/VM                 | Absent         | FORBIDDEN      | PASS   |
| capability admission                | Absent         | FORBIDDEN      | PASS   |
| Workbench/Studio                    | Absent         | FORBIDDEN      | PASS   |

