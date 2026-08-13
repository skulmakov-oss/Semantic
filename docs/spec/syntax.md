# Source Syntax Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current canonical source-level syntax for the public
Semantic language surface.

It describes the source language accepted by the Rust-like frontend path. It
does not redefine:

- SemCode binary structure
- verifier admission rules
- VM runtime behavior
- PROMETHEUS ABI semantics

## Current Source Surfaces

Semantic currently exposes two distinct source-oriented surfaces:

- a Rust-like executable surface used for functions, expressions, and control
  flow
- a Logos-oriented declarative surface used for `System`, `Entity`, and `Law`
  forms

This document covers the Rust-like executable surface first. The remaining
public source-surface contracts are specified separately in:

- `source_semantics.md`
- `diagnostics.md`
- `modules.md`
- `logos.md`
- `docs/LANGUAGE.md`

## Program Structure

The current Rust-like executable program is a sequence of top-level `record`,
`schema`, `enum`, and function declarations:

```sm
record Name {
    field: type,
    ...
}
```

```sm
schema Name {
    field: type,
    ...
}
```

```sm
schema Name version(2) {
    field: type,
    ...
}
```

```sm
config schema Name {
    field: type,
    ...
}
```

```sm
schema Name {
    Variant {
        field: type,
        ...
    },
    ...
}
```

```sm
record DecisionContext {
    camera: quad,
    quality: f64,
}
```

Function surface remains:

```sm
fn name(arg: type, ...) -> ret_type {
    ...
}
```

Trailing default-parameter sugar is also part of the current v0 surface:

```sm
fn name(arg: type, optional_arg: type = expr) -> ret_type {
    ...
}
```

First-wave declaration contracts are also part of the current surface:

```sm
fn name(arg: type, ...) -> ret_type requires(condition) {
    ...
}
```

```sm
fn name(arg: type, ...) -> ret_type ensures(condition) {
    ...
}
```

```sm
fn name(arg: type, ...) -> ret_type invariant(condition) {
    ...
}
```

Expression-bodied sugar is also part of the current v0 surface:

```sm
fn name(arg: type, ...) -> ret_type = expr;
```

Current rules:

- `record` introduces a nominal top-level record declaration
- `schema` introduces a compile-time-only top-level schema declaration
- `config schema`, `api schema`, and `wire schema` introduce the same
  compile-time-only schema declaration family with explicit role metadata
- record declarations must be non-empty
- schema declarations must be non-empty
- record field names must be unique within one declaration
- record-shaped schema field names must be unique within one declaration
- tagged-union schema variant names must be unique within one declaration
- tagged-union schema field names must be unique within one variant payload
- record field types currently use the ordinary source type grammar
- schema field and variant-payload types currently use the ordinary source type grammar
- `fn` introduces a function
- parameters are named and typed explicitly
- trailing parameters may attach a default initializer with `= expr`
- zero or more `requires(condition)` clauses may appear after the signature and
  before the function body
- zero or more `ensures(condition)` clauses may appear after any `requires`
  clauses and before the function body
- zero or more `invariant(condition)` clauses may appear after any
  `requires/ensures` clauses and before the function body
- the return type is optional; omitted return type means `unit`
- function bodies are block-delimited with `{ ... }`
- `fn ... = expr;` is accepted as shorthand for a single returned expression
- the public program entrypoint is `fn main()`

Current v0 record limits:

- `RecordName { field: expr, ... }` is the current stage-1 record construction form
- `record_value.field_name` is the current stage-1 read-only field access form
- `record_value with { field: expr, ... }` is the current stage-2 immutable record update form
- record literals and record updates may use field punning shorthand when the
  field name and the in-scope binding name are the same
- record literal fields must appear exactly once by name
- lowering preserves declaration-slot order rather than source-field order
- record types may now appear in executable local bindings, parameters, and returns
- record destructuring and record copy-with now participate in the stable source contract
- record punning now participates in the stable source contract only as field shorthand inside canonical nominal record forms
- record equality is allowed only when every field type already supports stable equality
- record values are not part of the PROMETHEUS host ABI surface
- anonymous brace-only record literals/patterns, mutation, methods, and inheritance are not part of this slice

