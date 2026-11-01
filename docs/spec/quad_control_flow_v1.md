# Quad-Native Control Flow v1

Status: proposed normative contract
Owner: `sm-front`, `sm-ir`, `sm-format`, `sm-verify`, and `sm-vm`

This contract extends control flow without changing the existing Boolean
instruction meanings.  It is versioned independently from Quad Logic Frame v1.

## Value model

`quad` has the canonical two-plane encoding:

| Value | Bits | Meaning |
|---|---:|---|
| `N` | `00` | unknown / insufficient evidence |
| `F` | `01` | strict false |
| `T` | `10` | strict true |
| `S` | `11` | conflict |

There is no implicit conversion from `quad` to `bool`.  In particular, `N` is
not false and `S` is not true.  Neither state may authorize an effect.

## Conditions

Boolean conditions retain their existing behavior.  At the approved Quad
control sites (`if`, `while`, and `assert`), a `quad` value is consumed
directly:

| Value | `if` | `while` | `assert(quad)` |
|---|---|---|---|
| `T` | then branch | next iteration | success |
| `F` | else branch, or continue if absent | normal exit | `AssertionFailed` |
| `N` | `QuadConditionUnknown` trap | `QuadConditionUnknown` trap | `QuadAssertionUnknown` trap |
| `S` | `QuadConditionConflict` trap | `QuadConditionConflict` trap | `QuadAssertionConflict` trap |

The condition is evaluated once per `if` decision or loop test.  A trap is
raised before either normal branch body executes and therefore before any
effect reachable only through that body.

`requires`, `ensures`, `invariant`, guards, and all other condition sites keep
their existing Boolean contract in v1.

## IR and SemCode boundary

The IR uses typed operations equivalent to `QuadBranch4` and `QuadAssert`.
Their operands are explicitly Quad-typed; a Boolean branch cannot be used as a
substitute.  The conceptual `QuadBranch4` successors are ordered `(N, F, T,
S)` and exactly one successor is selected.

The SemCode encoding is revision-gated.  Legacy `JMP_IF` and `ASSERT` retain
their current meanings.  An implementation may add new opcodes only after the
format owner assigns non-colliding values and the decoder, verifier, and Rust
VM implement the same revision gate.  Older decoders must reject the new
instructions rather than reinterpret them.

## Admission and execution

The verifier must validate Quad operand types, all successor boundaries,
reachable CFG edges, definite register assignment, and existing resource
limits.  Well-formed bytes are not semantic authorization.  The Rust VM is the
reference execution oracle and must dispatch the four states directly.

## Compatibility

Existing Boolean programs and legacy SemCode remain unchanged.  A compiler or
VM that cannot implement the complete contract must reject the unsupported
program or artifact deterministically; it must not project Quad values into
Boolean truthiness.
