# Calculator Reference Scenario

## Status

The calculator interaction dump is the canonical reference scenario for `ui-shell-kit`.

It is UI-local.

It is not Semantic runtime truth.

It is not production UI wiring.

## Purpose

This scenario documents the reference calculator flow that demonstrates:

- scene layout;
- deterministic input routing;
- state update;
- action emission;
- focus tracking;
- draw-command snapshot evidence;
- non-panicking UI-local arithmetic behavior.

The scenario exists to make the existing calculator example a stable reference for the experimental UI shell track.

## Scenario

Canonical input flow:

```text
Initial
press 7
press +
press 3
press =
Final
```

Expected result:

```text
Display: 10
```

## Expected Initial State

The initial calculator state is expected to show:

- display value `0`;
- no prior operator pending;
- a deterministic initial draw-command stream;
- no committed production-side effect.

## Input Sequence

The canonical interaction sequence is:

1. press `7`
2. press `+`
3. press `3`
4. press `=`

This sequence is the reference path for `calculator_interaction_dump`.

## Expected Final State

After the input sequence completes:

- display value is `10`;
- the calculator state reflects the final evaluated result;
- the scenario remains UI-local and deterministic.

## Action Trace Expectations

The interaction dump should show explicit UI actions for each press, including focus updates and button press notifications.

The trace should remain readable and deterministic for the same scene state.

The scenario is expected to show a compact action trace around:

- focus changes;
- calculator button press labels;
- button press records.

## Snapshot / Draw-command Expectations

The interaction dump and related snapshots are evidence for the calculator scene contract.

The command stream should remain stable for a given scene state unless the scene contract intentionally changes.

The current implementation emits a stable command gallery that includes the backdrop, panel, display, button grid, and labels.

The current command count is an evidence point, not a permanent hard contract unless the test suite explicitly enforces it.

Reference example:

- `cargo run --example calculator_interaction_dump`

## Motion Evidence

The calculator motion example is the paired evidence source for phase-based visual settling.

Reference example:

- `cargo run --example calculator_motion_dump`

The motion evidence should show the phase sequence:

- `Entrance`
- `Settling`
- `Settled`

## Boundary

This reference scenario belongs to `experiments/ui-shell-kit`.

It must remain inside the experimental POST-UI track.

It must not require production UI wiring.

It must not define `prom-ui` runtime behavior.

## Hard Non-goals

- no production UI wiring;
- no verifier changes;
- no VM changes;
- no SemCode changes;
- no runtime capability widening;
- no Workbench implementation dependency;
- no renderer backend decision;
- no GPU/shader backend contract;
- no browser/mobile target;
- no promotion to `prom-ui`.

## Acceptance Criteria

The document is complete when:

- the calculator interaction flow is documented as the canonical reference scenario;
- the `7 + 3 = 10` path is recorded;
- the initial and final display states are described;
- the action trace expectations are stated;
- the draw-command / snapshot evidence is treated as deterministic but not pixel-based;
- the motion evidence references `Entrance`, `Settling`, and `Settled`;
- no production wiring is introduced;
- no code outside `experiments/ui-shell-kit` or docs is modified.
