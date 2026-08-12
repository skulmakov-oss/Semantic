# Source Semantics Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current source-level execution meaning of Semantic
programs before lowering into IR, SemCode verification, or VM execution.

It complements `syntax.md` and `types.md` by specifying how the accepted
source forms are interpreted today.

It does not redefine:

- SemCode header or opcode rules
- verifier admission semantics
- VM runtime traps
- PROMETHEUS ABI effects

## Program Meaning

The current Rust-like executable surface is a deterministic function program.

Current rules:

- top-level source items currently include executable `Import` directives,
  nominal `record`, compile-time-only `schema`, and executable function
  declarations
- `record` declarations contribute nominal type identity but are not themselves executable entrypoints
- `schema` declarations contribute compile-time contract metadata only and are
  not executable entrypoints or value families
- top-level executable `Import` directives now admit one narrow module-entry
  slice on current `main`:
  - direct local-path bare imports such as `Import "helper.sm"`
  - direct local-path selected imports such as
    `Import "helper.sm" { Foo, Bar as Baz }` when the imported helper module
    stays within the current function-only helper slice
  - deterministic helper-module bundling before executable semantic checking
  - selected executable helper imports synthesize only the requested public
    bindings plus the required local helper-function call closure before
    executable semantic checking
- the current executable path still does **not** admit:
  - top-level alias imports
  - wildcard imports
  - public re-exports
  - namespace-qualified executable access
- package-qualified bare imports are admitted only through an enclosing
  `Semantic.package` dependency declaration and remain subject to SSF-06 root,
  identity, and provenance checks
- execution begins at `fn main()`
- `main` must currently have signature `fn main()`
- there is no dynamic entrypoint discovery or module-level executable code
- `fn name(...) -> ret = expr;` is semantically equivalent to a body containing
  only `return expr;`
- function-level `requires(condition)` clauses execute at callee entry after
  parameter binding and before ordinary body statements
- multiple `requires(condition)` clauses evaluate in source order
- function-level `ensures(condition)` clauses execute immediately before each
  function return path completes
- multiple `ensures(condition)` clauses evaluate in source order on the exit
  path that produced the return value
- function-level `invariant(condition)` clauses are first-wave entry/exit
  contract checks rather than a separate proof system
- invariant clauses that do not reference `result` execute at both function
  entry and function exit
- invariant clauses that reference the synthetic `result` binding execute only
  on function exit and only for non-unit returns
- multiple `invariant(condition)` clauses evaluate in source order at each
  applicable check point

Current v0 record declaration semantics:

- `record Name { ... }` introduces one nominal source type
- two records with the same field shapes remain distinct because identity is by record name, not by structural shape
- record declarations must be non-empty and may not repeat field names
- record declarations may refer to other declared record names in field types
- recursive record field graphs are not yet part of the stable contract
- stage-1 record literals use `RecordName { field: expr, ... }`
- record literal fields are validated by declared field names and types
- record literal evaluation is deterministic left-to-right in source field order
- the canonical runtime carrier stores slots in declaration order, not source order
- stage-1 record values may flow through executable locals, parameters, and returns
- stage-1 field access uses `record_value.field_name` and resolves against the canonical record declaration
- stage-1 field access lowers through deterministic declaration-slot reads
- stage-2 immutable record update uses `record_value with { field: expr, ... }`
- copy-with preserves the nominal record identity of its base value
- record field shorthand is sugar only: `RecordName { field }` means `RecordName { field: field }`, `value with { field }` means `value with { field: field }`, and `let RecordName { field } = value;` means `let RecordName { field: field } = value;`
- explicit record destructuring bind uses `let RecordName { field: target, ... } = value;`
- explicit record destructuring bind currently projects only the named subset of declaration fields
- `_` targets in explicit record destructuring bind discard the projected field value without creating a source binding
- explicit record `let-else` uses `let RecordName { field: target, ... } = value else return ...;`
- record `let-else` currently treats only explicit `quad` literal field targets as refutable checks
- record equality is allowed only when every field type already supports stable equality

Current v0 schema declaration semantics:

- `schema Name { ... }` introduces one compile-time-only schema declaration
- schema identity is nominal by schema name
- schema declarations currently support:
  - record-shaped forms `schema Name { field: type, ... }`
  - tagged-union forms `schema Name { Variant { field: type, ... }, ... }`
- schema declarations may now also carry one explicit role marker:
  - `config schema`
  - `api schema`
  - `wire schema`
- schema declarations may now also carry optional `version(<u32>)` metadata
  immediately after the schema name
