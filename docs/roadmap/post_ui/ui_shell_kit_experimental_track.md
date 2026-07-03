# ui-shell-kit Experimental Track

## Status

`ui-shell-kit` is a first-class experimental POST-UI track.

It is not production UI wiring.

It remains isolated by default.

## Purpose

`ui-shell-kit` exists to explore Semantic-owned UI shell primitives, including:

- geometry;
- layout;
- paint commands;
- theme tokens;
- visual effects;
- deterministic motion;
- input events;
- focus handling;
- accessibility surface;
- hit testing;
- scene/tree structure;
- deterministic snapshots;
- calculator reference scene.

The package is a research and evidence surface for a Semantic-native UI shell direction.

## Architectural Position

```text
ui-shell-kit
=
Semantic-owned UI shell reference
+ deterministic draw-command model
+ layout / geometry primitives
+ input / focus / accessibility contract
+ calculator scene prototype
+ snapshot evidence surface
+ motion evidence model
```

This belongs to the POST-UI exploration layer.

## Package Boundary

`experiments/ui-shell-kit` must not be treated as part of the production Semantic workspace unless a later promotion gate explicitly allows it.

No production wiring is introduced by this track document.

## Current Surface

The current exported module surface from `src/lib.rs` is:

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

## Examples as Living Specs

The following runnable examples exist and serve as living specs for the experimental track:

- `cargo run --example calculator_layout_dump`
- `cargo run --example calculator_interaction_dump`
- `cargo run --example calculator_motion_dump`
- `cargo run --example theme_variants_dump`
- `cargo run --example theme_glass_dump`
- `cargo run --example theme_aurora_dump`
- `cargo run --example theme_midnight_dump`
- `cargo run --example hit_test_dump`

Examples are evidence surfaces for layout, interaction, motion, themes, and hit testing.

## In Scope

- keep `ui-shell-kit` as a Semantic-owned experimental UI shell reference;
- preserve the isolated package boundary;
- document the calculator shell as a reference scene;
- preserve deterministic snapshots and motion evidence;
- keep examples as living specs;
- maintain the package as an exploratory shell track, not production UI wiring.

## Hard Non-goals

- no production UI wiring;
- no verifier changes;
- no VM changes;
- no SemCode changes;
- no runtime capability widening;
- no Workbench implementation dependency;
- no browser/mobile target;
- no GPU/shader backend decision;
- no claim that ui-shell-kit is the final Semantic UI framework.

## Relationship to prom-ui and Workbench

`ui-shell-kit` may inform future `prom-ui` or Workbench work, but no part of it may be promoted without a separate promotion gate.

See follow-up gate issue `#1315`.

## Evidence Surface

`ui-shell-kit` provides evidence through:

- deterministic draw-command snapshots;
- calculator scene render dumps;
- motion phase dumps;
- interaction/action trace dumps;
- golden-style tests;
- layout and hit-test evidence;
- theme and style variant coverage.

## Follow-up Track

- `#1312` — calculator shell contract
- `#1313` — draw-command snapshot policy
- `#1314` — phased motion evidence model
- `#1315` — promotion gate to `prom-ui` and Workbench

## Acceptance Criteria

- `ui-shell-kit` is documented as a first-class experimental POST-UI track;
- the package boundary is explicit;
- the package remains isolated from production wiring;
- current modules and examples are listed based on actual files;
- hard non-goals are recorded;
- relationship to `prom-ui` and Workbench is explicit;
- follow-up issues are linked;
- no code or workspace wiring changes are introduced.
