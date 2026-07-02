# PCC Control-flow Core Contract

Status: PCC-CF-1 contract draft

This document defines the currently qualified Practical Core control-flow
contract for Semantic.

It is based on:

- `examples/canonical/match_control_flow`
- `examples/canonical/option_result_control_flow`
- `docs/roadmap/pcc/control_flow_core_audit.md`

## Scope

This contract covers:

- `if / else`
- `match`
- `return`
- `assert`
- `fn main()`
- explicit `quad` state handling
- `Option(T)` / `Result(T, E)` match flow

This contract does not cover:

- `while`
- `loop`
- `break`
- `continue`
- labeled loops
- fallthrough semantics
- exceptions
- async/concurrency

## Core Rules

### Rule CF-1 - `if` condition type

`if` conditions must be `bool`.

Allowed:

```semantic
if x > 0 {
    assert(true);
}
```

Forbidden:

```semantic
if state {
    assert(true);
}
```

`quad` values must not be implicitly treated as truthy or falsy.

Use explicit comparison or `match`.

### Rule CF-2 - `quad` control flow

`quad` state must be handled explicitly.

Allowed forms:

```semantic
if state == T {
    assert(true);
}
```

or:

```semantic
match state {
    T => { 1 }
    F => { 0 }
    N => { 2 }
    S => { -1 }
    _ => { -100 }
}
```

### Rule CF-3 - `match` fallback arm

Current admitted surface requires an explicit `_` fallback arm.

This is required even when all currently known quad states are listed
explicitly.

This rule is documented as current behavior, not necessarily final language
design.

### Rule CF-4 - terminal branch behavior

A branch may terminate by `return`.

Canonical practical examples should prefer terminal branches for `match`
samples until expression-valued match semantics are separately frozen.

### Rule CF-5 - `Option(T)` / `Result(T, E)` match

Current admitted constructor forms:

```semantic
Option::Some(x)
Option::None
Result::Ok(x)
Result::Err(code)
```

Current admitted type forms:

```semantic
Option(i32)
Result(i32, i32)
```

### Rule CF-6 - `fn main()`

Current admitted canonical examples use:

```semantic
fn main() {
    ...
}
```

not:

```semantic
fn main() -> i32 {
    ...
}
```

Program success or failure in canonical examples should be represented through
`assert` unless a specific runner contract says otherwise.

### Rule CF-7 - `assert`

`assert` is allowed in canonical examples as a self-check mechanism.

It must not be used as a replacement for verifier admission or type checking.

## Qualification Fixtures

Positive fixtures:

- `examples/canonical/match_control_flow/src/main.sm`
- `examples/canonical/option_result_control_flow/src/main.sm`
- `examples/canonical/loop_control_flow/src/main.sm`
- `examples/canonical/text_collections_toolbox/src/main.sm`

Negative fixtures to add later:

- `if quad_expr`
- `match missing _ arm`
- `missing terminal return path`
- constructor mismatch
- non-bool condition

## Open Questions

- Should full quad coverage eventually remove the need for `_`?
- Should `match` become expression-valued?
- Should `fn main() -> i32` be admitted later?
- Should `assert` be part of stdlib or core builtin?

## Policy Note

See [`match_fallback_arm_policy.md`](match_fallback_arm_policy.md) for the
current fallback-arm policy and future exhaustiveness direction.

See [`terminal_return_paths_policy.md`](terminal_return_paths_policy.md) for
the current terminal-return policy and missing-return qualification direction.

## Non-Goals

- No parser changes.
- No VM changes.
- No verifier admission changes.
- No loop syntax expansion.
- No broadening of `Option` / `Result` forms.
- No compile-fail infrastructure changes in this issue.