- record-shaped schema declarations must be non-empty and may not repeat field
  names
- tagged-union schema declarations must declare at least one variant, may not
  repeat variant names, and may not repeat field names inside one variant
- schema field and variant-payload types reuse the current declared-type grammar
  and resolve against the ordinary nominal/executable type tables
- schema declarations currently live only in the canonical schema table owned by
  the frontend/typecheck path
- schema version metadata currently lives only in that canonical schema table as
  compile-time/tooling ownership data
- record-shaped schemas with explicit version metadata may now also participate
  in deterministic tooling-owned compatibility classification across two schema
  revisions
- tagged-union schemas with explicit version metadata may now also participate
  in deterministic tooling-owned compatibility classification across two schema
  revisions
- the current first-wave compatibility classes are `Equivalent`, `Additive`,
  and `Breaking`
- canonical schema evolution may now also derive tooling-owned migration
  metadata artifacts and stable formatted review output from those same
  compatibility reports
- canonical schema declarations may now also derive deterministic compile-time
  validation plans owned by the same frontend/typecheck path
- record-shaped schemas currently derive first-wave validation checks in
  declaration order:
  - required-field checks
  - field-type compatibility checks
- tagged-union schemas now also derive first-wave branch checks in variant
  declaration order:
  - allowed-branch checks
  - per-branch required-field checks
  - per-branch field-type compatibility checks
- `api schema` and `wire schema` declarations may now also derive one
  deterministic generated API contract artifact family owned by `smc-cli`
- generated API contract artifacts preserve schema, variant, and field
  declaration order for reviewability
- generated API contract artifacts currently carry explicit format-version and
  generator metadata for reproducibility
- `wire schema` declarations may now also derive one deterministic generated
  wire-contract artifact family owned by `smc-cli`
- generated wire-contract artifacts currently contain:
  - tagged wire unions derived from canonical tagged-union `wire schema`
    declarations
  - wire patch types derived from canonical record-shaped `wire schema`
    declarations
- generated wire-contract artifacts preserve variant, payload-field, and
  patch-field declaration order for reviewability
- generated wire-contract artifacts currently carry explicit format-version and
  generator metadata for reproducibility
- `config schema` declarations do not participate in generated API artifact
  derivation in the first-wave contract
- schema role markers currently contribute compile-time declaration metadata
  only; they do not imply loading, generation, transport, or runtime behavior
- schema declarations do not currently introduce executable types, runtime
  carriers, or host ABI shapes

Current first-wave function-contract semantics:

- only declaration-level `requires`, `ensures`, and narrow `invariant` are
  part of the current stable contract surface
- `requires` checks the narrow contract subset in a parameter-only environment
- `ensures` checks the same narrow subset on the return path
- non-unit functions may additionally use the synthetic `result` binding inside
  `ensures` and `invariant`
- `invariant` checks the same narrow subset at function entry and exit
- `invariant` clauses that reference `result` are exit-only checks
- `ensures` currently lowers to explicit core assertions before `ret`
- `invariant` currently lowers to the same explicit core assertion path used by
  the other contract clauses
- this slice is not yet a general proof/effect system and does not imply loop,
  block, or mutation-point invariant semantics

## Deterministic Evaluation Order

Current source evaluation order is deterministic.

Current rules:

- binary expressions evaluate the left operand before the right operand
- call arguments are evaluated left-to-right
- control-flow conditions are evaluated before branch selection
- `match` scrutinee is evaluated once before arm dispatch
- pipeline stages evaluate left-to-right and pass the previous stage value as
  the first argument of the next call
- short lambda call-site sugar evaluates its argument before the lambda body
  binding becomes visible

The source contract does not currently claim short-circuit laziness beyond the
observable deterministic behavior of the current lowering path.

## Names And Call Resolution

Function-call resolution is lexical and deterministic.

Current rules:

- ordinary user-defined functions are resolved from the program function table
- builtin math calls are resolved only when no user-defined function of the
  same name exists
- there is no overload resolution
- there is no dynamic dispatch
- named arguments reorder only after the target function resolves successfully

Current builtin names in the Rust-like surface are:

- `sin`
- `cos`
- `tan`
- `sqrt`
- `abs`
- `pow`
- `assert`

Current named-argument call semantics:

- named arguments are currently supported only for ordinary user-defined
  functions
- positional arguments may appear only before the first named argument
- after resolution, named arguments reorder to the declared parameter order
  before ordinary argument type-checking and lowering
- each required non-default parameter must receive exactly one argument in the
  current contract

Current default-parameter limits:

