# ui-shell-kit Surface Inventory

## Status

`ui-shell-kit` is a first-class experimental POST-UI sandbox.

This inventory is docs-only.

No code was changed.

No production UI wiring was introduced.

## Purpose

Classify the current `experiments/ui-shell-kit` exported surface before any promotion, extraction, or reuse decision.

The goal is to make the sandbox reviewable in terms of:

- stable reference evidence;
- reusable primitive candidates;
- scene-specific pieces;
- experimental / unstable areas;
- explicit non-promotion surfaces.

## Boundary

This inventory belongs to `experiments/ui-shell-kit` and stays inside the experimental POST-UI track.

It does not promote, refactor, move, or wire any code.

It does not define `prom-ui` ownership.

## References

- `#1310` — parent POST-UI track
- `#1311` — experimental boundary
- `#1312` — calculator shell contract
- `#1313` — draw-command snapshot policy
- `#1314` — phased motion evidence model
- `#1315` — promotion gate
- `#1316` — initial documentation spine
- `#1317` — calculator reference scenario

## Current Surface

`experiments/ui-shell-kit/src/lib.rs` exports the following modules:

- `theme`
- `geometry`
- `layout`
- `paint`
- `effects`
- `motion`
- `text`
- `components`
- `scene`
- `hit_test`
- `calculator_scene`
- `style`
- `event`
- `action`
- `focus`
- `accessibility`
- `tree`
- `diagnostics`
- `snapshot`
- `builder`
- `theme_variants`
- `calculator_state`
- `calculator_controller`

The module roles below are classified from the current source surface, tests, README, and reference examples such as:

- `cargo run --example calculator_layout_dump`
- `cargo run --example calculator_interaction_dump`
- `cargo run --example calculator_motion_dump`
- `cargo run --example theme_variants_dump`
- `cargo run --example theme_glass_dump`
- `cargo run --example theme_aurora_dump`
- `cargo run --example theme_midnight_dump`
- `cargo run --example hit_test_dump`

## Module Inventory

