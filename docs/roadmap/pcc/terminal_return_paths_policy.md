# PCC Terminal Return Paths Policy

Status: PCC-CF-3 policy note

This document records the current practical policy for terminal return paths in
Semantic control flow.

It is intentionally conservative. It does not change parser behavior. It does
not change lowering. It does not add diagnostics machinery. It records the
current practical contract so later work can separate policy from
implementation cleanly.

## Scope

This policy covers:

- `return`
- terminal `match` arms
- terminal `if / else` branches
- nested branches
- canonical examples using `assert`
- `fn main()` canonical shape

This policy does not cover:

- `while`
- `loop`
- `break`
- `continue`
- exceptions
- async/concurrency
- expression-valued `match`

## Current admitted behavior

Canonical examples currently use explicit terminal returns in branch-heavy
functions.

Examples:

- `examples/canonical/match_control_flow`
- `examples/canonical/option_result_control_flow`

These examples prefer explicit terminal branches such as:

```semantic
match state {
    T => return 1;
    F => return 0;
    N => return 2;
    S => return -1;
    _ => return -100;
}
```

rather than expression-valued `match`.

## Rule TRP-1 - `return` terminates function path

`return` exits the current function.

Any statements after an unconditional `return` are not part of the intended
practical path.

Unreachable-after-return diagnostics are not required by this policy unless
already implemented.

## Rule TRP-2 - branch-heavy canonical functions should be terminal

For current PCC canonical examples, branch-heavy `match` arms should prefer
explicit terminal returns.

This keeps the lowering and verifier expectations simple and visible.

## Rule TRP-3 - missing return paths must be qualified

For functions with non-void return type, missing return paths must be either:

- rejected by the current checker;
- trapped by the current execution contract;
- or recorded as a known gap.

PCC must not silently treat missing returns as stable behavior without
qualification.

## Rule TRP-4 - `fn main()` canonical shape

Current canonical examples use:

```semantic
fn main() {
    assert(...);
}
```

not:

```semantic
fn main() -> i32 {
    return 0;
}
```

This is a current admitted-surface rule, not necessarily final language
design.

## Positive qualification targets

- all `match` arms return
- nested `if` inside `match` returns
- `if / else` both return
- fallback `_` returns
- function continues after non-terminal `if` and returns at end

## Negative qualification targets

- missing return in one `if` branch
- missing `_` arm
- missing return after `match`
- unreachable statement after `return`
- wrong return type inside branch

## Current PCC decision

For current PCC:

```text
Canonical examples should use explicit terminal returns in branch-heavy
functions.
Missing return behavior must be documented before being treated as stable.
Expression-valued `match` remains out of scope.
```

## Follow-up work

- add positive terminal-return fixtures
- add negative missing-return diagnostics fixtures
- document current checker behavior
- decide whether unreachable-after-return should become warning or error