- builtin calls do not yet accept named arguments
- named arguments do not imply overload resolution or keyword-only parameters

Current default-parameter semantics:

- default parameters are currently supported only for ordinary user-defined
  functions
- only trailing parameters may declare defaults in the stable v0 surface
- when an argument is omitted for a trailing defaulted parameter, the declared
  default initializer is substituted before ordinary argument type-checking and
  lowering
- default initializers must remain within the current const-safe expression
  subset and may not depend on function parameters or local runtime bindings

Current first-wave function-contract semantics:

- `requires(condition)`, `ensures(condition)`, and narrow
  `invariant(condition)` are the current declaration-level function contract
  clauses in the stable source surface
- each `requires` condition is type-checked in a parameter-only environment
- each `ensures` condition is type-checked in a parameter-plus-optional-result
  environment
- each `invariant` condition is type-checked in the same narrow environment,
  with `result` allowed only for non-unit returns
- the current stable subset allows only parameter references, tuple literals,
  record field reads, optional `result`, and pure unary/binary operator
  expressions
- function calls, record construction, record copy-with, range literals,
  blocks, and control-flow expressions are not yet part of the stable
  contract-expression subset
- lowering translates each `requires` clause to an explicit core assertion at
  function entry
- lowering translates each `ensures` clause to an explicit core assertion on
  each return path
- lowering translates each `invariant` clause to explicit core assertions at
  function entry and/or function exit depending on whether the clause
  references `result`

Current v0 limits:

- builtin calls do not yet expose default parameters
- default parameters do not imply optional positional holes, keyword-only
  parameters, or overload resolution

## Numeric Literal Meaning

Current numeric-literal rules:

- unsuffixed decimal integer literals currently evaluate as `i32`
- unsuffixed decimal literals containing `.` currently evaluate as `f64`
- explicit `i32`, `u32`, `f64`, and `fx` suffixes fix the source-level literal type
- digit separators `_` are ignored for numeric-literal value decoding
- hexadecimal literals currently decode only for integer carriers
- explicit `fx` literals lower directly to the fixed-point value carrier and do
  not rely on `f64` surface policy

Current honest limit:

- extended numeric literal spelling does not currently widen integer arithmetic
  beyond the already documented operator surface
- exponent notation and binary/octal literal families are not yet part of the
  stable contract

## Units Of Measure

Current first-wave units-of-measure semantics:

- unit annotations refine the source type of `i32`, `u32`, `f64`, or `fx`
  without changing the underlying execution carrier
- unsuffixed numeric literals remain ordinary numeric literals and become
  unit-carrying only through typed positions such as annotated locals,
  parameters, returns, tuple elements, record fields, and contextual
  `Option(T)` / `Result(T, E)` payloads
- assignment, call, return, and pattern-binding transport require exact base
  family and exact unit-symbol equality
- unit annotations may travel through tuples, records, `Option(T)`, and
  `Result(T, E)` when those positions contain supported numeric families
- lowering erases units after semantic validation and reuses the existing
  numeric lowering path
- `fx` should be read as a stable value-transport and equality family inside the
  current line; binary arithmetic on `fx` remains outside the current contract
- unary `+` / unary `-` for `fx` remain limited to literal formation, not
  general `fx` expression rewriting; on a measured `fx` value this surfaces
  as the same explicit narrow-slice-gap rejection as measured `fx` binary
  `+`/`-`
- binary and unary `+`/`-` are admitted only for measured `f64`; measured
  `i32`/`u32` binary and unary `+`/`-` are rejected as unsupported operators
  (see "Operator Meaning" below for the full per-family breakdown and test
  evidence)

Current v0 limits:

- unit annotations are compile-time-only source contracts, not VM value tags
- implicit conversion between unit symbols is not part of the stable contract
- compound unit algebra, conversion tables, and inference from untyped numeric
  literals are not part of the first-wave surface
- `*`, `/`, and `%` on unit-carrying values are rejected in the first-wave
  contract

## Range Literal

Current range-literal semantics:

- `start..end` denotes a half-open ascending `i32` interval descriptor
- `start..=end` denotes a closed ascending `i32` interval descriptor
- both bounds are evaluated exactly once, left to right
- the current lowering path reuses an internal tuple-shaped carrier rather than
  introducing a dedicated runtime range family

Current v0 limit:

- range literals currently require `i32` bounds
- range literals are not yet part of the stable tuple/user-data surface
- range equality is not yet part of the stable source contract
- the source contract does not yet treat ranges as a general iterable family

## For-Range

Current `for ... in range` semantics:

