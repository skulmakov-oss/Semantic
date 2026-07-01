# Source Diagnostics Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current public diagnostics contract for Semantic
source programs.

It covers source-facing parse, policy, type, and module/linkage diagnostics. It
does not redefine verifier, VM, or host-ABI error reporting.

## Diagnostic Layers

The current source toolchain exposes diagnostics from distinct layers:

1. Rust-like and Logos parsing in `sm-front`
2. source-level type checking in `sm-front`
3. module/import/export and Logos semantic checks in `sm-sema`

These layers are all source-facing, but they do not yet share one fully frozen
diagnostic code taxonomy.

## Frontend Parse Diagnostics

The parser already emits rendered source diagnostics with code-bearing messages
for many syntax and Logos-surface failures.

Current examples include:

- `E0200` aggregated Logos parse failures
- `E0201` and nearby codes for `System` parsing
- `E0210` to `E0216` for `Entity` parsing
- `E0220` to `E0233` for `Law` and `When` parsing
- `E0234` to `E0237` for type/expression-level parser expectations
- block-expression parse failures such as missing trailing tail values
- `if`-expression parse failures such as missing `else` in value position
- expression-bodied function parse failures such as missing trailing `;`
- pipeline parse failures such as missing function-stage targets after `|>`
- short-lambda parse failures such as standalone non-invoked lambdas
- short-lambda surface failures such as rejected outer-local capture in v0
- `guard`-clause parse failures such as missing `else return`
- `let-else` parse failures such as plain binding targets, discard targets, or
  non-`return` else branches in the current v0 surface
- `match`-expression parse failures such as invalid literal arm patterns
- top-level parse failures such as unexpected items other than `fn`, `record`,
  `schema`, or `enum`
- malformed role-marked schema declarations such as `config`/`api`/`wire`
  appearing without a following `schema`
- malformed schema version markers such as missing parentheses, non-decimal
  forms, suffixed literals, or non-positive values
- extended numeric-literal parse failures such as invalid typed suffix/body
  combinations or decimal-only `f64`/`fx` requirements

Current guarantees:

- parser diagnostics include source position
- many parser diagnostics include rendered line/column context
- keyword mistakes may include case-insensitive suggestions

Current honest limit:

- the repository does not yet claim that every parser diagnostic code or exact
  wording is frozen as a long-term compatibility promise
- current `main` now admits executable double-quoted text literals and `text`
  type spelling in the source parser path
- exact parse wording for text-surface gaps is not yet a long-term frozen
  compatibility promise
- current `main` now admits standalone first-class closure literal syntax in
  contextual source positions
- current `main` now also admits direct invocation and verified runtime
  execution for one first-wave closure family
- exact parse/type wording for closure-surface gaps is not yet a long-term
  frozen compatibility promise

## Policy Diagnostics

The frontend distinguishes ordinary syntax failures from policy rejections.

Current rules:

- policy-gated rejections are surfaced as `policy violation: ...`
- `FrontendErrorKind` currently distinguishes `Syntax` from
  `PolicyViolation`
- policy rejections cover source features disabled by the active parser profile

Current examples include:

- `f64` surface disabled by profile policy
- Logos surface disabled by profile policy
- schema surface disabled by profile policy
- legacy compatibility paths disabled by profile policy

## Type Diagnostics

The Rust-like type checker currently reports source-level type failures as plain
messages rather than as a stable numeric code family.

Current message families include:

- unknown variable
- unknown assignment target
- assignment to const binding
- unknown function
- argument count mismatch
- argument type mismatch
- invalid `assert` argument count
- invalid `assert` condition type
- statement-only `assert` used in value position
- non-bool `requires` condition
- unsupported expression form inside `requires`
- non-bool `ensures` condition
- unsupported expression form inside `ensures`
- non-exhaustive enum `match`
- `quad` match without explicit default arm
- reserved `result` parameter name while `ensures` clauses are present
- non-bool `invariant` condition
- unsupported expression form inside `invariant`
- reserved `result` parameter name while `invariant` clauses are present
- `result` referenced from `invariant` on a unit-return function
- positional arguments after named arguments
- named arguments on builtin calls
- unknown named parameter
- duplicate named argument
- missing named argument for a declared parameter
- required parameter after default parameter
- non-const-safe default parameter initializer
- default parameter type mismatch
- non-bool `requires` condition
- unsupported expression form inside `requires`
- let-binding type mismatch
- discard-binding type mismatch
- invalid unit annotation on non-numeric type family
- measured numeric type mismatch across binding/call/return transport
- `*` / `/` requested on unit-carrying values in the first-wave surface
- `Option::None` without contextual `Option(T)` type
- `Result::Ok(...)` / `Result::Err(...)` without contextual `Result(T, E)` type
- non-const-safe initializer in const declaration
- invalid typed numeric literal form
- range literal requires `i32` bounds
- range equality not part of stable v0 range surface
- range literal rejected in tuple/user-data position
- first-class closure literals without contextual `Closure(T -> U)` type
- direct invocation of first-class closure values requires exactly one
  positional argument in the current `M8.4` first-wave surface
