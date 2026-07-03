# Draw-command Snapshot Policy

## Status

Draw-command snapshots are deterministic evidence for `ui-shell-kit`.

They are not pixel-perfect renderer screenshots.

They are not production renderer tests.

They are not Semantic runtime artifacts.

## Purpose

Snapshots exist to verify:

- stable scene projection;
- stable draw-command ordering;
- stable layout output;
- stable component rendering intent;
- stable interaction state rendering;
- stable evidence for examples and tests.

Snapshots are used to review the UI shell contract, not backend-specific rendering.

## Boundary

The snapshot policy belongs to `experiments/ui-shell-kit`.

It remains inside the experimental POST-UI track.

It must not require production UI wiring.

It must not define `prom-ui` renderer behavior.

## Snapshot Model

```text
scene state
  ↓
layout / style resolution
  ↓
draw-command emission
  ↓
stable command stream
  ↓
snapshot text / golden evidence
```

## What a Snapshot Captures

Based on the current implementation, snapshots capture:

- command kind;
- geometry / rects / positions;
- text labels and display text;
- style tokens or resolved style values when they are emitted as command fields;
- theme-dependent command output;
- draw order;
- interaction-state rendering when it changes emitted commands;
- command count through frame stats where examples or tests print it.

`ui-shell-kit` snapshots are command-stream snapshots. They are not image dumps.

## What a Snapshot Must Not Capture

Snapshots must not depend on:

- GPU backend;
- font rasterization;
- platform compositor;
- real wall-clock time;
- nondeterministic animation timing;
- device DPI quirks;
- operating system rendering differences;
- image pixels.

## Determinism Requirements

Given the same scene state, theme, bounds, interaction state, and motion phase, the draw-command snapshot must be stable.

If a snapshot path does not explicitly take a motion phase, the policy applies to the current non-motion snapshot state only.

See `#1314` for the phased motion evidence model.

## Command Ordering

Command order is part of the contract when it affects scene layering, hit readability, or stable evidence.

The current calculator scene emits commands in a layering order that includes:

- background / backdrop;
- panel / acrylic surface;
- header / decorative elements;
- display;
- buttons;
- focus / overlay elements when present.

This ordering should remain stable unless the scene contract is intentionally updated.

## Stable Fields

Stable fields are those that should not change without a contract update.

Examples present in the current implementation include:

- command type;
- rect / geometry;
- text content;
- draw order;
- semantic role of the command as expressed by the scene;
- button labels;
- display text;
- focus-state rendering when represented in commands.

## Unstable / Non-contract Fields

Fields that may change without breaking the contract are limited to non-semantic formatting details such as:

- minor internal formatting of snapshot text;
- debug-only labels;
- non-contract diagnostics;
- implementation-local helper names.

Visual geometry is not considered unstable when golden tests depend on it.

## Golden Evidence Policy

Golden snapshots are evidence.

They are not decorative dumps.

A golden update must be reviewed as a contract change unless explicitly marked as a non-contract formatting change.

If a snapshot changes, the reviewer must decide whether it is:

1. an intentional contract update;
2. a harmless formatting change;
3. a regression.

## Contract Breaks

The following count as contract breaks unless the change is explicitly accepted:

- unexpected command reorder;
- changed layout geometry without a contract update;
- missing display command;
- missing button command;
- changed button labels;
- lost focus-state rendering;
- nondeterministic snapshot output;
- panic during snapshot generation.

## Relationship to Calculator Shell

See `#1312` for the calculator shell contract.

The calculator shell is the first canonical scene using this evidence policy.

## Relationship to Motion Evidence

See `#1314` for the phased motion evidence model.

Motion snapshots must follow this draw-command policy, but motion phase semantics are defined separately in `#1314`.

## In Scope

- define deterministic draw-command snapshots as evidence;
- keep snapshot scope explicit and stable;
- preserve command ordering expectations;
- define stable and non-contract fields;
- document golden evidence review expectations;
- distinguish draw-command evidence from renderer/pixel output;
- preserve experimental-track isolation.

## Hard Non-goals

- no production UI wiring;
- no verifier changes;
- no VM changes;
- no SemCode changes;
- no runtime capability widening;
- no Workbench implementation dependency;
- no renderer backend decision;
- no pixel-perfect screenshot contract;
- no GPU/shader backend contract;
- no browser/mobile target.

## Acceptance Criteria

- snapshot determinism is documented;
- snapshot scope is explicit and stable;
- command ordering expectations are defined;
- golden evidence expectations are clear;
- contract break examples are listed;
- the policy distinguishes draw-command evidence from renderer/pixel output;
- the document links back to `#1310`;
- the document depends on `#1312`;
- no code or production wiring changes are introduced.