- `for i in start..end { ... }` evaluates the range expression once before the
  first iteration
- the loop variable is rebound each iteration from the current `i32` counter
- half-open ranges iterate while `current < end`
- closed ranges iterate while `current <= end`
- when the interval is already empty, the body does not execute

Current v0 limit:

- `for ... in range` currently accepts only `RangeI32` values
- general `for x in collection` source syntax is now admitted on current `main`
  as an owner-layer desugaring toward the named `Iterable` contract
- built-in `Sequence(type)` values now execute through the current first-wave
  iterable loop path on `main`
- direct record `Iterable` impls now execute through the same loop driver when
  they expose `fn next(self: Self, index: i32) -> Option(Item)`
- trait-side `Self` now denotes the impl-anchored receiver contract only inside
  trait method signatures and impl method type positions
- `Self` does not widen the general executable type surface beyond that narrow
  trait/impl contract
- ADT/schema iterable dispatch and indirect iterable projection remain outside
  the current stable contract
- descending ranges, custom step values, `continue`, and a general iterable
  subsystem are not yet part of the stable contract
- `for ... in range` does not widen the public operator surface beyond the
  current plain same-family `i32` relational slice

## Scope And Binding Rules

The executable Rust-like surface uses lexical block scoping.

Current rules:

- function parameters are bound in function scope before body execution
- `const` introduces an immutable source-visible local binding
- `let` introduces a source-visible local binding
- `let (a, b) = expr;` evaluates the right-hand side once and then binds named
  items left-to-right from the produced tuple value
- `let (a, T) = expr else return ...;` evaluates the right-hand side once,
  projects flat tuple items, and returns immediately when any refutable
  quad-literal item does not match
- `let _ = expr;` evaluates the right-hand side but introduces no source-visible binding
- `if` branches and `match` arms are checked in branch-local scopes
- branch-local bindings do not escape to sibling branches

Current honest limit:

- the repository does not yet freeze a richer user-facing policy around
  same-scope rebinding or style-level shadowing; users should not depend on
  subtle local-name reuse behavior as a stable contract

## Statements

Current statement meaning:

- `const` evaluates a compile-time-safe initializer expression before binding the name
- const bindings are immutable in the current source contract
- `let` evaluates the right-hand side before binding the name
- tuple destructuring bind evaluates the right-hand side once before projecting
  tuple items into named bindings
- record destructuring bind evaluates the right-hand side once before projecting
  the requested record fields into named bindings
- record copy-with evaluates the base value once, evaluates override expressions
  left-to-right, then rebuilds the canonical slot carrier in declaration order
- record `let-else` evaluates the right-hand side once, checks refutable
  `quad` field literals before introducing named bindings, and follows the
  explicit `else return` path on failure
- tuple destructuring assignment evaluates the right-hand side once before
  projecting tuple items into existing assignment targets
- tuple `let-else` introduces named bindings only on the success path; failure
  follows the explicit `else return` path
- discard bind evaluates the right-hand side and then drops the produced value
- `name op= expr;` evaluates as read-modify-write over the existing binding
- the current v0 compound forms are `+=`, `-=`, `*=`, `/=`, `&&=`, and `||=`
- `for ... in range` evaluates the interval descriptor once and then advances a
  hidden `i32` loop carrier by one per iteration
- `guard condition else return ...;` continues when the condition is `true`
- when the guard condition is `false`, the `else return` path terminates the
  current function immediately
- `assert(condition);` continues when `condition` is `true`
- `assert(condition);` terminates through the core fail-fast trap path when
  `condition` is `false`
- expression statements evaluate for effect and then discard any produced value
- `let name: T = expr;` introduces a local binding
- `let name = expr;` introduces a local binding whose type is inferred
  conservatively from the initializer and any surrounding expected type
- quad-heavy code may rely on local inference when the initializer already fixes
  the intended quad meaning, but explicit `: quad` annotations remain the
  clearest spelling when the source needs to communicate meaning directly
- `let mut name: T = expr;` is admitted as an explicit writable-local spelling in
  the current Rust-like path
- plain reassignment `name = expr;` is admitted for local bindings
- compound assignment `name += expr;`, `name -= expr;`, `name *= expr;`,
  `name /= expr;`, `name &&= expr;`, and `name ||= expr;` desugar through the
  same assignment path
- `return expr;` terminates the current function with that value
- `return;` terminates a `unit`-returning function

Current non-goal:

- the source contract does not claim deferred execution, generators, or
  coroutine-style statement behavior
