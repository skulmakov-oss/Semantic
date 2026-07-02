# PCC Control-flow Core Closeout

## 0. Status

Status:
  COMPLETE / CURRENT PCC CONTROL-FLOW CONTOUR CLOSED

This document does **not** claim full language completion.
This document does **not** claim exhaustiveness redesign.
This document does **not** claim loop-expression expansion, `for`, labeled
loops, or expression-valued `match`.

## 1. Closed Scope

The qualified control-flow contour is:

- `if / else`
- `match`
- `return`
- `assert`
- `fn main()`
- `match` over `quad`
- `match` over `Option(T)`
- `match` over `Result(T, E)`
- `while`
- `loop`
- `break;`
- `continue;`
- terminal return paths in branch-heavy functions

This closeout captures the currently admitted, practical-safe control-flow
surface that is backed by examples, negative fixtures, and 7hell wiring.

## 2. Completed Slices

- CF-0: current admitted control-flow audit
- CF-1: control-flow core contract
- CF-2: match fallback arm policy
- CF-3: terminal return paths policy
- CF-4: loop surface probe
- CF-4B: loop control-flow canonical promotion
- CF-5A: negative diagnostics fixture corpus
- CF-5B: compile-fail harness
- CF-6A: 7hell control-flow group plan
- CF-6B: fixed Hell 6 runner integration

## 3. Practical-Safe Surface

The following control-flow surface is qualified for the current PCC contour:

- `if` conditions must be `bool`
- `quad` does not act as implicit truthiness
- `match` currently requires explicit `_` fallback in the admitted surface
- terminal branch returns are supported and preferred in branch-heavy examples
- `while` and statement `loop` are admitted as practical control flow
- `break;` and `continue;` are admitted inside the loop surface

## 4. Canonical Examples

The canonical examples that cover this contour are:

- `examples/canonical/match_control_flow/src/main.sm`
- `examples/canonical/option_result_control_flow/src/main.sm`
- `examples/canonical/loop_control_flow/src/main.sm`
- `examples/canonical/text_collections_toolbox/src/main.sm`

These examples are part of the canonical pack and are wired into the public
example qualification surface.

## 5. Negative Diagnostics Coverage

The control-flow negative corpus is covered by:

- `tests/pcc_control_flow_negative.rs`

Fixtures:

- `tests/fixtures/pcc/control_flow/fail/if_quad_condition.sm`
- `tests/fixtures/pcc/control_flow/fail/while_quad_condition.sm`
- `tests/fixtures/pcc/control_flow/fail/break_outside_loop.sm`
- `tests/fixtures/pcc/control_flow/fail/continue_outside_loop.sm`
- `tests/fixtures/pcc/control_flow/fail/match_missing_fallback.sm`
- `tests/fixtures/pcc/control_flow/fail/missing_return_path.sm`

## 6. 7hell Coverage

Hell 6 now runs:

```bash
cargo test --test pcc_control_flow_negative
```

The runner remains:

- linear;
- hardcoded;
- fail-fast;
- without `--group` selector;
- without fixture registry.

## 7. Documented Current Quirks

- canonical `fn main()` uses no return type;
- current `match` requires explicit `_` arm;
- `match_missing_fallback` currently reports a parser-level
  `E0000 expected '{'` marker;
- `missing_return_path` currently reports
  `E0201 return type mismatch: expected I32, got Unit`;
- exact diagnostic span formatting is intentionally not over-specified.

## 8. Out of Scope

Still out of scope for the current control-flow contour:

- `for`
- iterators
- labeled loops
- `break expr`
- expression-valued `match`
- exhaustiveness checker
- advanced ADT matching
- async / concurrency
- 7hell group registry / selector

## 9. Validation

Passed:

```bash
cargo test --test pcc_control_flow_negative
powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1
```

Also covered through:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`

## 10. Next PCC Contour

Recommended next practical contour:

```text
Text Core
```

Reason:

Control flow is now qualified enough to support practical examples. The next
weak practical axis is text usability and text / std helpers.
