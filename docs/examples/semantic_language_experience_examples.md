# Semantic Language Experience Examples

Status: proposed examples only

## Purpose

This document shows high-impact examples for the Semantic language-experience
track.

The examples are guidance, not claims about fully implemented features.

## Related Docs

- [`docs/language/semantic_language_experience.md`](../language/semantic_language_experience.md)
- [`docs/language/semantic_style.md`](../language/semantic_style.md)
- [`docs/roadmap/language_maturity/semantic_language_experience_roadmap.md`](../roadmap/language_maturity/semantic_language_experience_roadmap.md)

## Example 1: explicit admission story

```semantic
check source
compile SemCode
verify artifact
run program
```

## Example 2: visible conflict handling

```semantic
if value == S {
    // conflict remains visible
}
```

## Example 3: visible uncertainty handling

```semantic
if unknown(value) {
    // uncertainty remains explicit
}
```

## Example 4: positive quad handling

```semantic
match state {
    N => { ... }
    F => { ... }
    T => { ... }
    S => { ... }
    _ => { ... }
}
```

## Example 5: compact onboarding story

```semantic
let state: quad = when known(input) {
    match input {
        N => { S }
        F => { F }
        T => { T }
        S => { S }
        _ => { S }
    }
} else {
    S
}
```

## Example 6: explicit control boundary

```semantic
if conflict(signal) {
    // conflict stays visible
} else if signal == T {
    // control remains explicit
}
```

## Warning

These examples are framing material. They do not widen grammar, verifier, or
runtime behavior.