- `guard` does not yet support arbitrary `else { ... }` recovery blocks

Current generated wire-contract limit:

- generated wire-contract artifacts are review/build outputs only
- record patch types do not imply a runtime patch application engine
- tagged wire unions do not imply transport/runtime integration

Current v0 const limit:

- `const` is statement-level only in the current source contract
- const initializers currently support only pure literal/const expression forms
- ordinary function calls, control-flow expressions, and references to
  non-const locals are not yet part of the stable const initializer subset

## Block Expressions

Current block-expression semantics:

- `{ ... tail }` evaluates its body in a fresh lexical block scope
- body statements run in source order before the tail expression
- the final unterminated tail expression becomes the value of the block
- block-local bindings do not escape the block expression

Current v0 limit:

- block-expression bodies currently accept only named `let` bindings,
  tuple destructuring binds, `const` bindings, discard binds, and expression
  statements before the tail value
- tuple `let-else` is not yet part of the stable value-producing block-body
  contract
- record `let-else` is not yet part of the stable value-producing block-body
  contract
- discard bind is accepted in value-producing block bodies, but richer pattern
  destructuring is not yet part of the stable block-expression contract
- `return` is not yet supported inside value-producing block bodies as a
  stable source contract

## Where-Clause

Current `where` semantics:

- `expr where a = x, b = y` is expression-suffix sugar over an ordinary
  value-producing block
- `where` bindings execute in source order before the tail expression
- later `where` bindings may reference earlier `where` bindings
- the tail expression sees all `where` bindings introduced by the clause

Current v0 limit:

- `where` is currently expression-suffix sugar only
- `where` bindings currently reuse ordinary local-bind semantics; richer
  destructuring and control-flow forms are not yet part of the stable contract

## UFCS / Method-Call Sugar

Current UFCS semantics:

- `receiver.name(args...)` is postfix sugar over the ordinary call form
  `name(receiver, args...)`
- after desugaring, ordinary parameter ordering, named-argument reordering, and
  lowering rules apply unchanged
- UFCS chaining remains ordinary nested-call structure after desugaring

Current v0 limit:

- UFCS currently requires explicit call parentheses
- `.name` without call parentheses is field access, not UFCS
- UFCS does not introduce method declarations or object-oriented dispatch

## Loop Expression

Current `loop` expression semantics:

- `loop { ... }` produces a value through explicit `break expr;`
- `break` values inside the same loop must agree on one result type
- lowering uses the existing label/jump/store/load path; no separate runtime
  carrier is introduced for this slice

Current v0 limit:

- only expression-form `loop` is part of the contract
- only `break expr;` is supported; bare `break;` is not
- loop-expression bodies currently do not allow `let-else`, `guard`, or
  `return`
- `continue`, statement-loop, and richer control interaction are deferred

## While Statement

Current `while` statement semantics:

- `while condition { ... }` is admitted as a statement form
- the condition must typecheck as `bool`
- lowering reuses the existing label/jump path; no new runtime carrier is
  introduced for this slice

Current v0 limit:

- `while` is statement-only and does not produce a value
- labeled loops remain deferred

## Statement Loop And Control Exits

Current statement-loop semantics:

- `loop { ... }` is admitted as a statement form
- bare `break;` exits the innermost admitted `while` or statement `loop`
- `continue;` resumes the next iteration of the innermost admitted `while` or
  statement `loop`
- lowering reuses the existing label/jump path; no new runtime carrier is
  introduced for this slice

Current v0 limit:

- statement `loop` does not produce a value
- `break expr;` remains restricted to loop-expression bodies
- labeled loops remain deferred

## Tuple Destructuring Bind

Current tuple-destructuring semantics:

- `let (a, b) = expr;` requires the right-hand side to produce a tuple value
- the tuple arity must match the binding item count exactly
- `_` items discard the corresponding tuple element without introducing a
  source-visible binding
- an optional whole-binding annotation, such as
  `let (a, b): (i32, bool) = expr;`, constrains the full tuple value before
  item bindings are introduced
- `(a, b) = expr;` assigns projected tuple items into already existing mutable
  bindings
- `_` items in tuple destructuring assignment discard the corresponding tuple
  element without requiring a target binding

Current v0 limit:

- tuple destructuring bind is currently statement-level only
- tuple `let-else` is currently statement-level only
- tuple destructuring assignment is currently statement-level only
- only flat name-or-`_` item lists are supported for plain tuple destructuring
- tuple `let-else` currently supports only flat name/`_`/quad-literal item
  lists
- tuple destructuring assignment does not introduce new bindings; every named
  item must already resolve to an existing non-const local
