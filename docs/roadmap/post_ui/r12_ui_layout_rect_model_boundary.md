# R12 UI Layout Rect Model Boundary

## 1. Purpose
This boundary document starts DoD-2: Minimal Layout Ready.

It defines the future `UiLayoutRectModel` contract without implementing the model, solver, text measurement, backend, or runtime behavior.

## 2. Closed Basis
DoD-1 Foundation UI Ready is complete via `#1112`.

Foundation evidence chain `#1104` through `#1111` is already merged and closed by that closeout.

| PR | Role | Status |
|----|------|--------|
| #1112 | Foundation Capability Closeout | MERGED |

## 3. Boundary Summary
`UiLayoutRectModel` is future inert geometry evidence derived from `UiRenderModel`.

Future flow:

```text
UiRenderModel
  -> UiLayoutRectModel
```

The boundary is deterministic, integer-only, source-linked, non-authoritative, backend-free, runtime-free, and effect-free.

Layout rectangles are inert geometry evidence.
Layout rectangles do not decide semantic truth.
Layout rectangles do not execute rendering.
Layout rectangles do not execute actions or effects.
Layout rectangles do not perform runtime admission.
Layout rectangles do not introduce backend drawing.

## 4. SEMANTIC_UI_DNA Compliance
PASS - UI remains projection/cache, not semantic authority.
PASS - evidence/source references are preserved.
PASS - renderer boundary remains non-authoritative.
PASS - no direct action execution introduced.
PASS - no direct effect execution introduced.
PASS - no runtime/capability bypass introduced.
PASS - Unknown/Conflict semantics were not flattened.

The boundary is read as a contract only.
It does not elevate layout rectangles into semantic truth or execution authority.

## 5. Proposed Future Model Shape
This PR does not implement the model, but it defines the future source-shape expectation:

```rust
pub struct UiLayoutRectModel {
    entries: Vec<UiLayoutRectEntry>,
}

pub struct UiLayoutRectEntry {
    render_node_id: UiRenderNodeId,
    source_projection_node_id: UiProjectedNodeId,
    source_ir_node_id: Option<UiIrNodeId>,
    rect: UiLayoutRect,
}

pub struct UiLayoutRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}
```

No code is introduced in this PR.

## 6. Allowed Semantics
Allowed future semantics:

- store inert rectangles;
- link rectangles to render nodes;
- preserve render/projection/IR source evidence;
- use deterministic integer coordinates;
- represent absence of layout separately from zero-sized layout;
- support future golden rect tests;
- support future backend frame construction.

## 7. Forbidden Semantics
Forbidden in this boundary and in the immediate source model:

- no solver logic in this boundary PR;
- no flexbox/grid algorithm;
- no floats;
- no text measurement;
- no font access;
- no OS access;
- no backend draw commands;
- no windowing;
- no hit testing;
- no interaction;
- no action execution;
- no effect execution;
- no runtime handoff;
- no capability admission;
- no semantic truth mutation.

## 8. Future-Gated Work
Future gates remain separate and unchanged:

- `R12-UI-LAYOUT-RECT-MODEL-SOURCE-PR`
  - creates the data model only
- `R12-UI-LAYOUT-INPUT-CONTRACT-TEST-PR`
  - proves layout input contract
- `R12-UI-MINIMAL-BLOCK-LAYOUT-SOLVER-SOURCE-PR`
  - implements minimal deterministic solver
- `R12-UI-LAYOUT-GOLDEN-RECTS-TEST-PR`
  - proves exact hardcoded rect output
- `R12-UI-TEXT-MEASUREMENT-BOUNDARY-PR`
  - defines measurement boundary only

This PR does not implement any of them.

## 9. Integer Geometry Rule
Layout coordinates and sizes must be integer-only for DoD-2.

No floats are allowed in the initial `UiLayoutRectModel` source gate.

Preferred initial types:

- `x: i32`
- `y: i32`
- `width: u32`
- `height: u32`

Rationale:

- determinism
- golden test stability
- no platform float drift
- no premature solver dependency

## 10. Source Evidence Requirements
The future rect entry must preserve:

- `render_node_id`
- `source_projection_node_id`
- `source_ir_node_id` where available

It must not drop source evidence.

If source evidence is unavailable, the entry must represent that explicitly rather than silently erasing it.

## 11. Determinism Requirements
Same `UiRenderModel` input must produce the same `UiLayoutRectModel` output.

Sibling order must remain stable.
Nested order must remain stable.
Rect entry order must follow deterministic render model order unless explicitly documented otherwise.

## 12. Non-Capabilities
This boundary does not mean:

- layout source model exists yet;
- layout solver exists yet;
- text measurement exists yet;
- physical placement exists yet;
- visible UI exists yet;
- backend drawing exists yet;
- windowing exists yet;
- hit testing exists yet;
- interaction exists yet;
- action request admission exists yet;
- runtime handoff exists yet;
- effect handoff exists yet.

## 13. Repository Scope
source files changed: NO
test files changed: NO
docs changed: YES
Cargo.toml changed: NO
Cargo.lock changed: NO
docs/dna changed: NO
Admission Guard changed: NO
GitHub CI used: NO

## 14. Validation
Local validation:

- `cargo fmt --check`: PASS
- `cargo test -p prom-ui --lib`: PASS
- `cargo test -p prom-ui`: PASS
- `git diff --check`: PASS
- tracked `pr_body` files: NO
- Admission Guard executed: YES
- Admission Guard result: FAIL - ENVIRONMENT PATHING
- Admission Guard changed: NO

## 15. Final Decision
PASS WITH WARNINGS — R12 UI Layout Rect Model boundary defined.

DoD-2 Minimal Layout Ready is now open.

This PR defines the future `UiLayoutRectModel` boundary only. It introduces no source code, no tests, no layout solver, no text measurement, no backend behavior, no runtime behavior, no action/effect behavior, and no capability admission.

## 16. Recommended Next Lane
R12-UI-LAYOUT-RECT-MODEL-SOURCE-PR
