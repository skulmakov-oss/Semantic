# Calculator Shell Contract

## Status

The calculator shell is the first canonical reference scene for `ui-shell-kit`.

It is UI-local.

It is not Semantic runtime truth.

It is not production UI wiring.

## Purpose

The calculator shell exists to prove that `ui-shell-kit` can express:

- scene layout;
- display rendering;
- button grid layout;
- input routing;
- state update;
- action emission;
- focus handling;
- hover / pressed states;
- deterministic draw-command output;
- non-panicking error behavior.

This contract defines the calculator shell as a reference scene for the experimental UI shell track.

## Boundary

The calculator shell belongs to `experiments/ui-shell-kit`.

It must remain inside the experimental POST-UI track.

It must not require production wiring.

## Scene Model

The current calculator scene surface is implemented across:

- `experiments/ui-shell-kit/src/calculator_scene.rs`
- `experiments/ui-shell-kit/src/calculator_controller.rs`
- `experiments/ui-shell-kit/src/calculator_state.rs`
- `experiments/ui-shell-kit/src/components.rs`
- `experiments/ui-shell-kit/src/layout.rs`
- `experiments/ui-shell-kit/src/action.rs`
- `experiments/ui-shell-kit/src/event.rs`
- `experiments/ui-shell-kit/src/focus.rs`
- `experiments/ui-shell-kit/src/hit_test.rs`

Observed behavior:

- the scene is a calculator panel centered inside the scene bounds;
- the calculator scene has a display region and a 4x4 button grid;
- the button set currently includes digits `0` through `9` and the operations `+`, `-`, `*`, `/`, `=`, `C`;
- hit targets are registered for the interactive buttons;
- the controller maps pointer input to button presses through hit testing;
- keyboard focus is tracked by a `FocusRing`;
- the controller emits UI actions when focus changes or buttons are pressed;
- render projection is derived from the calculator state and focus state.

This document describes only behavior that exists in the current implementation.

## Visual Structure

The current scene structure includes:

- backdrop / scene background;
- calculator panel;
- optional header / title area;
- display;
- button grid;
- calculator buttons;
- visible focus styling when a button is focused.

The calculator scene currently renders a glass-style panel with a title area and a small header chip, plus the display and button grid.

## Display Contract

The display shows the current UI-local calculator value as text.

Display text changes after input when the calculator state updates.

Error states are represented by the UI-local error value `ERR`.

Display rendering participates in deterministic snapshots through the draw-command stream.

## Button Grid Contract

The current button set is:

- `7`
- `8`
- `9`
- `/`
- `4`
- `5`
- `6`
- `*`
- `1`
- `2`
- `3`
- `-`
- `C`
- `0`
- `=`
- `+`

The grid is 4x4 and the buttons are laid out in calculator order.

## Input and Action Routing

The current route is:

```text
raw event
  ↓
hit test / focus handling
  ↓
calculator action
  ↓
state update
  ↓
render projection / snapshot output
```

The action model in `action.rs` currently includes:

- `ButtonPressed`
- `CalculatorButtonPressed`
- `FocusChanged`
- `CloseRequested`
- `None`

The controller uses hit testing to map pointer input to a button, updates focus, emits actions, and then updates calculator state.

## Calculator State Contract

The current calculator state is implemented in `calculator_state.rs` and includes:

- `display: String`
- `accumulator: Option<i64>`
- `pending_operator: Option<CalculatorButton>`
- `replace_on_next_digit: bool`
- `error: bool`

Observed behavior:

- digit input appends or replaces the display depending on state;
- clear resets the calculator back to `0` and clears pending state;
- operator input may evaluate a pending operation and then arm the next operator;
- equals evaluates the pending operation and clears the pending operator;
- error state blocks further input except `Clear`.

## Arithmetic Behavior

Supported arithmetic is UI-local and deterministic.

Current behavior includes:

- addition;
- subtraction;
- multiplication;
- division;
- equals;
- clear;
- digit input.

The calculator uses `i64` arithmetic and checked operations.

## Division-by-zero Behavior

Division by zero must not panic.

The current implementation resolves division by zero to the UI-local error state:

- `display = "ERR"`
- `error = true`

## Focus / Hover / Pressed States

The calculator shell currently supports visible interaction states for buttons:

- hovered;
- pressed;
- focused;
- default;
- disabled.

These are UI style states, not runtime states.

Focus is managed by `FocusRing` and mapped to the visible button presentation.

## Snapshot and Evidence Expectations

Calculator shell evidence may come from:

- `calculator_layout_dump`
- `calculator_interaction_dump`
- snapshot tests
- golden-style tests
- draw-command command count
- stable render output

This document does not define the full snapshot policy.

See `#1313` for the draw-command snapshot policy follow-up.

## In Scope

- define the calculator shell as the canonical reference scene for `ui-shell-kit`;
- describe the current UI-local state model;
- describe the current button set and action routing;
- describe deterministic arithmetic behavior;
- preserve non-panicking error behavior;
- preserve focus / hover / pressed presentation states;
- preserve evidence-oriented snapshots and dumps.

## Hard Non-goals

- no production UI wiring;
- no verifier changes;
- no VM changes;
- no SemCode changes;
- no runtime capability widening;
- no Workbench implementation dependency;
- no promotion decision;
- no claim that calculator behavior is Semantic runtime authority;
- no renderer backend decision.

## Relationship to Other POST-UI Issues

- `#1310` — parent POST-UI track
- `#1311` — experimental boundary
- `#1313` — draw-command snapshot policy
- `#1314` — phased motion evidence model
- `#1315` — promotion gate to `prom-ui` and Workbench

## Acceptance Criteria

- calculator shell behavior is described in contract form;
- visible UI states and state transitions are named;
- input routing and emitted actions are documented;
- arithmetic behavior is deterministic and documented;
- division by zero is explicitly non-panicking;
- the document states that this is UI-local behavior;
- the document links back to `#1310` and depends on `#1311`;
- no code or production wiring changes are introduced.
