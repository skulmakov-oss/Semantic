# Quad Language Examples

Status: proposed surface examples only

## Purpose

This document shows current core-form quad code next to proposed denser surface
forms.

The proposed forms are not claimed as implemented syntax unless the parser,
typechecker, and lowering rules explicitly admit them.

## Example 1: `quad_wave`

Current core form:

```semantic
fn quad_wave(index: i32) -> quad {
    let slot: i32 = index % 4;
    let state: quad = if slot == 0 {
        N
    } else if slot == 1 {
        F
    } else if slot == 2 {
        T
    } else {
        S
    };
    return state;
}
```

Proposed surface form:

```semantic
fn quad_wave(index: i32) -> quad {
    match index % 4 {
        0 => N,
        1 => F,
        2 => T,
        _ => S,
    }
}
```

## Example 2: `quad_probe_step`

Current core form:

```semantic
fn quad_probe_step(left: quad, right: quad) -> quad {
    let merged: quad = left || right;
    return merged;
}
```

Proposed surface form:

```semantic
fn quad_probe_step(left: quad, right: quad) -> quad = left || right;
```

## Example 3: readable quad predicate

Current core form:

```semantic
if merged == S {
    ...
}
```

Proposed surface form:

```semantic
if merged is S {
    ...
}
```

## Example 4: local type inference

Current core form:

```semantic
let left: quad = quad_wave(i);
let right: quad = quad_wave(i + 1);
let merged: quad = left || right;
let checksum: i32 = 0;
```

Proposed surface form:

```semantic
let left = quad_wave(i);
let right = quad_wave(i + 1);
let merged = left || right;
let checksum = 0;
```

## Example 5: expression-bodied function

Current core form:

```semantic
fn quad_probe_step(left: quad, right: quad) -> quad {
    let merged: quad = left || right;
    return merged;
}
```

Proposed surface form:

```semantic
fn quad_probe_step(left: quad, right: quad) -> quad = left || right;
```

## Example 6: `when` selection

Current core form:

```semantic
let state: quad = if ready == T {
    N
} else {
    S
};
```

Proposed surface form:

```semantic
let state: quad = when ready == T {
    N
} else {
    S
};
```

## Example 7: compact predicate chain

Current core form:

```semantic
if value == S {
    ...
} else {
    if value == N {
        ...
    } else {
        ...
    }
}
```

Proposed surface form:

```semantic
if value is S {
    ...
} else if unknown(value) {
    ...
} else {
    ...
}
```

## Warning

These examples preserve the `bool` / `quad` boundary. They do not approve
implicit `quad -> bool`, and they do not claim parser or verifier support by
themselves.