- nested tuple patterns, tuple field access, and general tuple pattern matching
  are not yet part of the stable source contract
- plain `let name = expr else ...` is not yet part of the stable source
  contract

## Control Flow

### If

Current `if` semantics:

- `if` requires a `bool` condition
- the `then` branch runs when the condition is `true`
- otherwise the `else` branch runs
- `else if` is treated as nested `if` inside the `else` branch

Current `if` expression semantics:

- `if condition { ... } else { ... }` may appear in value position
- `if condition { ... } else if condition { ... } else { ... }` is also
  admitted and lowers through the same nested `if` structure
- both branches are evaluated through value-producing block semantics
- both branches must produce the same type
- `else` is required for value-producing `if`

`quad` is intentionally not treated as an implicit condition type. Users must
write explicit comparisons.

### Quad values and control flow

`quad` values do not implicitly control execution flow.

`if` conditions must evaluate to `bool`.

A `quad` value may enter control flow only through an explicit predicate, such
as:

```sm
if signal == T {
    ...
}
```

or through explicit `match` handling:

```sm
match signal {
    T => { ... }
    F => { ... }
    N => { ... }
    S => { ... }
    _ => { ... }
}
```

This prevents semantic uncertainty or conflict from silently selecting a
control-flow branch.

Current v0 limit:

- `if` still requires `bool` conditions; `quad` is not an implicit condition
  type

### Expression-Bodied Functions

Current function-body shorthand is stable in the current source contract:

- `fn name(...) -> ret = expr;` lowers as a single returned expression
- the expression-bodied form is equivalent to a block body whose only effect is
  `return expr;`
- the shorthand keeps terse function definitions deterministic; it does not
  introduce a distinct runtime callable shape

### Quad Predicate Selection

Current quad selection remains explicit and deterministic:

- `if q == T { ... }` and `if q == F { ... }` are the current branch-selection
  forms for quad values
- `match q { ... }` remains the compact selection form when all four quad
  states must be handled together
- `when` is the compact expression-selection form used in quad-heavy code; it
  lowers to the canonical nested expression-`if` shape
- `is` is a readable alias for explicit quad comparison and lowers to `==`
- `else if` remains nested `if` sugar in source order and does not add a new
  quad predicate family

## Tuples

Current tuple semantics:

- tuple literals evaluate their elements left-to-right and package them into a
  single aggregate value
- tuple types are structural and preserve declared element order
- tuples may flow through ordinary local bindings, function parameters, returns,
  equality, and the verified execution path

Current v0 limit:

- tuple literals and tuple types require arity at least 2
- tuples are currently value carriers only; destructuring, field access, and
  tuple-specific operators are not yet part of the stable contract
- tuple values are not part of the PROMETHEUS host ABI surface

## Option and Result Standard Forms

Current first-wave semantics:

- `Option(T)` and `Result(T, E)` are built-in standard-form types in declared
  type positions
- they are not user-declared generic enums and do not imply a general generic
  type system
- constructor evaluation reuses the canonical ADT-style carrier path
- `Option::Some(value)` produces `Option(value_type)` when no stronger context
  is required
- `Option::None` currently requires contextual `Option(T)` type from the
  surrounding typed position
- `Result::Ok(value)` and `Result::Err(error)` currently require contextual
  `Result(T, E)` type from the surrounding typed position
- current verified execution coverage now includes constructor creation plus
  explicit `Option::Some/None` and `Result::Ok/Err` match flows over
  success/none/error paths

Current v0 limit:

- the current slice does not add angle-bracket generics
- the current slice does not add user-defined parameterized declarations
- the current slice does not inject hidden prelude enums into the nominal ADT
  table
- `Option` / `Result` match ergonomics are still limited to the existing
  canonical variant pattern machinery; they do not add bare `Some/Ok` names,
  nested payload patterns, or a wider generic pattern system

## Records

Current stage-1 record semantics:

- record literals construct nominal record values using the declared record name
- each declared field must be provided exactly once
- record literal source fields may appear in any order
- the stage-1 lowering path rewrites record values into a canonical slot carrier in declaration order
- record values currently participate only in local storage and internal verified execution

Current v0 limit:

- record field access is read-only and resolves by canonical declaration-slot order
- record copy-with is immutable and rebuilds a value of the same nominal record type
- unchanged record fields in copy-with are read from the base value through canonical declaration-slot access
- record destructuring bind currently supports only statement-level nominal
  `RecordName { ... }` patterns
- record `let-else` currently supports only statement-level explicit field
  mappings with `else return`
