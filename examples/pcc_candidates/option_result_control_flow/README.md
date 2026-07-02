# PCC candidate: option_result_control_flow

This is a PCC candidate sample.

It is **not** part of the canonical example pack yet.

## Purpose

This sample probes the current admitted surface for `Option` / `Result` style
control flow in Semantic.

The goal is to determine whether the language currently supports:

- `Option(T)` type surface;
- `Result(T, E)` type surface;
- `Option::Some` / `Option::None` constructors;
- `Result::Ok` / `Result::Err` constructors;
- `match` over these values;
- terminal return paths in each match arm;
- fallback `_` arm behavior;
- `assert`-based self-checks.

## Candidate status

This sample should remain under:

```text
examples/pcc_candidates/option_result_control_flow/
```

until the admitted surface is confirmed.

It may be promoted to:

```text
examples/canonical/option_result_control_flow/
```

only after it passes the project qualification path and the syntax is stable
enough for canonical examples.

## What this sample covers

- `Option`-like value flow;
- `Result`-like value flow;
- `match` over optional/success/error states;
- fallback `_` arm behavior;
- ordinary numeric return codes;
- deterministic branch handling.

## What this sample does not cover

- full error handling design;
- exceptions;
- async control flow;
- collection integration;
- text formatting of errors;
- public stdlib policy for `Option` / `Result`.

## Expected behavior

If the current admitted surface supports this contour, the sample should pass:

```bash
cargo run --bin smc -- check examples/pcc_candidates/option_result_control_flow/src/main.sm
```

If it fails, the failure should be treated as diagnostic information for PCC,
not as a canonical regression.

## Probe questions

Record the results:

```text
Option type syntax:
  status: pending
  accepted form: pending

Result type syntax:
  status: pending
  accepted form: pending

Constructors:
  Option::Some: pending
  Option::None: pending
  Result::Ok: pending
  Result::Err: pending

Match behavior:
  match over Option: admitted with _ fallback in current probe
  match over Result: admitted with _ fallback in current probe
  fallback _ arm required: yes, based on current parse result

Validation:
  smc check: pending

Conclusion:
  candidate status: probe passed, canonical promotion still pending
  promote to canonical: not yet
```

## Promotion criteria

This sample can become canonical only when:

- `smc check` passes;
- syntax is stable enough for public examples;
- constructor forms are documented;
- fallback `_` arm behavior is documented;
- no experimental assumptions remain;
- the sample can be added to canonical tests and smoke matrix without special
  casing.
