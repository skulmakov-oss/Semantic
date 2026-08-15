# Source Type Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current public source-level type contract for
Semantic programs.

It covers the executable source surface rather than the SemCode or VM
representation layer.

Compile-time-only declaration families such as `schema Name { ... }` are part
of the source contract, but they are not yet executable source-visible types or
VM value families.

Operational source-level meaning such as call resolution, control-flow
selection, and source diagnostics is specified separately in:

- `source_semantics.md`
- `diagnostics.md`

## Current Type Family

Current source-visible types:

- `quad`
- `bool`
- `text`
- `i32`
- `u32`
- `f64`
- `fx`
- measured numeric forms such as `f64[m]` and `u32[ms]`
- `unit`
- `qvec(N)` as a reserved parser-level family

Current compile-time-only declaration families:

- nominal `schema Name { ... }` declarations for boundary/model contracts
- record-shaped and tagged-union schema forms within that compile-time-only
  declaration family
- explicit schema-role metadata via `config schema`, `api schema`, and
  `wire schema`
- optional schema-version metadata via `version(<u32>)`
- deterministic record-schema compatibility reports across two explicit schema
  versions with first-wave classes `Equivalent`, `Additive`, and `Breaking`
- deterministic tagged-union schema compatibility reports across two explicit
  schema versions with the same first-wave classes
- canonical schema migration metadata artifacts and stable review formatting
  derived from those compatibility reports
- deterministic compile-time validation plans derived from canonical schema
  declarations and referenced declared types
- first-wave record-schema validation checks for required fields and field-type
  compatibility, kept in declaration order for inspectability
- first-wave tagged-union schema branch checks for allowed variants, required
  per-branch fields, and per-branch field-type compatibility, kept in variant
  declaration order for inspectability
- deterministic generated API contract artifacts derived only from canonical
  `api schema` and `wire schema` declarations
- generated API artifacts preserve declaration order and expose explicit
  format-version and generator metadata for reproducible review
- deterministic generated wire-contract artifacts derived only from canonical
  `wire schema` declarations
- generated wire-contract artifacts currently expose:
  - tagged wire unions from tagged-union `wire schema`
  - wire patch types from record-shaped `wire schema`
- generated wire-contract artifacts preserve declaration order and expose
  explicit format-version and generator metadata for reproducible review

## Text

Current honest baseline:

- the published stable `v1.1.1` line does not expose `text` as an executable
  source-visible type family
- current `main` now admits `text` in declared source type positions in the
  Rust-like executable path
- current `main` also admits a narrow double-quoted UTF-8 text literal family
  in the same source path
- current `main` admits same-family equality on `text`
- current `main` now admits a canonical runtime text carrier for admitted
  literal/equality/concatenation/to_text programs
- current `main` now admits `text + text` concatenation and explicit `to_text`
  for the admitted scalar families represented by existing fixtures
- current `main` does not widen the PROMETHEUS host ABI with text values

Current text-surface limits:

- the current literal spelling is narrow: double-quoted same-line UTF-8 text
  only
- interpolation, formatting, escape-rich string syntax, and host/runtime text
  ABI widening are not part of the current contract

## First-Class Closures

Current honest baseline:

- the published stable `v1.1.1` line does not claim first-class closure values
  or closure types
- current `main` now owns one first-wave closure family in the frontend owner
  layer
- current `main` now admits declared `Closure(T -> U)` type positions and
  standalone closure literals in contextual closure-typed positions
- current `main` now admits one canonical runtime closure carrier for the same
  first-wave family
- current `main` now admits direct invocation of admitted closure values

Current first-wave limits:

- current `main` admits immutable capture inventory for that first-wave family
- immutable capture only remains the intended direction for the first-wave
  family
- direct invocation only is the intended call model for that family
- direct invocation currently requires exactly one positional argument
- closure equality is not part of the current contract
- generic callable abstractions, traits/protocols, async closures, and host-ABI
  closure transport are not part of the current contract

## Unit

`unit` is the implicit type of functions without an explicit return type.

Current rule:

- `fn main()` must currently have implicit `unit` return and no parameters in
  the canonical executable path

## Quad

`quad` is a first-class semantic logic type with four values:

- `N`
- `F`
- `T`
- `S`

Current rules:

- `quad` participates in equality and implication
- `match` currently operates on `quad`, nominal enum, `Option(T)`,
  `Result(T, E)`, `i32`, and `u32` scrutinees; every other scrutinee type
  is rejected deterministically (SSF-07). All six families are fully
  executable, including `u32` (see
  `docs/spec/foundation_source_profile_v1.md`'s "Data and patterns" section
  for the exact admitted literal/range bound forms)
- `quad` is not accepted directly as an `if` condition
- the user-facing quad predicate vocabulary is documented in
  [`docs/language/quad_language_design.md`](../language/quad_language_design.md)

