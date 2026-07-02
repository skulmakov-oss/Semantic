# PCC Match Fallback Arm Policy

Status: PCC-CF-2 policy note

This document records the current `match` fallback arm policy for the Practical
Core Completion track.

It is intentionally conservative. It does not change parser behavior. It does
not introduce exhaustiveness checking. It documents the currently admitted
surface so later work can separate policy from implementation cleanly.

## Current admitted behavior

The current admitted surface requires an explicit `_` fallback arm in `match`.

This applies even when all currently known variants or states are listed
explicitly.

Example:

```semantic
fn classify(state: quad) -> i32 {
    match state {
        T => return 10;
        F => return 0;
        N => return 1;
        S => return -10;
        _ => return -100;
    }
}
```

## Observed examples

This behavior is reflected in:

- `examples/canonical/match_control_flow`
- `examples/canonical/option_result_control_flow`
- `docs/roadmap/pcc/control_flow_core_audit.md`
- `docs/roadmap/pcc/control_flow_core_contract.md`

## Why the current rule is safe

Requiring `_` keeps the parser, typechecker, and lowering contract simple:

- no exhaustiveness analysis is required yet;
- all match expressions have an explicit fallback path;
- unknown or future variants have a deterministic destination;
- canonical examples remain robust under future surface widening.

## Why the current rule is not final

The rule is conservative.

For closed domains like `quad`, this is verbose:

```semantic
match state {
    T => return 1;
    F => return 0;
    N => return 2;
    S => return -1;
    _ => return -100;
}
```

Since `quad` has exactly four states, a future exhaustiveness checker could
allow the `_` arm to be omitted when all states are covered.

## Policy decision for PCC

For the current PCC control-flow phase:

```text
Keep explicit `_` fallback arm required.
Do not change parser behavior in PCC-CF-2.
Document this as current admitted behavior.
```

## Future direction

A future issue may introduce an exhaustiveness checker.

Possible future policy:

```text
closed-domain match:
  full coverage => `_` not required

open-domain match:
  `_` required
```

Candidate closed domains:

- `quad`
- `Option(T)`
- `Result(T, E)`
- finite ADT variants

## Future implementation requirements

Before removing the `_` requirement, the project needs:

- explicit variant/domain metadata;
- exhaustiveness checking in semantic analysis;
- stable diagnostics for missing arms;
- lowering guarantee for fully covered match;
- verifier and VM compatibility confirmation;
- compile-fail fixtures for non-exhaustive match.

## Out of scope

This document does not:

- change parser behavior;
- change match lowering;
- introduce exhaustiveness checking;
- remove `_` from canonical examples;
- define ADT exhaustiveness generally;
- change `Option` / `Result` constructor semantics;
- widen the control-flow surface.

## Current canonical rule

Canonical examples should include `_` in `match` until a later exhaustiveness
policy is implemented and qualified.

## Follow-up issues

- PCC-CF-3: terminal return paths
- PCC-CF-4: qualify while/loop/break/continue surface
- PCC-CF-5: add negative diagnostics fixtures
- PCC-CF-X: future match exhaustiveness checker
