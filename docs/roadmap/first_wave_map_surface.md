# First-Wave Map Surface

Status: PR-D1 scope document
Program: Semantic application-completeness / snake benchmark path
Implementation target: PR-D2
Scope type: docs-only contract

## Purpose

Define the narrow `Map(K, V)` surface needed for benchmark-class application
state:

- Q-tables
- visit counters
- deterministic lookup/update loops
- snake benchmark state storage

This document does not implement Map. It defines the allowed implementation
boundary for PR-D2.

## Non-Goals

This first wave must not introduce:

- a general collection framework
- `Set`
- ordered maps
- hash customization
- iterators over maps
- map comprehensions
- reference/borrow semantics
- in-place mutation through aliases
- host-dependent nondeterminism
- floating-point keys
- record keys unless explicitly deferred
- ADT keys unless explicitly deferred

## Source Surface

Admitted type form:

```semantic
Map(K, V)
```

Admitted construction candidate:

```semantic
let q: Map(text, i32) = map_empty();
```

PR-D2 must choose one canonical construction spelling. If the repository later
requires a different surface constructor spelling, this document should be
updated before implementation starts.

## Required First-Wave Operations

Minimum surface for PR-D2:

| Operation      | Signature                                     | Meaning                            |
| -------------- | --------------------------------------------- | ---------------------------------- |
| `map_empty`    | `map_empty() -> Map(K, V)` by contextual type | create empty deterministic map     |
| `map_contains` | `map_contains(Map(K, V), K) -> bool`          | key presence check                 |
| `map_get`      | `map_get(Map(K, V), K, V) -> V`               | lookup with explicit default value |
| `map_set`      | `map_set(Map(K, V), K, V) -> Map(K, V)`       | persistent update returning map    |

## Semantics

First-wave Map is persistent / functional:

```semantic
q = map_set(q, key, value);
```

`map_set` returns a new `Map(K, V)` value. It does not mutate the previous map
in place.

This mirrors persistent sequence helpers:

```semantic
xs = push(xs, value);
xs = pop(xs);
```

## Determinism Requirements

Map behavior must be deterministic:

- same input program
- same SemCode
- same runtime configuration
- same operation order

must produce the same result.

No host-randomized hash behavior may leak into observable semantics.

## Key Policy

First-wave admitted key types should be narrow and deterministic.

Recommended PR-D2 admitted keys:

- `i32`
- `u32`
- `bool`
- `text`
- `quad`

Explicitly defer:

- `f64`
- `fx`
- records
- ADTs
- sequences
- maps as keys

Rationale:

- `contains(sequence, value)` already uses a scalar comparable admitted set.
- Map keys require stable equality and stable ordering/hash semantics.
- Complex keys can be added later after a separate equality/key contract.

## Value Policy

Values do not require equality.

Recommended PR-D2 admitted values:

- any currently constructible `V` that can be stored as a runtime `Value`

If implementation risk is high, PR-D2 may start with scalar values only, but
this document should state the preferred direction and require explicit
justification for narrowing.

## Error Policy

Required type errors:

- non-map first argument to map operations
- key type mismatch
- value type mismatch for `map_set`
- wrong arity
- named arguments if current builtin convention rejects them

Runtime errors should be avoided for normal missing-key lookup by using an
explicit default value:

```semantic
let score: i32 = map_get(q, state_key, 0);
```

## Snake Benchmark Use Cases

Example Q-table:

```semantic
let mut q: Map(Text, i32) = map_empty();

let key: Text = "state/action";
let old: i32 = map_get(q, key, 0);
q = map_set(q, key, old + 1);
```

Example visit counter:

```semantic
let mut visits: Map(Text, i32) = map_empty();

let count: i32 = map_get(visits, state_key, 0);
visits = map_set(visits, state_key, count + 1);
```

## PR-D2 Acceptance Criteria

PR-D2 may start only after this scope is merged.

PR-D2 must include:

- parser/typecheck support as needed
- lowering/IR/SemCode support as needed
- verifier capability requirements
- VM execution
- positive tests
- negative type tests
- snake benchmark matrix update

PR-D2 must not:

- introduce broad collection abstractions
- implement Set
- implement map iteration
- add host-nondeterministic behavior
- claim full application-completeness
