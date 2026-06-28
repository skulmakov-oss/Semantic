# R12 UI Text Measurement Boundary

## 1. Purpose
This boundary document defines the future text measurement boundary for Semantic UI.

It does not implement text measurement, font access, shaping, rendering, backend integration, or layout solver changes.

This is the final boundary item required before closing DoD-2 Minimal Layout Ready.

## 2. Closed Basis
DoD-2 layout rect model, layout input contract, minimal block solver, and golden rect tests are complete through `#1117`.

| PR | Role | Status |
|----|------|--------|
| #1113 | Layout Rect Model Boundary | MERGED |
| #1114 | Layout Rect Model Source | MERGED |
| #1115 | Layout Input Contract Test | MERGED |
| #1116 | Minimal Block Layout Solver Source | MERGED |
| #1117 | Layout Golden Rects Test | MERGED |

## 3. Boundary Summary
`TextMeasureProvider` is a future boundary that supplies deterministic text measurement evidence to layout code.

Future flow:

```text
UiRenderModel text evidence
  -> TextMeasureProvider
  -> UiTextMeasurement evidence
  -> UiLayoutRectModel / future solver input
```

The boundary is deterministic, integer-only in the initial contract, source-linked, backend-free at the `prom-ui` core boundary, runtime-free, effect-free, and non-authoritative.

Text measurement is inert geometry evidence.
Text measurement does not decide semantic truth.
Text measurement does not execute rendering.
Text measurement does not access native fonts from `prom-ui` core.
Text measurement does not access OS APIs from `prom-ui` core.
Text measurement does not execute actions.
Text measurement does not execute effects.
Text measurement does not perform runtime admission.
Text measurement does not introduce backend drawing.

## 4. SEMANTIC_UI_DNA Compliance
PASS - UI remains projection/cache, not semantic authority.
PASS - evidence/source references are preserved.
PASS - renderer boundary remains non-authoritative.
PASS - no direct action execution introduced.
PASS - no direct effect execution introduced.
PASS - no runtime/capability bypass introduced.
PASS - Unknown/Conflict semantics were not flattened.

This boundary is read as a contract only.
It does not elevate text measurement into semantic truth, execution authority, or platform ownership.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- Boundary language stays inside projection/evidence boundaries.
- Boundary does not move font, OS, backend, runtime, or capability ownership into `prom-ui` core.
- Boundary preserves explicit Unknown/Conflict handling as required by the DNA.

DNA conflicts detected: none
DNA-driven constraints applied:
- no overclaiming of measurement readiness
- no font/backend/runtime/action/effect/capability claim
- no unsupported proof by assertion

## 5. Proposed Future Boundary Shape
This PR does not implement the boundary, but it defines the future source-shape expectation:

```rust
pub trait TextMeasureProvider {
    fn measure_text(&self, request: UiTextMeasureRequest) -> UiTextMeasurement;
}

pub struct UiTextMeasureRequest {
    render_node_id: UiRenderNodeId,
    source_projection_node_id: UiProjectedNodeId,
    source_ir_node_id: Option<UiIrNodeId>,
    text_len: usize,
    constraints: UiTextMeasureConstraints,
}

pub struct UiTextMeasurement {
    render_node_id: UiRenderNodeId,
    source_projection_node_id: UiProjectedNodeId,
    source_ir_node_id: Option<UiIrNodeId>,
    width: u32,
    height: u32,
    baseline: Option<i32>,
}
```

No code is introduced in this PR.

## 6. Allowed Semantics
Allowed future semantics:

- request inert text measurement evidence;
- store measured width/height as deterministic integer evidence;
- preserve render/projection/IR source evidence;
- represent absence of measurement explicitly;
- support future golden measurement tests;
- support future layout solver input;
- allow backend/native crates to provide measurement through a boundary trait later.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future core source gate:

- no native font access inside `prom-ui` core;
- no OS text measurement inside `prom-ui` core;
- no glyph rasterization inside `prom-ui` core;
- no shaping engine inside `prom-ui` core;
- no backend draw commands;
- no windowing;
- no hit testing;
- no interaction;
- no action execution;
- no effect execution;
- no runtime handoff;
- no capability admission;
- no semantic truth mutation;
- no floating-point measurement in the initial boundary;
- no hidden platform-dependent measurement.