Current v0 schema limits:

- record-shaped `schema Name { field: type, ... }` declarations are part of
  the current surface
- tagged-union `schema Name { Variant { ... }, Other { ... } }` declarations
  are part of the current surface
- schema declarations are compile-time-only and do not introduce executable
  value carriers
- schema role markers are currently explicit only as top-level prefixes:
  `config schema`, `api schema`, and `wire schema`
- schema declarations may now also attach optional version metadata directly
  after the schema name: `schema Name version(2) { ... }`
- schema names are not yet valid in executable local, parameter, return, or
  match type positions

## Statements

Current statement forms:

- `const name = expr;`
- `const name: type = expr;`
- `let name = expr;`
- `let name: type = expr;`
- `let (a, b) = expr;`
- `let (a, _): (type_a, type_b) = expr;`
- `let RecordName { field_name: local_name, other_field: _ } = expr;`
- `let RecordName { field_name: T, other_field: local_name } = expr else return;`
- `let next = current with { quality: 1.0 };`
- `let (a, T) = expr else return;`
- `let (a, T): (type_a, quad) = expr else return expr;`
- `let _ = expr;`
- `let _: type = expr;`
- `name += expr;`
- `name -= expr;`
- `name *= expr;`
- `name /= expr;`
- `name &&= expr;`
- `name ||= expr;`
- `(a, b) = expr;`
- `for name in 0..10 { ... }`
- `for name in 0..=10 { ... }`
- `for name in collection { ... }`
- `guard condition else return;`
- `guard condition else return expr;`
- `assert(condition);`
- `if condition { ... } else { ... }`
- `if condition { ... } else if condition { ... } else { ... }`
- `match quad_expr { T => { ... } ... _ => { ... } }`
- `match quad_expr { T if ready == true => { ... } ... _ => { ... } }`
- `match maybe_expr { Maybe::Some(value) => { ... } _ => { ... } }`
- `return;`
- `return expr;`
- expression statements: `expr;`

Current statement rules:

- semicolons terminate executable statements
- `const` is currently statement-level only
- `const` initializer syntax mirrors `let` but uses a narrower compile-time-safe expression subset
- `let _ = expr;` is the current discard-bind surface
- tuple destructuring bind is currently flat only and accepts only names or `_`
- record destructuring bind is currently statement-level only
- record destructuring bind uses `RecordName { field: target }` and now also allows field shorthand `RecordName { field }`
- record destructuring bind currently supports only named targets or `_`
- record `let-else` is currently statement-level only
- record `let-else` uses `RecordName { field: target } = expr else return ...;` and also allows shorthand bind items `RecordName { field }`
- record `let-else` currently allows refutable items only through explicit `quad` literals `N/F/T/S`
- plain record destructuring bind does not currently accept quad-literal field targets
- tuple `let-else` currently requires tuple destructuring target and `else return`
- `let-else` tuple items are currently flat only and accept only names, `_`,
  or `quad` literals `N/F/T/S`
- tuple destructuring assignment is currently flat only and accepts only names or `_`
- compound assignment is statement-level sugar only
- `for ... in range` currently accepts only `i32` range expressions
- `guard` currently supports only the `else return` form
- `assert(condition);` is a statement-level builtin contract form
- `requires(condition)` is currently a function-level contract clause only
- `ensures(condition)` is currently a function-level contract clause only
- `invariant(condition)` is currently a function-level contract clause only
- `if` conditions must be `bool`
- `match` currently accepts `quad`, nominal enum, `Option(T)`, `Result(T, E)`,
  `i32`, and `u32` scrutinees; every other scrutinee type is rejected
  deterministically (SSF-07)
- `quad` `match` requires an explicit default arm `_ => { ... }`
- enum `match` may omit `_ => { ... }` only when explicit unguarded variant
  coverage is exhaustive
- `Option` / `Result` `match` may omit `_ => { ... }` only when explicit
  unguarded `Some/None` or `Ok/Err` coverage is exhaustive