- named arguments on direct closure invocation are rejected in the current
  `M8.4` first-wave surface
- closure equality is not part of the current `M8.4` first-wave surface
- for-range requires `i32` range expression
- relational operators beyond plain same-family `i32` operands remain outside the
  first application-completeness wave
- iterable `for x in collection` loop over a non-iterable value
- iterable `for x in collection` loop whose explicit `Iterable` impl does not
  satisfy the executable contract `fn next(self: Self, index: i32) -> Option(Item)`
- iterable `for x in collection` loop that relies on ADT/schema or indirect
  explicit `Iterable` dispatch beyond the direct-record slice on current `main`
- duplicate record declaration
- duplicate schema declaration
- top-level record/function name collision
- top-level schema/function name collision
- top-level schema/record name collision
- top-level schema/enum name collision
- empty record declaration
- empty schema declaration
- duplicate record field name
- duplicate schema field name
- duplicate tagged-union schema variant name
- duplicate tagged-union schema field name
- unknown record field type
- unknown schema field type
- generated validation-plan failure for missing required record field
- generated validation-plan failure for incompatible record field type
- generated validation-plan failure for disallowed tagged-union branch
- generated validation-plan failure for missing required tagged-union branch field
- generated validation-plan failure for incompatible tagged-union branch field type
- generated API contract build failure while parsing or validating canonical
  source/schema declarations
- generated API contract build failure while canonicalizing declared field or
  payload types into artifact-surface type text
- generated wire-contract build failure while parsing or validating canonical
  source/schema declarations
- generated wire-contract build failure while canonicalizing tagged-union
  payload or patch-field types into artifact-surface type text
- non-`bool` `while` condition
- `break expr;` outside `loop` expression bodies, including inside `while`
  statements
- bare `break;` outside admitted `while` or statement-loop context
- `continue;` outside admitted `while` or statement-loop context
- recursive record field graph
- record type declared but not yet available in executable parameter/return annotation positions
- duplicate field in record literal
- unknown field in record literal
- missing field in record literal
- unknown field in record field access
- record field access on non-record value
- duplicate field in record destructuring bind pattern
- duplicate binding in record destructuring bind pattern
- unknown record type in record destructuring bind
- unknown field in record destructuring bind
- record destructuring bind on non-matching record value
- unknown record type in record let-else
- unknown field in record let-else
- record let-else on non-matching record value
- record let-else without refutable quad literal field pattern
- record let-else literal pattern on non-quad field
- unknown record type in record copy-with
- record copy-with on non-record base
- empty record copy-with override set
- duplicate field in record copy-with
- unknown field in record copy-with
- record equality requested outside the stable field-equality subset
- invalid tuple arity
- tuple type mismatch
- tuple destructuring bind requires tuple value
- tuple destructuring bind arity mismatch
- let-else tuple destructuring bind requires tuple value
- let-else tuple destructuring bind arity mismatch
- let-else tuple literal pattern requires quad element
- nested tuple destructuring bind rejected in v0
- tuple destructuring assignment requires tuple value
- tuple destructuring assignment arity mismatch
- unknown tuple assignment target
- tuple destructuring assignment to const target
- malformed or empty `where` binding lists
- `break expr;` outside `loop` expression context
- loop-expression bodies that currently use unsupported `for-range`,
  iterable `for-each`, `guard`, or `return`
