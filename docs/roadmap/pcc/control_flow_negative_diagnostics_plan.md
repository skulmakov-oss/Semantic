# PCC Control-flow Negative Diagnostics Plan

Status: PCC-CF-5A fixture corpus + diagnostics plan

This document defines the first negative fixture corpus for Practical Core
control-flow diagnostics.

It is intentionally conservative. The corpus is documented first, and the
compile-fail harness can follow as a separate issue if needed.

## Goal

Record the failure modes that the current practical control-flow contour should
reject with stable diagnostics.

## Scope

This plan covers negative fixtures for:

- implicit `quad` truthiness in `if`
- implicit `quad` truthiness in `while`
- `break` outside loop
- `continue` outside loop
- missing `_` fallback arm in the current `match` policy
- missing return path in a non-void function

## Fixture Path

```text
tests/fixtures/pcc/control_flow/fail/
```

## Fixture Matrix

| Fixture | Expected failure |
| --- | --- |
| `if_quad_condition.sm` | `if` condition must be `bool` |
| `while_quad_condition.sm` | `while` condition must be `bool` |
| `break_outside_loop.sm` | `break` outside loop |
| `continue_outside_loop.sm` | `continue` outside loop |
| `match_missing_fallback.sm` | `_` arm required by current PCC match policy |
| `missing_return_path.sm` | `return type mismatch` / missing return path family |

## Harness Policy

If compile-fail infrastructure is not available yet, these fixtures remain a
documented corpus only.

They should not be wired into CI until a stable negative diagnostics harness
can assert:

- command failure;
- stable diagnostic code or message marker;
- no panic;
- no verifier or VM execution after failed admission.

## Current observation

The `missing_return_path.sm` fixture currently manifests through the stable
`return type mismatch` family when a non-void function uses a bare `return;`
on the failing path.

The current `match_missing_fallback.sm` probe surfaces as a parser rejection
with `Error [E0000]: expected '{'` when the fallback arm is omitted from the
quad match. That is the current observed marker for this negative case.

The other loop/control fixtures surface through stable `E0201` diagnostics with
specific control-flow markers.

## Harness Status

`PCC-CF-5B` adds:

```text
tests/pcc_control_flow_negative.rs
```

The harness currently asserts:

- the fixture fails;
- expected broad diagnostic markers are present;
- the process does not panic.

It intentionally does not assert full diagnostic text or exact spans.

## Follow-up

- PCC-CF-5B: add compile-fail harness for PCC diagnostics.

## Non-Goals

- No parser changes.
- No lowering changes.
- No VM changes.
- No verifier admission changes.
- No CI wiring in this issue.