- `_` in `match` remains the current wildcard/default arm spelling
- enum match patterns currently require explicit `Enum::Variant`
- standard-form match patterns currently require explicit `Option::Some`,
  `Option::None`, `Result::Ok`, or `Result::Err`
- enum match payload patterns are currently flat only and accept only names or `_`
- unit-returning calls may be used as statements
- extended numeric literal spelling does not itself widen arithmetic support

## Expressions

Current expression forms:

- literals:
  - quad literals: `N`, `F`, `T`, `S`
  - bool literals: `true`, `false`
  - text literals:
    - double-quoted text `"hello"`
    - empty text `""`
  - integer literals:
    - decimal `123`
    - decimal with separators `1_000`
    - hexadecimal `0xff`
    - explicit typed forms `123i32`, `123u32`, `0xffu32`
  - floating and fixed-point literals:
    - decimal `1.25`
    - decimal with separators `1_000.25`
    - explicit `f64` forms `1.25f64`, `100f64`
    - explicit `fx` forms `1.25fx`, `100fx`
- variables
- function calls
- named-argument calls:
  - `open(path = main_path, mode = read_only)`
  - `value |> stage(limit = 10)`
- UFCS / method-call sugar:
  - `value.scale(10.0)`
  - `sensor.clamp(min = 0.0, max = 1.0)`
- pipeline chains:
  - `value |> stage()`
  - `value |> stage(arg)`
  - `value |> (x => expr)`
- where-clause suffix:
  - `sqrt(a + b) where a = x * x, b = y * y`
  - `total where total: f64 = 1.0`
- range literals:
  - `0..10`
  - `0..=10`
- short lambda immediate-call sugar:
  - `(x => expr)(arg)`
- tuple literals:
  - `(1, true)`
  - `(value, ready, 1.0)`
- ordered sequence literals:
  - `[1, 2, 3]`
  - `["alpha", "beta"]`
- record literals:
  - `DecisionContext { camera: T, quality: 0.75 }`
  - `DecisionContext { quality: 0.75, camera: T }`
- record field access:
  - `ctx.camera`
  - `ctx.quality`
- record copy-with:
  - `ctx with { quality: 1.0 }`
  - `ctx with { camera: F, quality: 0.25 }`
  - `ctx with { quality }`
- record punning shorthand:
  - `DecisionContext { camera, quality }`
  - `let DecisionContext { camera, quality: _ } = ctx;`
  - `let DecisionContext { camera: T, quality } = ctx else return;`
- tuple types:
  - `(i32, bool)`
  - `(f64, quad, bool)`
- text type:
  - `text`
- ordered sequence type:
  - `Sequence(i32)`
- first-wave units-of-measure annotations:
  - `f64[m]`
  - `u32[ms]`
  - `Option(f64[m])`
  - `(f64[m], u32[ms])`
- block expressions with a trailing tail value:
  - `{ let x = 1; x }`
- `if` expressions with explicit `else` blocks:
  - `if ready { 1.0 } else { 0.0 }`
- `match` expressions with value-producing arms:
  - `match state { T => { 1.0 } _ => { 0.0 } }`
  - `match state { T if ready == true => { 1.0 } _ => { 0.0 } }`
- `when` expressions for quad-oriented rule selection:
  - `when ready { 1.0 } else { 0.0 }`
- `loop` expressions with explicit `break value`:
  - `loop { break 1.0; }`
  - `loop { if ready { break 1.0; } else { break 0.0; } }`
- parenthesized expressions
- unary operators:
  - `!`
  - unary `+`
  - unary `-`
- binary operators:
  - `*`, `/`
  - `+`, `-`
  - `<`, `<=`, `>`, `>=`
  - `==`, `!=`
  - `is`
  - `&&`, `||`
  - `->`

Current v0 numeric-literal limits:

- unsuffixed integer literals currently mean `i32`
- unsuffixed decimal literals with `.` currently mean `f64`
- explicit `fx` literals are decimal-only and do not require `f64` surface policy
- hexadecimal literals currently target only integer carriers
- exponent notation and binary/octal literal families are not yet part of the
  stable surface
