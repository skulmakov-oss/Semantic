# Motion Phase Contract

## Status

Motion phases are deterministic evidence for `ui-shell-kit`.

They are not free-form animation.

They are not renderer backend behavior.

They are not Semantic runtime artifacts.

## Purpose

The motion phase contract exists to verify:

- stable phase names;
- stable phase ordering;
- deterministic visual settling;
- reviewable motion evidence;
- stable draw-command output per phase;
- calculator scene polish without nondeterminism.

## Boundary

The motion phase contract belongs to `experiments/ui-shell-kit`.

It remains inside the experimental POST-UI track.

It must not require production UI wiring.

It must not define `prom-ui` animation behavior.

It must not define GPU/shader behavior.

## Motion Model

```text
scene state
  ↓
motion phase
  ↓
phase-dependent visual parameters
  ↓
draw-command emission
  ↓
deterministic snapshot / dump evidence
```

## Phase Set

The current phase set is:

- `Entrance`
- `Settling`
- `Settled`

These names are used by the current motion dump example and calculator scene evidence flow.

## Phase Semantics

- `Entrance`:
  - initial visual presentation phase.
- `Settling`:
  - intermediate visual settling phase.
- `Settled`:
  - final stable visual state.

The current code selects motion explicitly by phase value rather than by a realtime animation clock.

## Determinism Requirements

Given the same scene state, theme, bounds, interaction state, and motion phase, the emitted draw-command stream must be stable.

Motion must not depend on:

- wall-clock time;
- platform timer jitter;
- GPU frame timing;
- OS compositor behavior;
- randomness;
- device refresh rate.

Explicit phase selection is a strength of the current implementation because it keeps motion reviewable and repeatable.

## Visual Properties Under Motion

The current calculator scene applies phase-dependent changes to:

- ambient glow intensity;
- panel glow intensity;
- header line width;
- title/subtitle emphasis through color interpolation;
- header chip width;
- display glow.

These properties are implemented in `experiments/ui-shell-kit/src/calculator_scene.rs`.

## Relationship to Draw-command Snapshots

See `#1313` for the draw-command snapshot policy.

Motion evidence must be represented through deterministic draw-command output.

The snapshot policy defines how command-stream evidence is reviewed.

This document defines how motion phases affect that evidence.

## Relationship to Calculator Shell

See `#1312` for the calculator shell contract.

The calculator shell is the first canonical scene using the phased motion evidence model.

## Evidence Examples

The current evidence examples are:

- `cargo run --example calculator_motion_dump`
- `cargo run --example calculator_interaction_dump`

The `calculator_motion_dump` example should demonstrate the phase sequence:

- `Entrance`
- `Settling`
- `Settled`

## Contract Breaks

The following count as contract breaks unless explicitly accepted:

- renaming a motion phase without contract update;
- removing a documented phase;
- making phase output nondeterministic;
- changing phase ordering without contract update;
- removing documented phase-dependent visual properties;
- making motion depend on wall-clock time;
- panic during motion dump generation;
- motion evidence no longer producing stable draw-command output.

## In Scope

- define current motion phases;
- document deterministic phase semantics;
- document phase-dependent visual evidence;
- connect motion evidence to draw-command snapshots;
- keep calculator motion reviewable through examples;
- preserve experimental-track isolation.

## Hard Non-goals

- no production UI wiring;
- no verifier changes;
- no VM changes;
- no SemCode changes;
- no runtime capability widening;
- no Workbench implementation dependency;
- no renderer backend decision;
- no GPU/shader backend contract;
- no realtime animation scheduler contract;
- no browser/mobile target;
- no claim that `ui-shell-kit` motion is the final Semantic animation model.

## Acceptance Criteria

- motion phases are named and documented;
- phase ordering and phase semantics are clear;
- settle behavior is deterministic and explainable;
- visible evidence of motion is defined for examples and snapshots;
- the document states what counts as a motion contract break;
- the document preserves the experimental sandbox boundary;
- the document links back to `#1310`;
- the document depends on `#1313`;
- no code or production wiring changes are introduced.
