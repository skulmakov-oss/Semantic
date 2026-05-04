# B-Wave Baseline — Imperative Core

Status: PR-B5 freeze document  
Program: Semantic application-completeness / snake benchmark path  
Scope type: docs/tests freeze  
Runtime-code changes: none

## Purpose

This document freezes the B-wave imperative-core baseline after the landed
application-completeness B-wave work.

The B-wave closes the ordinary imperative substrate required by benchmark-class
Semantic programs:

- same-family `i32` comparisons;
- same-family `i32` arithmetic;
- mutable locals;
- reassignment;
- condition-driven loops;
- open-ended loops;
- loop exits.

This document does not widen the public release contour and does not claim the
snake benchmark pack is complete.

## Landed Inputs

| Item | Status | Evidence |
|---|---:|---|
| PR-B1 — imperative baseline audit | landed | PR #407, merge SHA `c228b167` |
| PR-B2 — `i32 /` and `%` | landed | PR #408, merge SHA `d2226c19` |
| PR-B3 — `let mut` + reassignment | pre-program landed | confirmed by PR-B1 audit fixtures |
| PR-B4 — `while` | pre-program landed | confirmed by PR-B1 audit fixtures |
| PR-B4.5 — `loop` / `break` / `continue` | pre-program landed | confirmed by PR-B1 audit fixtures |

## Frozen Positive Surface

The following imperative surfaces are part of the current benchmark-relevant
positive baseline on `main`:

| Surface | Status |
|---|---:|
| `i32 > i32` | admitted |
| `i32 < i32` | admitted |
| `i32 >= i32` | admitted |
| `i32 <= i32` | admitted |
| `i32 + i32` | admitted |
| `i32 - i32` | admitted |
| `i32 * i32` | admitted |
| `i32 / i32` | admitted |
| `i32 % i32` | admitted |
| unary `-i32` | admitted |
| `let mut` mutable locals | admitted |
| plain reassignment | admitted |
| compound assignment over mutable locals | admitted |
| `while condition { ... }` | admitted |
| statement `loop { ... }` | admitted |
| bare `break;` inside loop | admitted |
| `continue;` inside loop | admitted |

## Frozen Runtime Contracts

### Division by zero

`i32 / i32` traps at runtime when the divisor is zero.

This must not host-panic. It is represented as a VM runtime trap.

### Modulo by zero

`i32 % i32` traps at runtime when the divisor is zero.

This must not host-panic. It is represented as a VM runtime trap.

### Arithmetic overflow

The VM must avoid host panic for overflow-sensitive integer division/modulo
cases such as `i32::MIN / -1` and `i32::MIN % -1`.

Those cases are handled through checked arithmetic and mapped to the runtime
trap taxonomy.

## Frozen Negative Surface

The following negative contracts remain intentionally covered:

- bare `break;` outside `while` / statement `loop` is rejected;
- `continue;` outside `while` / statement `loop` is rejected;
- division by zero fails at runtime;
- modulo by zero fails at runtime.

## Out Of Scope

B-wave does not add or claim:

- `u32` arithmetic completion;
- `f64` or `fx` widening beyond the existing contour;
- measured-numeric widening;
- text concatenation;
- formatting;
- stdout / print surface;
- file I/O;
- browser or UI ownership;
- snake benchmark completion.

## Remaining Application-Completeness Gaps

After B-wave, the active blockers for the benchmark family are outside the
imperative core:

- narrow admitted stdout experiment surface;
- canonical benchmark examples and close-out evidence.

## DoD

B-wave is considered frozen when:

- the application-completeness ledger marks PR-B1 through PR-B5 closed or
  pre-program landed where appropriate;
- the snake benchmark gap matrix carries positive fixtures for the admitted
  imperative surface;
- runtime-negative fixtures cover division/modulo by zero;
- no runtime-code changes are included in PR-B5;
- CI is green.