- typed literal spelling does not imply new integer arithmetic beyond the
  already documented operator surface
- tuple literal arity must be at least 2 in the current contract

## Field Punning And Tail Expressions

The current source contract keeps a narrow, deterministic sugar layer for
readability. These forms are syntax-only conveniences and lower into the same
canonical core shapes as their explicit counterparts.

### Field Punning

Field punning is accepted in record literals and record updates when a field
name is spelled exactly the same as an in-scope binding name.

Current forms:

- `RecordName { field }`
- `record_value with { field }`

Canonical lowering:

- `RecordName { field }` lowers to `RecordName { field: field }`
- `record_value with { field }` lowers to `record_value with { field: field }`

Current constraints:

- the field name must resolve to a local binding in scope
- the shorthand does not introduce a new binding
- the shorthand does not rename or infer a different field name
- the shorthand is available only where the surrounding record form is already valid

### Tail Expression Return

Value-producing blocks may end with a final expression that becomes the block
result without an explicit `return`.

Current form:

- `{ let d = x - y; abs(d) }`

Canonical lowering:

- the tail expression lowers as if the block contained `return abs(d);`

Current constraints:

- the shorthand applies only when the block is already in value position
- intermediate statements still require semicolons
- a trailing expression without `;` is the only implicit-return form
- statement-only blocks do not gain an implicit return rule
- the shorthand does not change control-flow exhaustiveness rules for `if`,
  `match`, or `loop`

### Related Lowering Rule

Both sugar forms preserve the current canonical lowering discipline:

- the parser accepts the surface form only when the underlying core shape is
  already well-typed and unambiguous
- lowering expands the sugar before SemCode generation
- diagnostics should point back to the canonical explicit form when the sugar
  is malformed

Current text-literal limits:

- the current executable text surface is only double-quoted same-line text
- interpolation and multi-line text blocks are not part of the current surface
- text concatenation is not yet part of the current source contract

Current active collections checkpoint on `main`:

- one ordered sequence family is now admitted in declared type positions as
  `Sequence(type)`
- bracketed ordered sequence literals are now admitted in the Rust-like source
  path
- postfix indexing `expr[index]` is now admitted when `expr` is an admitted
  ordered sequence and `index` is `i32`
- same-family equality is now admitted for ordered sequence values when the
  item type already supports stable equality
- `for name in collection { ... }` is now part of the current syntax contract
  through the narrow `Iterable` owner-layer loop surface described below
- built-in `Sequence(type)` values now participate in that admitted iterable
  loop path on current `main`
- `len`, `is_empty`, maps, sets, and generic collection abstractions are not
  part of the current `M8.3` first-wave syntax contract

Current v0 range-literal limits:

- range literals currently accept only `i32` bounds
- `start..end` is half-open and `start..=end` is closed
- range literals currently lower through an internal executable interval
  descriptor rather than a dedicated runtime range opcode
- range equality is not yet part of the stable source contract
- range literals are not yet part of the stable tuple/user-data surface
- `for ... in range` currently exposes only the narrow `i32` interval surface
- `for name in collection { ... }` is source-admitted on current `main` as the
  `Iterable` owner-layer loop surface
- built-in `Sequence(type)` collections now execute through the current
  first-wave iterable loop path on `main`
- direct record `Iterable` impl dispatch now executes on current `main` when the
  impl exposes `fn next(self: Self, index: i32) -> Option(Item)`
- `Self` is admitted only in trait method signatures and impl method type
  positions on current `main`
- `Self` outside trait/impl method type positions is not part of the stable
  syntax contract
- ADT/schema iterable dispatch and indirect iterable projection are not part of
  the current syntax contract
- descending/custom-step/general iterable range forms are not yet part of the
  stable syntax contract

Current v0 tuple limits:

- tuples are currently aggregate value carriers only
- tuple literals and tuple types are supported
- tuple destructuring bind is currently statement-level only
- tuple destructuring bind currently supports only flat name-or-`_` item lists
- tuple `let-else` currently supports only flat name/`_`/quad-literal item
  lists
