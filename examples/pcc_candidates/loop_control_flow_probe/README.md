# PCC candidate: loop_control_flow_probe

This is a PCC candidate probe sample.

It is **not** canonical yet.

## Purpose

This sample probes the admitted surface for `while`, `loop`, `break`, and
`continue` in Semantic.

The goal is to determine whether the current practical contour supports:

- `while` with a `bool` condition;
- `loop` as a statement-loop surface;
- `break;` inside the admitted loop body;
- `continue;` inside the admitted loop body;
- mutable rebinding inside loop bodies;
- deterministic return paths after loop completion;
- `assert`-based self-checks.

## Candidate status

This sample should remain under:

```text
examples/pcc_candidates/loop_control_flow_probe/
```

until the admitted surface is confirmed.

It may be promoted to:

```text
examples/canonical/loop_control_flow_probe/
```

only after it passes the project qualification path and the syntax is stable
enough for canonical examples.

## Promotion note

This probe has been promoted to:

```text
examples/canonical/loop_control_flow/
```

The probe directory is kept as an audit trail for the original qualification
step.

## What this sample covers

- `while` control flow;
- `loop` control flow;
- `break` and `continue`;
- mutable integer updates in loop bodies;
- return-after-loop logic;
- deterministic branch handling.

## What this sample does not cover

- `for` loops;
- `loop` as a value-producing expression;
- labeled loops;
- `break expr;` loop-expression flow;
- async control flow;
- iterator adapters;
- speculative syntax not present in the current admitted surface.

## Expected behavior

If the current admitted surface supports this contour, the sample should pass:

```bash
cargo run --bin smc -- check examples/pcc_candidates/loop_control_flow_probe/src/main.sm
```

If it fails, the failure should be treated as diagnostic information for PCC,
not as a canonical regression.

## Probe questions

Record the results:

```text
while condition:
  status: pending
  accepted form: pending

loop surface:
  status: pending
  accepted form: pending

break / continue:
  status: pending
  accepted form: pending

mutable loop updates:
  status: pending
  accepted form: pending

Validation:
  smc check: pending

Conclusion:
  candidate status: pending
  promote to canonical: not yet
```

## Promotion criteria

This sample can become canonical only when:

- `smc check` passes;
- syntax is stable enough for public examples;
- the loop/break/continue policy is documented;
- no experimental assumptions remain;
- the sample can be added to canonical tests and smoke matrix without special
  casing.