### Quad lexical model

The current lexical model keeps quad tokens explicit and compact:

- `N` means unknown
- `F` means false
- `T` means true
- `S` means conflict

These values are semantic data, not truthiness aliases.
`S` is not silently normalized away, and `N` is not collapsed into `F`.

### Quad operation families

The source contract groups quad operators into three families:

- identity predicates: `==` and `!=`
- evidence operators: `&&`, `||`, `!`, `->`
- explicit control predicates: comparisons such as `q == T`, `q == F`, and
  quad selection through `match`

The families are intentionally distinct:

- identity predicates return `bool`
- evidence operators return `quad`
- control predicates return `bool` and are the only path into branch selection

### Quad predicate vocabulary

Current source-level branch vocabulary stays explicit:

- `if q == T { ... }`
- `if q == F { ... }`
- `match q { N => ... F => ... T => ... S => ... _ => ... }`

Design notes for future quad-heavy surface syntax may use `when` or `is` as
compact vocabulary, but the current contract still requires explicit
comparison or `match`.

### Quad equality

`quad == quad` is a structural identity predicate.

It returns `bool`, not `quad`.

Examples:

- `N == N` -> `true`
- `F == F` -> `true`
- `T == T` -> `true`
- `S == S` -> `true`
- `N == S` -> `false`
- `T == S` -> `false`
- `T == F` -> `false`

This operator does not perform semantic inference, evidence merging,
consensus detection, or conflict resolution.

`==` and `!=` are identity predicates.
They always return `bool`.
They never return `quad`.
They never drive control flow through implicit `quad` coercion.

Identity predicates answer whether two values have the same source/runtime
state.

Evidence operators operate on T/F evidence planes and return `quad`.

Therefore:

- `S == S` -> `true : bool`
- `S && S` -> `S : quad`
- `S || T` -> `S : quad`
- `if S` -> rejected
- `if signal == S` -> admitted because the condition is `bool`

| Family | Example | Result type | Meaning |
| --- | --- | --- | --- |
| Identity predicate | `a == b` | `bool` | same value state |
| Evidence algebra | `a && b`, `a || b`, `!a`, `a -> b` | `quad` | T/F evidence-plane operation |
| Control-flow predicate | `a == T`, `a == S` | `bool` | explicit boolean condition |
| Readable quad predicate | `a is S`, `known(a)`, `unknown(a)`, `conflict(a)` | `bool` | surface alias for an existing bool predicate |
| Future semantic helper | `quad_consensus(a, b)` | explicit API-defined type | policy-defined analysis |

## Standard Forms

Current first-wave standard forms:

- `Option(T)` is the canonical optional-value type family in declared type
  positions
- `Result(T, E)` is the canonical success/error type family in declared type
  positions
- these forms are language-owned standard families, not user-defined generic
  declarations
- they currently lower through the same canonical aggregate carrier family used
  by nominal ADTs
- explicit `Option::Some/None` and `Result::Ok/Err` patterns participate in
  the stable match surface over these families

## Bool

`bool` is the ordinary binary condition type.

Current rules:

- `if` conditions must evaluate to `bool`
- `!`, `&&`, and `||` are valid on `bool`
- equality comparisons on `bool` are valid

## I32 And U32

`i32` and `u32` are the current integer-oriented execution types.

Current rules:

- arithmetic operators are expected to stay within the same numeric family
- equality comparisons are valid inside the same family
- plain same-family `i32` relational comparisons `>`, `<`, `>=`, `<=` are now
  admitted on current `main`
- plain same-family `i32` unary `-` and binary `+`, `-`, `*`, `/`, `%` are now
  admitted on current `main`
- division and modulo by zero remain runtime failure edges exercised by the
  current benchmark-negative fixtures
- `u32` remains equality-only in the current first application-completeness wave
- implicit cross-family numeric coercion is not part of the current contract

## F64

`f64` is the current floating-point math family.

Current rules:

- `f64` availability is gated by the active parser profile / compile policy
- arithmetic operators `+`, `-`, `*`, `/` are supported on `f64`
- equality comparisons are supported on `f64`
- current builtin math calls include:
  - `sin`
  - `cos`
  - `tan`
  - `sqrt`
  - `abs`
  - `pow`

## Fx

`fx` is the fixed-point-oriented numeric family.

Current rules:

- the canonical `fx` value path is end-to-end
- explicit `fx` annotations are supported
- `fx` currently accepts explicit literals and existing `fx`-typed values in
  the public Rust-like path
- contextual literal admission into `fx` is supported only where the expected
  type is already `fx`
- on current `main`, plain `fx` unary `+` / `-` and plain binary `+`, `-`,
  `*`, `/` between already-typed `fx` operands are admitted by source typing as
  part of the post-stable expansion track
- stable `fx` behavior in the current line is value transport plus equality, not
  full arithmetic parity with `f64`