- tuple destructuring bind currently requires arity at least 2
- tuple destructuring assignment is currently statement-level only
- tuple destructuring assignment currently supports only flat name-or-`_` item lists
- tuple equality follows ordinary `==` / `!=` when both operands have the same
  tuple type
- tuple field access and tuple pattern matching beyond flat destructuring bind
  are not yet part of the stable surface

Current first-wave units-of-measure syntax rules:

- declared type positions may attach `[unit_symbol]` only to `i32`, `u32`,
  `f64`, or `fx`
- unit annotations may appear inside tuples, record fields, `Option(T)`, and
  `Result(T, E)` payload positions when the annotated inner type is still one
  of those core numeric families
- this is a narrow source-level contract only; the execution carrier remains
  the existing numeric family after semantic validation
- unit annotations on non-numeric type families are rejected in v0
- compound unit syntax such as `m/s` or exponent-style unit algebra is not part
  of the stable surface
- unannotated numeric literal spelling remains unchanged; units are acquired
  only through typed positions

Current first-wave `Option` / `Result` standard-form rules:

- declared type positions now accept `Option(type)` and `Result(ok_type, err_type)`
- canonical constructors are:
  - `Option::Some(value)`
  - `Option::None`
  - `Result::Ok(value)`
  - `Result::Err(error)`
- this is a narrow standard-forms surface, not a general generic type system
- angle-bracket type application and user-defined parameterized declarations are
  not part of this slice
- `Option::Some(value)` may infer `Option(T)` from its payload type
- `Option::None` currently requires contextual `Option(T)` type
- `Result::Ok(...)` and `Result::Err(...)` currently require contextual
  `Result(T, E)` type
- canonical `Option` / `Result` constructors reuse the existing `MakeAdt`
  lowering family
- `match` now accepts explicit standard-form patterns:
  - `Option::Some(name_or_)`
  - `Option::None`
  - `Result::Ok(name_or_)`
  - `Result::Err(name_or_)`
- exhaustive `Option` / `Result` matches may omit `_ => { ... }` when all
  unguarded variants are covered
- this still reuses the existing canonical ADT-style pattern machinery rather
  than widening the general pattern system

Current precedence, from tighter to looser:

1. primary expressions and calls
2. unary `!`, unary `+`, unary `-`
3. `*`, `/`
4. `+`, `-`
5. `<`, `<=`, `>`, `>=`
6. `==`, `!=`
7. `&&`
8. `||`
9. `->`
10. `|>`

Current short-lambda rules:

- short lambda syntax is currently single-parameter only: `x => expr`
- the published stable `v1.1.1` line keeps short lambdas as non-first-class
  call-site sugar only
- current `main` now admits standalone first-class closure literals in
  contextual positions such as `let f: Closure(f64 -> f64) = (x => x + 1.0);`
- the stable v0 surface accepts short lambdas only as:
  - immediate call sugar: `(x => expr)(arg)`
  - pipeline stage sugar: `value |> (x => expr)`
- immediate-call and pipeline sugar remain capture-free in the stable line
- current `main` admits immutable capture inventory for standalone first-class
  closure literals
- `Closure(T -> U)` is the current narrow declared type spelling for the
  admitted first-wave closure family on `main`
- typed lambda parameters and multi-argument lambda forms are not yet part of
  the stable source contract

Current active closures checkpoint on `main`:

- `docs/roadmap/language_maturity/first_class_closures_full_scope.md`
- published `v1.1.1` still keeps short lambdas as non-first-class call-site
  sugar only

Current named-argument rules:

- ordinary user-defined calls may use named arguments
- positional arguments are allowed only as a leading prefix before any named
  argument
- required non-default parameters must still be supplied exactly once in v0
- named arguments reorder to the declared parameter order before ordinary
  type-checking and lowering
- named arguments are not yet part of the builtin-call surface

Current first-wave `requires` rules:

- `requires(condition)` currently attaches only to ordinary user-defined
  function declarations
- `requires(condition)` may appear on block-bodied and expression-bodied
  functions
