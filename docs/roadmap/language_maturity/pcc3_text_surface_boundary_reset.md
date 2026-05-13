# PCC-3 Text Surface Boundary Reset

Status: pre-entry guard
Track: PCC-3-0
Layer: language maturity / readiness
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `docs/roadmap/language_maturity/practical_core_completion_v0_3.md`
- `#477`
- `#478`
- `#479`
- `#356`

## 1. Purpose

This document resets the text/string surface boundary before PCC-3.

PCC-3 may work on text/string mechanics, but it must not canonize legacy surface vocabulary while doing so. The purpose of this reset is to keep Semantic from drifting into a C/Rust-like source surface with strings by accident.

Hello World remains required as proof of life. It is not a general stdout story; it is controlled observation, bounded by the current surface and the language maturity boundary.

## 2. Current phase state

| Phase | State |
|---|---|
| PCC-1 Control Flow Core | closed |
| PCC-2 Numeric Core | closed |
| PCC-3 Text/String Core | not started |
| PCC-3-0 Text Surface Boundary Reset | active / pre-entry guard |

## 3. Boundary rules

- Text/string mechanics may be stabilized in PCC-3.
- Public surface vocabulary is not frozen by PCC-3A tests.
- `fn main`, `print`, `return`, `assert` must not be treated as canonical Semantic vocabulary.
- Legacy forms may remain bridge-only if currently needed by the existing frontend.
- Canonical direction must be decided through `#478` and `#479`.
- Hello World / observation boundary remains blocked until the text surface boundary is clear.
- Linguist recognition remains deferred.

## 4. Hello World decision

Hello World is required.
It is not optional.
It is the proof-of-life path for mature engineers and external readers.

But the canonical Hello World must not be:

```text
fn main() {
    print("Hello, World!");
    return;
}
```

That form is explicitly not canonical Semantic surface.

Instead, the direction is:

```text
entry HelloWorld {
    state boot: quad = T;
    require boot == T;
    observe "Hello, World!";
    complete T;
}
```

This is semantic-native direction / future canonical direction unless and until the grammar and implementation already support it. This document does not claim that syntax is executable.

## 5. Vocabulary boundary table

| Legacy / bridge term | Semantic-native direction | Status before audit |
|---|---|---|
| `fn main` | `entry` | bridge / not canonical |
| `print` | `observe` | bridge / not canonical |
| `return` | `complete` | bridge / not canonical |
| `assert` | `require` | bridge / not canonical |
| `stdout` | observation sink | not general I/O |
| `I/O` | controlled observation/effect | capability-bound |
| `run` | execute / transition / evaluate | layer-dependent |

## 6. Relationship to future issues

- `#478` should audit legacy surface vocabulary.
- `#479` should define command lexicon and density/style rules.
- `#477` should be revisited only after this boundary is accepted.
- `#356` / Linguist readiness remains deferred until public surface identity stabilizes.

## 7. PCC-3 entry rule

PCC-3A may start only after this PR lands.

PCC-3A may add text/string core gate fixtures, but it must not:

- implement Hello World;
- add general observation;
- canonize `print`, `main`, `return`, or `assert`;
- start Linguist readiness;
- touch UI / Workbench / I70.

## 8. Explicit non-goals

This PR does not:

- implement text/string features;
- implement `observe` / `print`;
- implement Hello World;
- implement new grammar;
- rename current syntax globally;
- modify examples;
- modify tests;
- modify frontend/parser/typechecker;
- start `#477`;
- close `#478` or `#479`;
- start Linguist recognition;
- start UI / Workbench / I70;
- start package builder.

## 9. Acceptance checklist

- PCC-3 pre-entry boundary recorded
- Hello World requirement acknowledged
- legacy Hello World form rejected as canonical
- semantic-native observation direction recorded
- `#477` remains blocked / dependent
- `#478` / `#479` remain future surface decision issues
- Linguist readiness remains deferred
- no code / test / fixture changes
- no grammar changes
- no UI / Workbench / I70
- PCC-3A not started

## 10. Boundary summary

```text
Hello World is required.
Legacy Hello World is not canonical.
Text mechanics may proceed.
Observation vocabulary must remain guarded.
```