Current honest limits:

- the published stable `v1.1.1` line still remains narrower than the `f64`
  arithmetic surface
- canonical lowering/verified execution for the widened plain `fx` arithmetic
  surface has now landed on current `main`
- emitted plain `fx` arithmetic programs use a promoted `SEMCODE3` header line
  instead of widening the older `SEMCODE2` artifact contract in place
- coercion from non-literal non-`fx` expressions is not yet the full intended
  long-term contract
- unary `+` and unary `-` on `fx` are admitted only for literal formation in
  the published stable `v1.1.1` contract; the widened post-stable path is
  described separately
- completed first-wave post-stable widening for general-purpose `fx`
  arithmetic is documented in
  `docs/roadmap/language_maturity/fx_arithmetic_full_scope.md`

## Units Of Measure

First-wave units of measure are source-level refinements over the existing core
numeric families.

Current supported forms:

- `i32[unit]`
- `u32[unit]`
- `f64[unit]`
- `fx[unit]`

Current rules:

- the bracket payload is a single unit symbol
- measured numeric types may appear in locals, parameters, returns, tuple
  elements, record fields, `Option(T)`, and `Result(T, E)` payload positions
- assignment, call, return, and pattern-binding transport require exact base
  type and exact unit-symbol equality
- `==` and `!=` are valid on any measured type (any base) when both operands
  have the same base type and the same unit symbol
- binary and unary `+`/`-` currently typecheck only for measured `f64` with
  matching operands; measured `fx` binary and unary `+`/`-` report an
  explicit narrow-slice gap, and measured `i32`/`u32` binary and unary `+`/`-`
  are rejected as unsupported operators in the first-wave surface
- lowering erases the unit annotation after semantic validation and reuses the
  existing numeric execution carrier

Current honest limits:

- units are not part of the VM value representation or public host ABI shape
- implicit conversions between unit symbols are not part of the contract
- compound unit algebra such as `m/s`, `N*m`, or exponent notation is not part
  of the first-wave surface
- `*`, `/`, and `%` on unit-carrying values are intentionally rejected in the
  first-wave contract
- `+`/`-` on unit-carrying `i32`/`u32` values are intentionally rejected; SSF-07
  selected this as the current boundary rather than leaving it undecided, and
  any future widening to measured `i32`/`u32` arithmetic is a later decision

## QVec

`qvec(N)` exists as a parser-level family and should be treated as reserved or
partial rather than fully stabilized in the current public source contract.

Until the repository documents a fuller execution and library story for
`qvec(N)`, it should not be treated as a broadly stable user-facing type family.

## Equality And Control Rules

Current equality and control rules:

- `==` and `!=` require meaningful same-family comparisons
- `if` requires `bool`
- `match` requires `quad`, nominal enum, `Option(T)`, `Result(T, E)`, `i32`,
  or `u32`, all fully executable (see "Quad" above and
  `docs/spec/foundation_source_profile_v1.md`'s "Data and patterns" section)

## Function Typing Rules

Current function typing rules:

- every parameter has an explicit type
- return type defaults to `unit` if omitted
- `return expr;` must match the declared return type
- `return;` is valid only for `unit`-returning functions

## Builtin Typing Rule

Builtin functions are part of the public type contract and are checked as typed
calls, not as dynamically typed escape hatches.

That means:

- argument count must match
- argument types must match the builtin signature
- builtin typing failures are ordinary source-level type errors

## Current Exclusions

The current source type contract does not yet claim stable support for:

- schema names as executable local, parameter, or return types
- user-defined parameterized algebraic data types
- generics
- trait or protocol systems
- implicit numeric widening across unrelated families
- a broad collection type ecosystem

Current active collections checkpoint on `main`:

- `docs/roadmap/language_maturity/collections_surface_full_scope.md`
- current `main` now admits one ordered sequence collection family through
  `Sequence(type)` in declared source type positions
- current `main` now admits bracketed ordered sequence literals in the
  Rust-like source path
- current `main` now admits same-family equality for ordered sequence values
  when the item type already supports stable equality
- current `main` now admits a canonical runtime carrier for the same ordered
  sequence family
- current `main` now admits `expr[index]` when the base is an admitted ordered
  sequence and the index is `i32`
- current `main` now admits `for value in sequence { ... }` for that sequence
  family through the current first-wave iterable loop surface
- current `main` still does not admit `len` or `is_empty` for that sequence
  family in the current `M8.3` first-wave surface

Current active closures checkpoint on `main`:

- `docs/roadmap/language_maturity/first_class_closures_full_scope.md`
- published `v1.1.1` still does not claim first-class closure values or
  closure types in the public source type contract
- current `main` now admits one first-wave runtime closure carrier and direct
  invocation for admitted closure values

## Contract Rule

Any public change to source-visible type meaning or source type-checking rules
should update this document in the same change series.
