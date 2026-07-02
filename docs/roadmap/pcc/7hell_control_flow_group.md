# PCC 7hell Control-flow Group

Status: PCC-CF-6 qualification group plan

This document defines the control-flow fixture group for `7hell`.

It is a documentation-first plan. It does not change `tools/7hell` yet. It
records the current positive and negative control-flow coverage so the runner
can be extended deliberately later.

## Goal

Define a `7hell` group that qualifies both accepted and rejected control-flow
behavior.

## Positive coverage

Canonical positive sources:

- `examples/canonical/match_control_flow/src/main.sm`
- `examples/canonical/option_result_control_flow/src/main.sm`
- `examples/canonical/loop_control_flow/src/main.sm`

These cover:

- `if / else`
- `match` over `quad`
- `match` over `Option(T)`
- `match` over `Result(T, E)`
- `while`
- `loop`
- `break;`
- `continue;`
- terminal returns
- canonical `fn main()` shape

## Negative coverage

Negative fixtures:

- `tests/fixtures/pcc/control_flow/fail/if_quad_condition.sm`
- `tests/fixtures/pcc/control_flow/fail/while_quad_condition.sm`
- `tests/fixtures/pcc/control_flow/fail/break_outside_loop.sm`
- `tests/fixtures/pcc/control_flow/fail/continue_outside_loop.sm`
- `tests/fixtures/pcc/control_flow/fail/match_missing_fallback.sm`
- `tests/fixtures/pcc/control_flow/fail/missing_return_path.sm`

These qualify:

- no implicit `quad` truthiness;
- loop control legality;
- current `_` fallback policy;
- current missing return behavior;
- no panic on bad input.

## Existing test coverage

Current coverage is provided by:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`
- `tests/pcc_control_flow_negative.rs`

## 7hell integration target

Current `7hell` integration is a fixed Hell 6 step that runs:

```bash
cargo test --test pcc_control_flow_negative
```

The runner is currently linear and hardcoded, so no separate group selector is
introduced here.

## Acceptance criteria

- control-flow positive examples are listed;
- control-flow negative fixtures are listed;
- current tests are linked;
- 7hell integration is documented as a fixed Hell 6 step;
- no duplicate fixture corpus is created unnecessarily.

## Out of scope

- adding new control-flow syntax;
- changing diagnostics;
- implementing exhaustiveness checking;
- changing loop semantics;
- introducing expression-valued `match`;
- modifying `tools/7hell` in this issue.