## 8. Determinism Requirements
Same request must produce the same measurement under the same provider.

Measurement output must be stable for golden tests.

No platform-dependent nondeterminism may leak into `prom-ui` core.

Initial measurement evidence must use integer dimensions.

If future native measurement is needed, it must be isolated behind a backend/native boundary and golden-tested separately.

## 9. Source Evidence Requirements
Future measurement evidence must preserve:

- `render_node_id`
- `source_projection_node_id`
- `source_ir_node_id` where available

It must not drop source evidence.

If source evidence is unavailable, measurement evidence must represent that explicitly rather than silently erasing it.

## 10. Provider Boundary Rules
`prom-ui` core may depend on an abstract provider trait in a future source gate.

`prom-ui` core must not depend on native font libraries.

`prom-ui` core must not call OS measurement APIs.

`prom-ui` core must not depend on backend/windowing crates.

backend/native crates may later implement provider traits outside `prom-ui` core.

## 11. Future-Gated Work
Future gates remain separate and unchanged:

- `R12-UI-TEXT-MEASUREMENT-MODEL-SOURCE-PR`
  - creates request/measurement data model only
- `R12-UI-TEXT-MEASURE-PROVIDER-TRAIT-SOURCE-PR`
  - defines provider trait only
- `R12-UI-TEXT-MEASURE-GOLDEN-TEST-PR`
  - proves deterministic fake/provider output
- `R12-UI-LAYOUT-SOLVER-TEXT-INPUT-INTEGRATION-PR`
  - integrates measurement evidence into layout solver input
- native/provider implementation gates
  - remain outside `prom-ui` core or behind strict boundary

This PR does not implement any of them.

## 12. Non-Capabilities
This boundary does not mean:

- text measurement source model exists yet;
- `TextMeasureProvider` trait exists yet;
- any font access exists yet;
- any OS measurement exists yet;
- any shaping engine exists yet;
- any glyph rasterization exists yet;
- layout solver uses text metrics yet;
- visible UI exists yet;
- backend drawing exists yet;
- windowing exists yet;
- hit testing exists yet;
- interaction exists yet;
- action request admission exists yet;
- runtime handoff exists yet;
- effect handoff exists yet.

## 13. Semantic Test Position
Current gate uses documentation because the text measurement boundary is not implemented yet.

No Semantic executable tests are added in this gate.

Future-gated Semantic contract work should define meaning-level invariants such as:

- text measurement evidence is not semantic truth;
- absence of measurement is not zero measurement;
- platform-specific measurement must not leak into core authority;
- rendering must not execute effects;
- layout must not bypass verifier/runtime/capability gates.

This section prevents forgetting the Semantic layer, but does not create fake Semantic tests.

## 14. Repository Scope
source files changed: NO
test files changed: NO
docs changed: YES
Cargo.toml changed: NO
Cargo.lock changed: NO
docs/dna changed: NO
Admission Guard changed: NO
GitHub CI used: NO

## 15. Validation
Local validation:

- `cargo fmt --check`: PASS
- `cargo test -p prom-ui --lib`: PASS
- `cargo test -p prom-ui`: PASS
- `git diff --check`: PASS
- tracked `pr_body` files: NO
- Admission Guard executed: YES
- Admission Guard result: FAIL - ENVIRONMENT PATHING
- Admission Guard changed: NO

## 16. Final Decision
PASS WITH WARNINGS — R12 UI Text Measurement boundary defined.

This PR defines the future `TextMeasureProvider` / text measurement evidence boundary only. It introduces no source code, no tests, no font access, no OS measurement, no shaping, no backend behavior, no runtime behavior, no action/effect behavior, and no capability admission.

DoD-2 Minimal Layout Ready now has its final boundary requirement documented.

## 17. Recommended Next Lane
R12-UI-MINIMAL-LAYOUT-CAPABILITY-CLOSEOUT-PR
