# Semantic Feature Maturity Matrix

Status: draft status document  
Scope: documentation/status discipline only  
Owner: project status / roadmap documentation

This document separates **architecture maturity**, **feature implementation**, and
**public release claims** for Semantic.

Semantic is being built as a **verifier-first deterministic execution platform**
with a semantic bytecode contract. The execution-core architecture is currently
more mature than the everyday practical language surface required for general
application development.

A feature being present on `main` does **not** automatically make it part of the
published stable release surface.

```text
landed on main != published stable
```

## Maturity levels

| Level | Meaning |
|---|---|
| D0 | Documented only / roadmap intent |
| D1 | Parsed by the source frontend |
| D2 | Typechecked / semantically accepted |
| D3 | Lowered to IR |
| D4 | Emitted to SemCode |
| D5 | Accepted by `sm-verify` |
| D6 | Executed by `sm-vm` |
| D7 | Qualified by tests, golden evidence, 7hell, or benchmark-positive evidence |
| N/A | Explicitly out of scope for the current release surface |

## Status vocabulary

| Status | Meaning |
|---|---|
| Stable release surface | Publicly claimed current contract |
| Qualified limited release | Works in a narrow qualified path, but does not widen all related features |
| Implemented but unqualified | Landed or partially working, but not yet full release evidence |
| Experimental | Present for evaluation or future shaping |
| Roadmap | Planned or tracked, not current stable behavior |
| Roadmap blocker | Needed for application completeness, but not yet qualified |
| Out of scope | Deliberately excluded from the current bounded contract |

## Current matrix

| Feature | Maturity level | Confidence | Evidence type | Status | Notes |
|---|---:|---|---|---|---|
| Native quad logic | D6 | High | Docs claim, VM/spec evidence | Stable release surface | `N / F / T / S` is a native semantic value domain. |
| i32 relational operators | D7 | High | Test evidence | Qualified limited release | Covers relational/equality-style operators (`==`, `!=`, `<`, `<=`, `>`, `>=`). |
| same-family i32 arithmetic | D7 | High | Test evidence | Qualified application-completeness contour | Covers `+`, `-`, `*`, `/`, `%` and unary `-`. Not published stable. |
| Mutable locals & reassignment | D7 | High | Test evidence | Qualified application-completeness contour | Supports `let mut` declarations and plain reassignments. Not published stable. |
| Loops and control exits | D7 | High | Test evidence | Qualified application-completeness contour | Supports `while` loops, statement `loop`, and exits `break`/`continue`. Not published stable. |
| Text concatenation and to_text | D7 | High | Test evidence | Qualified application-completeness contour | Supports `text + text` and explicit `to_text(scalar)`. Not published stable. |
| Sequence indexing and iteration | D7 | High | Test evidence | Qualified limited release | Qualified for sequence iteration and indexing. |
| First-class immutable closures | D7 | High | Test evidence | Qualified limited release | Immutable closure path only; mutable capture semantics are separate. |
| Map surface | D7 | High | Test evidence | Qualified application-completeness contour | Supports `Map(K, V)` functional get, set, contains. Not published stable. |
| Deterministic seeded PRNG | D7 | High | Test evidence | Qualified application-completeness contour | Deterministic seeded PRNG (xorshift64) via `random_seed` / `random_next_i32`. Not published stable. |
| Controlled observation (stdout) | D7 | High | Test evidence | Qualified application-completeness contour | Narrow `print(text)` via `CAP_STDOUT` capability. Not published stable. |
| Bounded project-root CLI baseline | D7 | High | Test evidence | Qualified application-completeness contour | Supports resolving and running routes from project root. Excludes registry, multi-package resolution, package manager semantics. Not published stable. |
| Runtime ownership OWN0 | D6 | High | Docs claim, VM/spec evidence | Stable release surface, frozen | Tuple and direct record-field access paths only. |
| Function contracts: `requires` / `ensures` | D5 | Medium | Docs claim, verifier spec | Implemented but unqualified | Requires syntax, typecheck, lowering, verifier and runtime qualification. |
| PROMETHEUS ABI / host boundary | D6 | Medium | Docs claim, CLI evidence | Implemented but unqualified | Needs qualification of ABI, capability policy, audit, and host bridge. |
| Units-of-measure surface | D2 | High | Docs claim, crate/status evidence | Experimental | Type/semantic surface only. |
| ADT payload paths for ownership | N/A | High | Runtime ownership docs | Out of scope | Explicitly excluded from the current OWN0 slice. |

## Important distinctions

### same-family i32 arithmetic is qualified

The current status includes qualified support for:

```text
same-family i32 arithmetic:
  +, -, *, /, %, unary -
```

This is distinct from multi-family numeric compatibility or implicit float conversions.

### Text concatenation is not general formatting

Text concatenation and explicit `to_text` are qualified, but general template formatting and implicit conversion of complex structures are roadmap/non-goals.

### Controlled observation is not general stdout

The active controlled-observation path is intentionally narrow:

```text
verified SemCode
  -> VM controlled observation event
  -> capability gate
  -> audit decision
  -> CLI rendering envelope
```

Narrow `print(text)` is qualified under the `CAP_STDOUT` capability, but general file I/O, command-line arguments (argv), or unconstrained stdout remain out of scope.

### OWN0 is intentionally narrow

The frozen runtime ownership slice supports:

- tuple access paths;
- direct record-field access paths;
- frame-local borrow lifetime;
- overlap rejection for exact, parent-child, and child-parent paths;
- sibling writes when paths do not overlap.

It explicitly does not support:

- ADT payload paths;
- schema paths;
- partial borrow release before frame exit;
- advanced alias / region reasoning;
- inter-frame borrow persistence;
- indirect projections.

## Documentation rule

README, examples, and public docs should avoid presenting roadmap or
unqualified features as stable behavior.

A feature should be promoted in public-facing documentation only when it has:

- a documented contract;
- test or golden coverage;
- verifier / VM evidence where applicable;
- CLI-visible behavior where applicable;
- explicit inclusion in the current release surface.

## Current project framing

Semantic should be described as:

```text
an emerging verifier-first deterministic execution platform
under active Practical Core Completion,
with a limited qualified release surface.
```

It should not be described as a mature general-purpose application language yet.