| Module | Current role | Classification | Evidence source | Likely future owner | Promotion risk | Notes |
|---|---|---|---|---|---|---|
| `theme` | Theme token container and default shell palette | Reusable primitive candidate | `src/lib.rs`, `README.md`, `tests/theme_variant_tests.rs`, theme examples | Shared UI shell primitives | Medium | Strong candidate for future reuse, but palette semantics should stay sandbox-owned until a gate passes. |
| `geometry` | `UiRect` / point / inset / fit helpers | Reusable primitive candidate | `src/lib.rs`, `tests/geometry_tests.rs`, layout examples | Shared UI shell primitives | Low | Foundational geometry is broadly reusable. |
| `layout` | Stack/grid/panel layout helpers and calculator layout math | Reusable primitive candidate | `src/lib.rs`, `tests/layout_tests.rs`, `calculator_layout_dump` | Shared UI shell primitives | Medium | Generic layout helpers look reusable; calculator-specific layout calls should stay scene-local. |
| `paint` | Abstract draw-command frame and command stream | Reference evidence | `src/lib.rs`, `tests/paint_tests.rs`, `tests/snapshot_tests.rs` | Evidence/presentation layer | High | This is the command-stream basis for snapshots, so it is evidence-bearing before it is reusable. |
| `effects` | Acrylic, glow, shadow, backdrop helpers | Reusable primitive candidate | `src/lib.rs`, `tests/effects_tests.rs`, `calculator_motion_dump` | Shared UI shell primitives | Medium | Strong visual primitive candidate, but should not become a production renderer contract by accident. |
| `motion` | Deterministic interpolation/easing helpers | Reference evidence | `src/lib.rs`, `tests/motion_tests.rs`, `calculator_motion_dump`, `#1314` | Evidence/presentation layer | High | Motion is currently evidence-oriented and must stay deterministic. |
| `text` | Measurement and text-origin helpers | Reusable primitive candidate | `src/lib.rs`, `tests/text_tests.rs` | Shared UI shell primitives | Low | Good reusable primitive if kept renderer-agnostic. |
| `components` | Panel/display/button composition helpers | Reusable primitive candidate | `src/lib.rs`, `tests/component_contract_tests.rs` | Shared UI shell primitives | Medium | Looks reusable, but still needs boundary discipline around visual roles. |
| `scene` | Scene wrapper / scene-kind metadata / content rect helpers | Reusable primitive candidate | `src/lib.rs`, `tests/scene_tests.rs`, calculator scene examples | Shared UI shell primitives | Medium | Generic enough to remain a shell primitive candidate. |
| `hit_test` | Hit-target registry and lookup | Reusable primitive candidate | `src/lib.rs`, `tests/hit_test_tests.rs`, `hit_test_dump` | Shared interaction primitives | Medium | Useful for broader shell interactions; keep it independent of calculator specifics. |
| `calculator_scene` | Calculator layout/render reference scene | Scene-specific | `src/lib.rs`, `tests/calculator_scene_tests.rs`, calculator examples, `#1312`, `#1317` | Calculator reference sandbox | High | The canonical calculator scene is intentionally scenario-specific. |
| `style` | Button/display state-to-style resolution | Reusable primitive candidate | `src/lib.rs`, `tests/style_tests.rs` | Shared UI shell primitives | Medium | Good reusable styling layer, but needs stable token semantics. |
| `event` | Raw input normalization into `UiEvent` | Reusable primitive candidate | `src/lib.rs`, `tests/event_tests.rs`, interaction examples | Shared interaction primitives | Medium | Input normalization is generic, but should remain sandbox-local until promoted explicitly. |
| `action` | UI action model / action queue | Reusable primitive candidate | `src/lib.rs`, `tests/action_tests.rs` | Shared interaction primitives | Medium | Potentially reusable as a shell contract surface. |
| `focus` | Focus ring and keyboard navigation | Reusable primitive candidate | `src/lib.rs`, `tests/focus_tests.rs`, controller examples | Shared interaction primitives | Medium | Good generic primitive, but should not absorb app-level policy. |
| `accessibility` | Accessibility nodes and roles | Reusable primitive candidate | `src/lib.rs`, `tests/accessibility_tests.rs` | Shared UI shell primitives | Medium | Reusable if kept as a shell-accessibility contract only. |
| `tree` | UI tree structure and node IDs | Reusable primitive candidate | `src/lib.rs`, `tests/tree_tests.rs` | Shared UI shell primitives | Medium | Generic tree model is a likely candidate for later reuse. |
| `diagnostics` | Frame validation / correctness checks | Do not promote | `src/lib.rs`, `tests/diagnostics_tests.rs` | Test/evidence harness only | Low | This is validation support, not a production UI primitive. |
| `snapshot` | Stable command-stream serialization and dump helpers | Reference evidence | `src/lib.rs`, `tests/snapshot_tests.rs`, calculator/layout/motion dumps, `#1313` | Evidence/presentation layer | High | Snapshot output is contract evidence, not pixel output. |
| `builder` | Fluent builders for components | Reusable primitive candidate | `src/lib.rs`, `tests/builder_tests.rs` | Shared UI shell primitives | Medium | Useful ergonomics layer if kept thin and explicit. |
| `theme_variants` | Preset theme families and palette exploration | Experimental / unstable | `src/lib.rs`, `tests/theme_variant_tests.rs`, theme examples | Sandbox experimentation | Medium | Good for exploration; too experimental to treat as a promotion candidate yet. |
| `calculator_state` | UI-local calculator state machine | Scene-specific | `src/lib.rs`, `tests/calculator_state_tests.rs`, `#1312`, `#1317` | Calculator reference sandbox | High | Encodes calculator behavior, so it should remain scene-local for now. |
| `calculator_controller` | Input-to-action coordinator for the calculator scene | Scene-specific | `src/lib.rs`, `tests/calculator_controller_tests.rs`, interaction example, `#1317` | Calculator reference sandbox | High | This is tightly coupled to the calculator reference path and should not be treated as generic shell state. |

## Classification Summary

- Reference evidence:
  - `paint`
  - `motion`
  - `snapshot`
- Reusable primitive candidate:
  - `theme`
  - `geometry`
  - `layout`
  - `effects`
  - `text`
  - `components`
  - `scene`
  - `hit_test`
  - `style`
  - `event`
  - `action`
  - `focus`
  - `accessibility`
  - `tree`
  - `builder`
- Scene-specific:
  - `calculator_scene`
  - `calculator_state`
  - `calculator_controller`
- Experimental / unstable:
  - `theme_variants`
- Do not promote:
  - `diagnostics`

## Hard Non-goals

- no production UI wiring;
- no code changes;
- no module moves;
- no refactors;
- no workspace changes;
- no `prom-ui` integration;
- no Workbench dependency;
- no verifier changes;
- no VM changes;
- no SemCode changes;
- no runtime capability widening;
- no renderer backend decision;
- no promotion decision.

## Acceptance Criteria

- the current `ui-shell-kit` surface is classified;
- exported modules are listed based on actual code;
- reusable primitive candidates are named only as candidates;
- scene-specific pieces are clearly separated from generic primitives;
- unstable / experimental areas are marked honestly;
- non-promotion surfaces are explicit;
- the inventory links back to `#1310` and the prior POST-UI evidence issues;
- no code or workspace files are changed;
- future promotion still requires `#1315` gate discipline.