- each `condition` must be `bool`
- the current stable subset allows only parameter references, tuple literals,
  record field reads, and pure unary/binary operator expressions
- call expressions, block/control-flow expressions, range literals, record
  construction, and record copy-with are not part of this slice

Current first-wave `ensures` rules:

- `ensures(condition)` currently attaches only to ordinary user-defined
  function declarations
- `ensures(condition)` may appear on block-bodied and expression-bodied
  functions
- each `condition` must be `bool`
- the current stable subset allows parameter references, optional synthetic
  `result` binding, tuple literals, record field reads, and pure unary/binary
  operator expressions
- the synthetic `result` binding is reserved while `ensures` clauses are
  present and is available only for non-unit returns
- call expressions, block/control-flow expressions, range literals, record
  construction, and record copy-with are not part of this slice

Current UFCS / method-call rules:

- `receiver.name(args...)` is accepted as postfix call sugar
- UFCS currently desugars to ordinary call order: `name(receiver, args...)`
- UFCS may chain because it remains ordinary expression/call surface after
  desugaring
- `.name` without `(...)` is parsed as field access and then typechecked only
  against nominal record values
- UFCS does not introduce object members or method declarations

Current where-clause rules:

- `expr where name = value, ...` is accepted as expression-suffix sugar
- each `where` binding currently follows ordinary `let` spelling with optional
  whole-binding type annotation
- bindings appear in source order and are visible to later `where` bindings and
  to the tail expression
- `where` currently desugars through the existing block-expression path

Current loop-expression rules:

- `loop { ... }` is accepted only as an expression form in this slice
- loop-expression bodies currently exit only through `break expr;`
- bare `break;` is not part of the stable v0 contract
- `break expr;` is currently valid only inside `loop` expression bodies

Current honest limit:

- `where` is currently expression-suffix sugar only
- `where` bindings currently use ordinary local names only; tuple/record
  destructuring is not yet part of the stable `where` contract
- `loop` is admitted both as a value-producing expression form and as a
  statement form
- `break expr;` is currently valid only inside loop-expression bodies
- bare `break;` and `continue;` are currently valid only inside admitted
  `while` and statement-loop bodies
- `while condition { ... }` is admitted only as a statement form; value
  `while` and labeled loops are not yet part of the stable contract

Current default-parameter rules:

- only trailing parameters may declare defaults in v0
- omitted arguments may currently be filled only from those declared trailing
  defaults
- default initializers are part of ordinary user-defined function declarations
- builtin calls do not expose default-parameter surface

## Quad-Specific Surface Rules

Current `quad` model values are:

- `N` = unknown
- `F` = false
- `T` = true
- `S` = conflict

Current source restrictions:

- `if quad_expr` is forbidden; users must write an explicit comparison
- `quad` is one of the admitted `match` scrutinee types (see the `match`
  bullet under "## Statements" above for the full allowlist)
- `when` is an expression-level surface form for explicit boolean selection
- `is` is a readable alias for explicit quad comparison, not a truthiness
  coercion
- quad implication uses `->`

## Builtin Calls

Builtin calls currently share call syntax with ordinary functions:

```sm
sqrt(9.0)
pow(2.0, 3.0)
abs(-1.0)
assert(ready == true);
```

Current builtins are resolved as part of the public source surface and must not
require a separate foreign-call syntax. `assert` shares ordinary call-like
syntax but is statement-only in the current contract.

## Imports And Module Surface

The current repository supports source-level imports and re-exports. That
surface is part of the language contract and is specified in `modules.md`.

## Current Exclusions

The current source contract does not yet claim stable support for:

- relational operators outside the current plain same-family `i32` first-wave slice
- user-defined aggregate value operations beyond top-level nominal record declarations
- collections as first-class language forms
- generics or trait-like abstraction
- exceptions or Python-style dynamic execution

Current active collections checkpoint on `main`:

- `docs/roadmap/language_maturity/collections_surface_full_scope.md`
- concurrency-oriented source constructs

## Contract Rule

Any public change to source syntax, source statement forms, or operator meaning
should update this document in the same change series.
