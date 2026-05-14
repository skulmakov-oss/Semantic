# Semantic Hello World Shape

Status: decision-prep draft

See also:

- [`semantic_command_lexicon.md`](semantic_command_lexicon.md)
- [`semantic_style.md`](semantic_style.md)

## 1. Purpose

This document prepares the canonical Hello World shape decision for Semantic.

Hello World is required as proof of life.
This is not implementation.
This is not grammar finalization.
This decision depends on lexicon and density work.
The goal is to choose the preferred canonical direction for later
implementation planning.

## 2. Non-Goals

This document does not:

- change grammar
- change parser or typechecker behavior
- implement runtime / effect behavior
- change capability / effect admission
- implement `observe`
- implement `print`
- implement `entry` / `complete` / `require`
- implement Hello World
- implement a formatter
- rewrite README content
- rewrite examples
- rewrite fixtures
- rewrite tests
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Decision Inputs

Inputs from earlier work:

- Surface Audit: legacy form rejected as canonical
- LEXICON-B: `entry` / `complete` are preferred directions, non-executable
- LEXICON-C: `require` is preferred direction, non-executable
- LEXICON-D: `observe` is preferred direction, non-executable; `print` is
  rejected
- LEXICON-E: compact quad-style density guidance

## 4. Candidate Shapes

### A. Rejected legacy shape

```semantic
fn main() {
    print("Hello, World!");
    return;
}
```

Status: rejected-as-canonical

Reason: legacy imperative vocabulary, generic output, wrong model.

### B. Verbose Semantic directional shape

```semantic
entry HelloWorld {
    state boot: quad = T;
    require boot == T;
    observe "Hello, World!";
    complete T;
}
```

Status: preferred canonical direction / non-executable sketch

Reason: makes state, requirement, observation, and completion explicit.

### C. Compact Semantic directional shape

```semantic
entry HelloWorld:
    boot:quad = T
    require boot==T
    observe "Hello, World!"
    complete T
```

Status: density candidate / non-executable sketch

Reason: better visual density, but grammar not decided.

### D. Minimal observation-only shape

```semantic
entry HelloWorld {
    observe "Hello, World!";
}
```

Status: candidate but incomplete

Reason: good onboarding simplicity, but may hide state / require / completion
model.

## 5. Decision Table

| shape | status | strengths | risks | decision |
|---|---|---|---|---|
| legacy `fn main` / `print` / `return` | rejected-as-canonical | familiar to current executable bridge users | legacy imperative vocabulary, generic output, wrong model | reject as canonical public Hello World |
| verbose Semantic directional | preferred canonical direction / non-executable sketch | explicit state / requirement / observation / completion relation | more verbose than a future dense style | recommend as canonical direction for later implementation planning |
| compact Semantic directional | density candidate / non-executable sketch | better visual density, shorter example | grammar not decided, may be too early to standardize | keep as density experiment only |
| minimal observation-only | candidate but incomplete | simple onboarding, fewer moving parts | hides state / require / completion model | keep as secondary teaching shape only |
| bridge executable fallback | bridge-only / compatibility path | may preserve existing tests / fixtures | must not become public canonical Hello World | keep only for compatibility, not canonical |

## 6. Recommended Direction

Recommended canonical direction for later implementation planning:

```semantic
entry HelloWorld {
    state boot: quad = T;
    require boot == T;
    observe "Hello, World!";
    complete T;
}
```

This is:

- not executable yet
- not grammar-final
- accepted as preferred direction only
- an implementation later requires a scoped issue, likely `#477` or a
  successor
- exact density / brace / semicolon form may still be adjusted

## 7. Why Not Minimal Hello World

Semantic is not just a print language.

The first example should teach meaning -> requirement -> controlled observation
-> completion.

Minimal shape may be used later as a secondary onboarding form, but not as the
architecture-bearing canonical proof.

## 8. Why Not Legacy Bridge Fallback

Bridge fallback may remain for compatibility / test path only.
It must not be public canonical Hello World.
It undermines the surface audit decision.

## 9. Implementation Prerequisites

Before actual implementation, the following must be decided:

- grammar decision for `entry`
- grammar decision for `state`
- grammar decision for `require`
- grammar decision for `observe`
- grammar decision for `complete`
- verifier / admission policy for requirement and observation
- capability / effect policy for observation sink
- SemCode lowering plan
- VM / runtime behavior
- diagnostic plan
- tests / fixtures / golden SemCode plan
- CTF impact check
- README / examples alignment after implementation

## 10. Future Issue Handoff

- `#477` M-Hello remains blocked until this decision is accepted and
  implementation scope is opened.
- `#479` can close after this and final closeout if all lexicon / density
  deliverables are covered.
- Linguist readiness remains deferred.

## 11. Acceptance Checklist

- Hello World candidate shapes compared
- legacy shape rejected
- recommended canonical direction recorded
- non-executable status explicit
- implementation prerequisites listed
- `#477` remains blocked
- no grammar changes
- no implementation
- no README/examples rewrite
- no tests/fixtures changes
- no Linguist readiness