- UFCS method-call sugar written without explicit `(...)`
- return type mismatch
- invalid `guard` condition type
- invalid `if` condition type
- `if`-expression branch type mismatch
- invalid `match` guard condition type
- `match`-expression branch type mismatch
- invalid `match` scrutinee or missing `_` arm
- enum match pattern that does not match the scrutinee enum
- standard-form match pattern that does not match the scrutinee `Option(T)` /
  `Result(T, E)` family
- non-exhaustive `Option(T)` / `Result(T, E)` match without `_`
- enum match payload arity mismatch
- unsupported enum match payload item shape
- unsupported statement forms inside a value-producing block
- unsupported operator for a type family
- explicit `fx` coercion gap messages for still-narrow source cases
- runtime trap / diagnostics for plain `fx` division by zero or arithmetic
  overflow in the widened post-stable execution path
- explicit gap messages for unit-carrying `fx` arithmetic outside the first
  post-stable `fx` arithmetic slice
- explicit gap messages for text concatenation or text arithmetic beyond the
  current `M8.1` Wave 3 literal/equality/runtime surface
- explicit gap messages for collection-helper forms beyond the current `M8.3`
  first-wave iterable/carrier/index/equality surface
- explicit source/runtime failures for non-`i32` sequence indexes, negative
  sequence indexes, or out-of-bounds sequence indexes in the current `M8.3`
  first-wave execution path

Current honest limit:

- exact wording of type-check messages is not yet a fully frozen compatibility
  contract
- current `main` now admits executable text literal typing and same-family
  equality in the source type-check path
- current `main` now also admits verified runtime execution for the same narrow
  text literal/equality surface
- current `main` now also admits executable ordered sequence literals,
  same-family equality, `expr[index]`, and `for value in sequence` through the
  current `M8.3` first-wave carrier/iterable path
- current `main` now also admits plain same-family `i32` relational typing and
  lowering through the existing verified compare path
- current `main` still reports explicit gap messages for non-`i32` relational
  operators until later application-completeness waves land
- current `main` still reports explicit gap messages for text arithmetic beyond
  equality until later `M8.1` waves land
- generated validation failures are currently documented as deterministic plan
  categories, not yet as a separate runtime or CLI diagnostic family
- generated API contract failures are currently documented as deterministic
  build-error families prefixed as `generated API contract build error: ...`,
  not yet as a separate numeric-code taxonomy
- generated wire-contract failures are currently documented as deterministic
  build-error families prefixed as `generated wire contract build error: ...`,
  not yet as a separate numeric-code taxonomy
- users should treat the failure class as stable before treating the full text
  as stable

## Module And Linkage Diagnostics

The current module/import/export surface carries the most explicit stable source
diagnostic codes in the language contract.

Current public codes include:

- `E0238` cyclic imports
- `E0239` import read/parse/load failures
- `E0240` re-export policy violation
- `E0241` alias or binding collisions
- `E0242` public re-export collisions
- `E0243` symbol re-export cycles with deterministic chain traces
- `E0244` missing selected import symbol
- `E0245` duplicate select alias, wildcard/select conflict, or kind mismatch

Current guarantees:

- these diagnostics are rendered as source-level semantic errors
- line/column information is preserved where available
- repository tests exercise these failure families directly
- wildcard overlap is currently resolved by declaration-order fallback rather
  than by a dedicated ambiguity diagnostic

## Logos Semantic Diagnostics

The Logos path also emits semantic warnings and errors.

Current warning families include:

- `W0250` non-idiomatic law naming
- `W0251` large law
- `W0252` unused entity field
- `W0253` magic-number style warning

Current rule:

- warnings are part of the source contract, but they do not currently block
  execution in the same way as source errors

## Diagnostic Stability Rules

Current stability expectations:

- source diagnostics should remain deterministic for the same input and active
  profile
- introducing a new stable diagnostic code family should update this document
- changing the meaning of an existing module/import/export code should update
  this document in the same change series

Current honest limits:

- not every frontend type error has a stable numeric code yet
- exact prose wording is not fully frozen across all source layers

## Non-Goals

This document does not yet claim stable support for:

- localized diagnostics
- machine-readable JSON diagnostics as a frozen public API
- a single complete numeric-code taxonomy for every frontend/type error

## Contract Rule

Any public change to source-facing diagnostic families, stable source error
codes, or policy/type/module diagnostic boundaries should update this document
in the same change series.