- record `let-else` currently allows refutable matching only through explicit
  `quad` literal field targets
- record punning remains sugar over the canonical nominal record forms and does
  not introduce a separate runtime path
- anonymous brace-only record forms and nested record patterns remain out of
  scope
- record equality remains gated to the stable field-equality subset
- record values are not part of the PROMETHEUS host ABI surface

### Guard

Current `guard` semantics:

- `guard condition else return;` is allowed in `unit`-returning functions
- `guard condition else return expr;` is allowed when `expr` matches the
  function return type
- `guard` requires a `bool` condition
- `guard` is purely statement-level in the current source contract

### Match

Current `match` semantics:

- `match` currently accepts `quad` scrutinees and nominal enum scrutinees
- `quad` arms match only the literal patterns `N`, `F`, `T`, `S`
- enum arms currently use explicit nominal patterns `Enum::Variant` or
  `Enum::Variant(name, _)`
- non-default arms may attach a `bool` guard with `if guard_expr`
- `quad` `match` still requires an explicit default arm `_`
- enum `match` may omit `_` only when unguarded variant coverage is exhaustive
- `_` currently means wildcard/default only in `match`, not a general rich
  pattern system
- the first matching arm is selected deterministically

Current `match` expression semantics:

- `match scrutinee { ... }` may appear in value position
- each non-default arm uses a value-producing block after `=>`
- expression arms may use the same `if guard_expr` form as statement-side arms
- value-producing `quad` `match` still requires `_`
- value-producing enum `match` may omit `_` only when unguarded variant
  coverage is exhaustive
- all arms, including `_`, must produce the same type

This is a deliberately narrow source contract rather than a full general
pattern-matching system.

Current v0 limit:

- the default `_` arm does not yet support guards
- enum match payload patterns are currently flat only and accept only names or `_`
- guarded enum arms do not contribute to exhaustiveness
- nested enum patterns and enum literal payload checks are not yet part of the
  stable source contract

## Short Lambdas

Current short-lambda semantics:

- the published stable `v1.1.1` line keeps short lambdas as capture-free
  call-site sugar only
- `(x => expr)(arg)` is interpreted as a fresh lexical block equivalent to
  `{ let x = arg; expr }`
- `value |> (x => expr)` is interpreted as the same block sugar with `value` as
  the bound argument
- the lambda body is checked and lowered through ordinary block-expression
  semantics; no alternate runtime callable representation is introduced
- current `main` now also admits standalone first-class closure literals in
  contextual `Closure(T -> U)` positions
- admitted standalone closure literals record immutable capture inventory in
  declaration order of first use
- current `main` now materializes admitted first-class closures through one
  canonical runtime closure carrier
- current `main` now admits direct invocation of admitted closure values
  through ordinary call syntax using exactly one positional argument

Current active limits:

- the published stable line still does not claim first-class closures
- the current first-wave closure family still supports exactly one parameter
- named arguments, multi-argument forms, and non-direct callable abstraction
  remain outside the current contract
- closure equality is not part of the current first-wave surface
- typed closure parameters, multi-argument closures, and async closure forms are
  not part of the current contract

Current active closures checkpoint on `main`:

- `docs/roadmap/language_maturity/first_class_closures_full_scope.md`
- published `v1.1.1` still keeps short lambdas as capture-free sugar rather
  than as runtime closure values

## Operator Meaning

Current operator meaning:

- `==` and `!=` produce `bool`
- plain same-family `i32 <`, `<=`, `>`, and `>=` now produce `bool` on current
  `main`
- same-family `text == text` and `text != text` are now admitted on current
  `main`
- same-family `Sequence(T) == Sequence(T)` and `Sequence(T) != Sequence(T)` are
  now admitted on current `main` when `T` already supports stable equality
- `&&` and `||` work on `bool` and `quad` only when both operands are of the
  same family
- `!` works on `bool` and `quad`
- `->` is quad implication and returns `quad`
- `+`, `-`, `*`, `/` currently have stable arithmetic meaning on `f64`
- current `main` now also admits the first same-family `i32` arithmetic slice:
  unary `-`, binary `+`, `-`, `*`
- `i32 / i32` remains outside the current first arithmetic wave

Current first-wave units operator rules:

- binary and unary `+`/`-` preserve a unit annotation only for measured `f64`
  operands when both operands have the same measured type; measured `fx`
  binary and unary `+`/`-` report an explicit narrow-slice gap, and measured
  `i32`/`u32` binary and unary `+`/`-` are rejected as unsupported operators;
  SSF-07 selected this as the current boundary rather than leaving it
  undecided, and any future widening to measured `i32`/`u32` arithmetic is a
  later decision
