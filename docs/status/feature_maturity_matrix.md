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
| i32 relational operators | D7 | High | Roadmap note, test evidence | Qualified limited release | Covers relational/equality-style operators, not general arithmetic. |
| Text equality | D7 | High | Roadmap note, test evidence | Qualified limited release | Same-family text equality only; does not imply formatting or concatenation. |
| Sequence indexing and iteration | D7 | High | Roadmap note, test evidence | Qualified limited release | Qualified for the benchmark-positive baseline. |
| First-class immutable closures | D7 | High | Roadmap note, test evidence | Qualified limited release | Immutable closure path only; mutable capture semantics are separate. |
| Runtime ownership OWN0 | D6 | High | Docs claim, VM/spec evidence | Stable release surface, frozen | Tuple and direct record-field access paths only. |
| Function contracts: `requires` / `ensures` | D5 | Medium | Docs claim, verifier spec | Implemented but unqualified | Should be split into syntax, typechecking, lowering, verifier integration, and runtime behavior in future status updates. |
| PROMETHEUS ABI / host boundary | D6 | Medium | Docs claim, CLI evidence | Implemented but unqualified | Should be split into ABI, capability policy, audit, VM host bridge, and real effects. |
| Units-of-measure surface | D2 | High | Docs claim, crate/status evidence | Experimental | Type/semantic surface only unless later evidence proves deeper pipeline support. |
| Integer arithmetic | D0/D1 | Medium | Roadmap blocker | Roadmap blocker | Parser evidence should be confirmed before claiming D1. Distinct from i32 relational operators. |
| Mutable locals / reassignment | D0/D1 | Medium | Roadmap blocker | Roadmap blocker | Parser evidence should be confirmed before claiming D1. Interaction with active borrow paths must be specified. |
| Loops and control exits | D0/D1 | Medium | Roadmap blocker | Roadmap blocker | Parser evidence should be confirmed before claiming D1. Requires bounded execution / quota discipline. |
| Map surface | D0 | High | Roadmap note | Roadmap | Not current stable behavior. |
| Deterministic seeded PRNG | D0 | High | Roadmap note | Roadmap | Must remain deterministic and replay-compatible when introduced. |
| Text concatenation / formatting | D0 | High | Roadmap note | Roadmap | Not implied by text equality. |
| General stdout | D0 | High | Roadmap / non-goal note | Roadmap | Narrow controlled observation work must not be read as general stdout. |
| ADT payload paths for ownership | N/A | High | Runtime ownership docs | Out of scope | Explicitly excluded from the current OWN0 slice. |

## Important distinctions

### i32 relational operators are not general integer arithmetic

The current status distinguishes between:

```text
i32 relational operators:
  ==, !=, <, <=, >, >=

integer arithmetic:
  +, -, *, /, %, overflow behavior, checked arithmetic contract
```

A relational operator being qualified does not imply that the full arithmetic
surface is qualified.

### Text equality is not text formatting

Text equality being qualified does not imply support for:

- text concatenation;
- formatted printing;
- implicit scalar-to-text conversion;
- general stdout.

### Controlled observation is not general stdout

The active controlled-observation path is intentionally narrow:

```text
verified SemCode
  -> VM controlled observation event
  -> capability gate
  -> audit decision
  -> CLI rendering envelope
```

It must not be read as broad I/O support.

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