- `==` and `!=` are valid on unit-carrying values of any base type only when
  both sides have the same measured type
- `*`, `/`, and `%` on unit-carrying values are rejected in the first-wave surface
- after unit validation, lowering reuses the existing numeric execution opcodes
  rather than widening the runtime operator family

Current honest limit:

- the published stable `v1.1.1` line keeps `fx` arithmetic intentionally
  narrower than `f64` arithmetic in the Rust-like source surface
- current `main` now admits plain `fx` unary/binary arithmetic at source typing
  level, and canonical lowering/verified execution now admit that widened
  surface under a promoted `SEMCODE3` line
- completed first-wave post-stable widening for general-purpose `fx`
  arithmetic is documented in
  `docs/roadmap/language_maturity/fx_arithmetic_full_scope.md`
- the published stable `v1.1.1` line still does not expose executable `text`
- current `main` now admits `text` through the source type/equality layer and
  through the canonical runtime carrier for those admitted programs
- current `main` now also admits bounded `text + text` concatenation and
  explicit `to_text` for the admitted scalar families represented by the
  existing fixtures
- host-facing text ABI widening is still out of scope
- current `main` now also admits one ordered sequence family with bracketed
  literals, same-family equality, and `expr[index]` through the canonical
  `M8.3` first-wave carrier path
- current `main` now also admits plain same-family `i32` relational operators
  through the existing verified compare opcodes
- current `main` now also admits plain same-family `i32` unary `-` and binary
  `+`, `-`, `*`, `/`, `%` through explicit lowering, verified opcodes, and VM
  execution
- division and modulo by zero currently surface as deterministic runtime
  failures in the benchmark-negative fixtures
- broader numeric relational surfaces for `u32`, `f64`, `fx`, and measured
  values remain outside the current application-completeness wave
- broader integer arithmetic for `u32` and mixed numeric families remains
  outside the current first arithmetic wave
- iteration, `len`, `is_empty`, maps, sets, and collection protocol machinery
  remain outside the current `M8.3` first-wave contract
- current `main` now also admits one first-wave closure family through the
  canonical verified runtime path, including immutable capture materialization
  and direct invocation with exactly one positional argument
- closure equality, host-ABI transport, and broader callable abstraction
  machinery remain outside the current `M8.4` first-wave contract

## Builtin Call Meaning

Builtin math calls are part of the source contract, not a foreign escape hatch.

Current rules:

- builtins are type-checked before lowering
- builtin math calls lower through the same call surface as ordinary functions
- `assert` shares ordinary call-like syntax but lowers to a dedicated core
  assertion opcode rather than an ordinary function call
- the verified execution path recognizes the supported builtin set explicitly

Current builtin signatures:

- `sin(f64) -> f64`
- `cos(f64) -> f64`
- `tan(f64) -> f64`
- `sqrt(f64) -> f64`
- `abs(f64) -> f64`
- `pow(f64, f64) -> f64`
- `assert(bool);` as a statement-level builtin contract

## Pipeline

Current `|>` semantics:

- `input |> stage()` is equivalent to `stage(input)`
- `input |> stage(arg1, arg2)` is equivalent to `stage(input, arg1, arg2)`
- `input |> stage(name = arg1)` is equivalent to `stage(input, name = arg1)`
- `input |> stage(name = arg1)` may still omit later trailing defaulted
  parameters
- pipeline stages are currently restricted to bare function names or ordinary
  call syntax

Current v0 limit:

- placeholder-based pipeline forms are not part of the current contract
- arbitrary right-hand expressions after `|>` are not yet supported

## Logos Semantics Boundary

The Logos-oriented surface is declarative rather than executable in the same
way as the Rust-like function path.

Current rules:

- Logos programs describe `System`, `Entity`, and `Law` declarations
- law ordering is deterministic by descending priority
- current `When` condition/effect bodies are stored as structured text
  fragments at this stage
- Logos input does not lower directly into the Rust-like SemCode function path

## Non-Goals

The current source semantics contract does not yet claim stable support for:

- exceptions
- heap/object semantics
- executable record mutation or structural record/object member model
- async or concurrency execution
- user-defined operator overloading
- lazy evaluation as a first-class source feature
- dynamic imports or runtime package resolution

## Contract Rule

Any public change to source-level call resolution, block scope behavior,
control-flow meaning, operator meaning, or builtin semantics should update this
document in the same change series.
