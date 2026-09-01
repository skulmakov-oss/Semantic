# Semantic Foundation Source Profile 1.2

Status: stable-candidate source contract; not published stable
Contract identifier: `semantic.foundation.source/1.2`
Evidence base: `main` at `4de0b6eb1cd5d8e5dc37989e9b9b95a5a8e07e57`
Parser acceptance envelope: `semantic.foundation` profile version `1.0`

Version `1.1` documented the already-enforced `match` scrutinee allowlist and
range-pattern exhaustiveness carve-out (SSF-07) as a backward-compatible
clarification of `1.0`, with no grammar, semantic, or rejection-behavior
change. Version `1.2` is not a pure clarification: SSF-07's review process
found and fixed three real phase-consistency defects in the `match`/pattern
boundary, each backward-compatible with everything `1.1` actually promised
(no previously Included form stopped working; every change either adds new
executable capability or converts a previously undocumented, buggy edge case
into either correct behavior or a clean deterministic rejection):

- `u32` `match` moved from typecheck-only (every arm either trapped at
  runtime or failed to lower) to fully Included and executable across its
  whole domain (`0` through `u32::MAX`), for both literal and range arm
  forms. This is purely additive — no `1.1` program could have depended on
  the old trap/lowering-failure behavior, since no `u32` match ever
  produced a correct result under `1.1`.
- the exclusive equal-bound range pattern (`5..5`) moved from a silent
  miscompilation (matched the literal value `5` as if written `5..=5`,
  ignoring that it is semantically an empty range) to a deterministic
  lowering-phase rejection. `5..5` was never part of the Included
  executable surface under `1.1` — this closes an undocumented, buggy edge
  case rather than removing a promised capability.
- or-pattern match arms (`A | B`) moved from a phase-inconsistent state
  (typecheck/exhaustiveness accepted them, then lowering failed with a
  family-specific, sometimes actively misleading diagnostic) to a single,
  deterministic rejection at typecheck time, worded identically regardless
  of scrutinee family or wildcard presence. Or-patterns were never part of
  the Included executable surface under `1.1` either; this only moves the
  rejection earlier and makes its diagnostic uniform.

The evidence base commit above remains a behavioral-snapshot anchor for
SSF-01's original gathering, not a claim that every currently-mapped test
already existed there. What keeps this contract honest is the Qualification
rule below, not the frozen snapshot: the mapped evidence in
`tests/match_surface_qualification.rs` and the frontend/IR unit suites must
stay green in the current tree, not merely at the evidence-base commit.

## Authority

This document is the normative source-language contract candidate selected by
SSF-01. It is narrower than the current parser acceptance envelope and narrower
than all behavior landed on `main`.

```text
source contract != parser acceptance envelope
parser acceptance != executable qualification
executable qualification != published stable
```

Only forms listed as **Included** below belong to this contract. A form marked
**Experimental** may still be accepted by current-main tooling, but receives no
Foundation compatibility promise. A form marked **Deferred** is owned by a
later phase or roadmap and is not required for Foundation Source 1.2.

The Rust-like surface is the only executable profile governed here. Logos
remains a separate experimental declarative profile pending SSF-02.

## Version policy

- incompatible grammar or source-meaning changes require a new major contract
  version;
- backward-compatible additions or clarifications require a minor version;
- diagnostic wording may improve, but a documented diagnostic code/category
  and rejection phase must not silently change within a minor line;
- implementation on `main` does not change this contract without an explicit
  contract revision and qualification update.

The existing `ParserProfile` identity/version remains
`semantic.foundation`/`1.0`. It is a compile-time acceptance envelope and
currently admits experimental extensions. This source contract is the public
promise candidate inside that envelope; it does not rename or widen the parser
profile.

## Included executable surface

### Programs, functions, and bindings

- source files contain Rust-like items and one executable `fn main()`
  entrypoint for programs;
- named functions have explicit parameter and return types;
- `let`, `const`, `let mut`, and type-preserving assignment are included;
- lexical block scope is deterministic;
- explicit `return` is included, including `return;` for `unit`.

Overloading, variadic functions, implicit argument conversion, and runtime
dynamic dispatch are not included. The current-main `print(text)` observation
path is not part of this source contract; its public spelling and capability
contract are owned by SSF-03/SSF-04.

### Control and value-producing expressions

- `if` / `else` statements and value-producing expressions;
- bounded exhaustive `match`, including scalar, quad, enum, `Option`, and
  `Result` cases covered by the qualification contour;
- boolean match guards and compact guard-return clauses;
- `while`, statement `loop`, range `for`, and admitted sequence iteration;
- `break`, `continue`, and `return` with deterministic outside-context
  rejection;
- blocks with an optional final value expression;
- `assert(bool)` with deterministic runtime failure for false conditions.

The Rust-like `when` expression remains experimental. Logos `When` is governed
only by the Logos profile and is not executable under this contract.

### Data and patterns

- nominal records: construction, field read, function transport, equality-safe
  comparison, bounded destructuring, and immutable copy-with;
- tuples: literals, types, function transport, bounded destructuring, and
  qualified ownership-safe paths;
- nominal enums/ADTs with unit and admitted payload constructors;
- `Option(T)` and `Result(T, E)` standard variants and exhaustive match flow;
- bounded patterns and destructuring already covered by record, tuple, enum,
  Option, and Result qualification;
- `match` scrutinees are admitted at typecheck time for `quad`, nominal
  enums/ADTs, `Option(T)`, `Result(T, E)`, `i32`, and `u32`; every other
  scrutinee type (including `text`, `bool`, tuples, and records) is
  rejected deterministically at typecheck time (SSF-07). All six admitted
  scrutinee families are fully Included and executable — `u32` match now
  works across its complete domain (`0` through `u32::MAX`, both literal
  and range arm forms), lowering through a dedicated `u32` pattern-literal
  carrier instead of miscarrying every arm through an `i32` one;
- for `i32` and `u32` scrutinees, a literal match arm may be spelled either
  as a bare literal (`5 => { ... }`) or as an inclusive single-value range
  (`5..=5 => { ... }`) — both are equivalent and Included, lowering as a
  literal-equality comparison typed to the scrutinee's own family. The
  literal's type suffix, if any (`5u32`), is informative only; the
  scrutinee's declared type is what selects the `i32` or `u32` comparison
  path, so an unsuffixed literal (`5 => { ... }`) works identically against
  either family. The admitted bound range is `0` through the scrutinee
  family's own maximum (`i32::MAX` = `2147483647`, or `u32::MAX` =
  `4294967295`); a bound past that maximum is rejected deterministically at
  the lowering phase with "integer match pattern literal is outside i32
  range" or "...u32 range" respectively — see "Deterministically
  unsupported forms" below. A negative bound (`-5..=-5`) does not even
  parse (`E0000: expected match pattern`), since range-pattern parsing
  requires the current token to be a bare `Num` literal and does not admit
  a leading unary `-`; this restriction is unconditional and applies
  regardless of scrutinee family. A type-suffixed **range bound**
  (`5i32..=5i32`, as opposed to a suffixed bare literal) is a separate,
  narrower restriction and does not parse either ("range pattern bound
  does not accept a type suffix; use a plain integer"), since
  `parse_i64_pattern_bound` rejects every suffixed range bound regardless
  of scrutinee type;
- a genuine multi-value range (`1..=5` inclusive or `1..5` exclusive, over
  either `i32` or `u32`) is typecheck-only, not Included — a known M9.4
  Wave 1 boundary, rejected deterministically at the lowering phase
  ("integer range match pattern lowering is not yet implemented in the IR
  backend"). The **exclusive**, degenerate single-value form (`5..5`, over
  either `i32` or `u32`) is also typecheck-only and rejected
  deterministically at the identical lowering phase and with the identical
  diagnostic as the multi-value case: lowering only takes the
  literal-equality fast path when the range is both equal-bound **and**
  inclusive, so an exclusive equal-bound range falls through to the
  same "not yet implemented" rejection as any other unsupported range form,
  rather than being silently treated as the literal value. See
  "Deterministically unsupported forms" below for both. An incomplete
  range match without a wildcard `_` arm is still rejected deterministically
  at typecheck time through the same "match requires default arm '_'"
  check every non-exhaustive match falls back to, independent of the
  lowering gaps above. There is no tuple match-arm pattern at all — tuples
  are already excluded from the scrutinee allowlist above, and tuple
  destructuring is the separate, `let`/assignment-only mechanism already
  covered by the tuple bullet earlier in this list, not a `match`-arm
  concept;
- or-pattern match arms (`A | B`) are rejected deterministically at
  typecheck time for every admitted scrutinee family — `quad`, `i32`,
  `u32`, enum/ADT, `Option(T)`, and `Result(T, E)` — with one diagnostic
  worded identically regardless of family or wildcard presence: "or-pattern
  match arms ('A | B') are not supported; split into separate arms with
  identical bodies instead". `if let` is unaffected by this restriction, as
  it is a distinct binding construct from `match`;
- `Sequence(T)` values, indexing, iteration, length/emptiness, contains,
  persistent push/prepend/pop operations;
- `Map(K, V)` deterministic functional empty/get/set/contains operations for
  admitted key/value families.

The operations above select required source semantics, not the final
`std.seq`/`std.map` module API. SSF-03 owns importable names and compatibility
without weakening the selected behavior.

Schemas and schema migration syntax remain experimental and unpromoted.
Generalized indirect collection/ADT/schema ownership paths remain outside this
source contract pending SSF-08.

### Closures and static dispatch

- captureless, single-argument first-wave short lambdas with direct invocation;
- built-in `Sequence(T)` iteration;
- direct-record user-defined `Iterable` static dispatch in the Gate 1 contour.

Captureful, mutable, or multi-argument closures, broad generic functions, broad
trait/protocol dispatch, trait objects, associated types, blanket impls,
specialization, and default methods are experimental or deferred to SSF-07.
Broad trait/impl support itself remains an experimental, unqualified surface
(see "Experimental but currently accepted extensions" below); within that
still-experimental surface, SSF-07 has deterministically closed the
impl-declaration conformance check specifically, described there.

### Modules

- single-file programs;
- direct local-path bare helper imports;
- direct local-path selected imports over the qualified helper-module contour;
- package-qualified bare helper imports declared by the SSF-06 local package contract;
- deterministic missing-file, duplicate-symbol, and cycle rejection.

Alias imports, wildcard imports, public re-exports, namespace-qualified access,
and remote/broad package ecosystems are not included in this source contract.
Project and package identity, containment, and provenance remain owned by
SSF-05 and SSF-06 rather than by the parser/typechecker.

### Scalar families

| Family | Included contract |
|---|---|
| `quad` | Native `N`, `F`, `T`, `S`; equality and explicit evidence-aware control. No implicit normalization to `bool`. |
| `bool` | Literals, equality, boolean conditions, and admitted logic. |
| `text` | UTF-8 literal carrier, equality, concatenation, and explicit `to_text` for admitted scalar families. Indexing/slicing and general formatting are not included. |
| `i32` | Literals, equality/order, unary minus, and same-family `+`, `-`, `*`, `/`, `%`. Cross-family implicit conversion is excluded. Overflow policy is frozen: `+`, `-`, `*` wrap silently on two's-complement overflow (no trap); unary minus lowers through the same `-` (`SubI32`) path and wraps identically, so `-i32::MIN` evaluates to `i32::MIN` rather than trapping; `/` and `%` trap deterministically on division/modulo by zero and on the `i32::MIN / -1` (and `i32::MIN % -1`) overflow edge case. See `crates/sm-vm/src/semcode_vm.rs`'s `vm_wraps_i32_*`/`vm_traps_on_i32_*` tests for the frozen contract evidence; each case runs under both `OptLevel::O0` (runtime opcodes) and `OptLevel::O1` (`crystalfold.rs` constant folding) via the shared `assert_wraps_under_all_opt_levels`/`assert_traps_under_all_opt_levels` helpers, since the two are independent implementations of the same policy. |
| `u32` | Literals, equality, and `match` scrutinee/pattern selection (see "Data and patterns" above). General arithmetic, conversions, and overflow policy are deferred to SSF-07. |
| `f64` | Literals and same-family arithmetic/order. Cross-family coercion is excluded. Transcendental math builtins remain experimental until their cross-platform compatibility/determinism policy is qualified. |
| `fx` | Explicit fixed-point literals, equality/order, and the qualified same-family arithmetic contour. Cross-family and measured arithmetic remain excluded. |
| `unit` | Function/result unit value and `return;`. |

`i32` overflow policy is now frozen (see the table row above). u32 arithmetic
policy, cross-family conversion, measured numeric forms, UTF-8 indexing,
collection ordering, and advanced abstraction decisions remain owned by
SSF-07; this contract does not fill those gaps by implication.

## Experimental but currently accepted extensions

The following may parse, typecheck, or execute on current `main`, but are not
Foundation Source 1.2 compatibility promises:

- Rust-like `when`;
- schemas and schema migration forms;
- broad generics and trait/protocol forms beyond direct-record `Iterable`;
- `requires`, `ensures`, and `invariant` contracts;
- measured numeric forms;
- transcendental `f64` math builtins pending stdlib and cross-platform policy;
- broader imports/exports/module access;
- closure forms beyond the included captureless single-argument slice.

Tooling and examples must call these **experimental** or **current-main only**.
Their acceptance is not a silent contract expansion.

Within the still-experimental broad generics/trait surface, SSF-07 has
deterministically closed one specific compile-time check, split across two
validators in `crates/sm-front/src/typecheck.rs`: `validate_trait_coherence`
rejects blanket/generic impls and duplicate (trait, type) impls (pre-existing
from SSF-07's earlier blanket-impl slice), and `validate_impl_conformance`
now additionally rejects an impl whenever its method set doesn't exactly
match its trait's declared method set — no fewer, no more methods — with
matching arity/parameter/return types. This is a compile-time declaration
check only, not a compatibility promise for the broader trait/protocol
surface: impl methods have no invocation syntax anywhere in the language
outside the hardcoded
Iterable `next()` for-loop desugaring, so general trait method dispatch and
UFCS call resolution remain deferred and experimental.

First-wave impl targets are direct declared nominal records or ADTs only.
`validate_impl_conformance` proves the impl target identity exactly once,
against the canonical `RecordTable`/`AdtTable` (the same resolver every other
declared-type position in the frontend uses), before conformance succeeds —
an impl for an undeclared target rejects deterministically even when the
trait's own method signatures never reference `Self` and so would not
otherwise force that resolution. `Self` in both trait and impl method
signatures denotes exactly that one resolved concrete type and preserves its
real nominal family: `Self` for a record-target impl resolves to
`Type::Record`, `Self` for an enum/ADT-target impl resolves to `Type::Adt` —
never a guessed `Record` regardless of the target's actual family. Broader
generic/blanket/dynamic impl forms remain outside this closed surface, as
described above.

`Self` is reserved contextual type syntax, not an ordinary nominal type
name: the parser admits it only while `self_type_scope` is populated, which
is true exactly while parsing a trait method signature or an impl method's
type positions (`parse_trait_decl`/`parse_impl_decl`, via
`with_self_type_scope`), direct or nested through any currently admitted
structural type family. Before SSF-07 #1646, `Self` written anywhere else
(an ordinary function parameter/return, a record field, an ADT payload, a
`let` annotation) silently parsed as `Type::Record("Self")` — a genuine
nominal type reference to whatever happens to be named `Self`, which
existed if and only if some other declaration happened to share that name,
and rejected with an unrelated "unknown nominal type" diagnostic otherwise.
`parse_type` now rejects `Self` outside `self_type_scope` deterministically,
at the exact point recognition occurs, before any record/ADT table lookup
ever runs — this holds unconditionally, even when a record or enum is
declared with the literal name `Self`, so an ordinary type reference
spelled `Self` can never be silently resolved as that unrelated
declaration.

`TraitTable` (`build_trait_table`) is itself the admitted owner-layer trait
contract, not an unchecked cache of raw parsed declarations: every non-`Self`
parameter and return type in every trait method signature is canonicalized
against the same `RecordTable`/`AdtTable` authority before a trait is
admitted — independent of whether any impl for it exists. A declared record
name canonicalizes to `Type::Record`, a declared enum/ADT name canonicalizes
to `Type::Adt`, and an unknown or ambiguous nominal signature type rejects
deterministically, in nested positions (tuples, `Sequence`, `Option`,
`Result`, closure parameter/return) as well as directly. `Self` remains the
one reserved trait placeholder admitted through this same canonicalizer; a
trait's own declared type parameters are never admitted here, but a
signature referencing one is no longer what rejects a generic trait — see
below.

First-wave traits are non-generic: `build_trait_table` requires
`TraitDecl.type_params.is_empty()` before any method-signature work begins,
independent of whether the parameters are referenced by any method and
independent of whether an impl exists. Generic trait syntax
(`trait Foo<T> { ... }`) may still parse — the raw AST is not required to
reject it — but a non-empty `type_params` list is never erased or treated
as though the trait were non-generic; it rejects deterministically instead,
mirroring the sibling `ImplDecl.type_params` enforcement in
`validate_trait_coherence`. Generic trait syntax is future/out-of-scope
work, not a defect to paper over: no successful `TraitTable` entry can have
a non-empty `type_params`, and `Self` remains supported exactly as
described above regardless.

A canonical identity alone is not sufficient for trait admission: the
canonical type must also be on the current executable-admitted type surface.
`build_trait_table` consumes `ensure_executable_type_supported` — an
exhaustive match over every `Type` variant, with no catch-all arm, so adding
a new `Type` variant without updating this function is a compile error
rather than a silent new admission — after canonicalizing each signature
type. `QVec` is a reserved, not-yet-promoted-to-executable type family and
rejects deterministically wherever it appears in a trait method signature,
directly or nested, exactly like an unknown nominal type. `TraitTable`
success therefore means both canonical identity is proven *and* the current
executable/admitted type surface is proven — never identity alone.

First-wave generic-capable *function* definitions admit at most one type
parameter per definition site; zero is non-generic and one is the
contracted generic surface. The parser has no arity limit of its own
(`parse_type_params_with_bounds` may represent `<T, U, ...>` and mixed
bound combinations like `<T: Bound, U>` as raw AST, since `Function` has a
`trait_bounds: Vec<TraitBound>` field to faithfully retain them); admission
of the arity is enforced at `build_fn_table`, never by truncating to the
first parameter or silently admitting the extras. Traits and impls already
require zero type parameters under the stricter contracts described above,
so an out-of-contract arity there was already rejected before this boundary
existed; it is not a new restriction for those two families.

Trait bounds on a *non-function* type parameter (`record`/`enum`/`trait`/
`impl`) are a different case from bare arity, and are not admitted at the
parser at all: `TraitDecl`/`ImplDecl`/`RecordDecl`/`AdtDecl` each carry only
a bare `type_params: Vec<SymbolId>`, with nowhere to store a parsed
`TraitBound`. Before SSF-07 #1633, `parse_type_params` (the thin wrapper
these four declaration forms use, as opposed to `Function`'s
`parse_type_params_with_bounds`) parsed and then silently discarded any
bound on a non-function type parameter — the raw parser accepted
`record R<T: SomeTrait> { ... }` and returned `Ok` with a `RecordDecl` whose
`type_params` carried `T` but no trace that a bound was ever written, a
recognized-then-erased-semantics defect independent of and prior to any
later owner-layer rejection. `parse_type_params` now rejects deterministically
whenever the parsed bound list is non-empty, for all four declaration forms,
before returning to its caller. A **bare**, unbounded type-parameter list
(`record R<T> { ... }`, `enum E<T> { ... }`, `trait Tr<T> { ... }`,
`impl<T> Trait for X { ... }`) is unaffected and continues to parse as raw
AST exactly as before — its later canonical zero-arity rejection remains the
separate, later authority described above (`build_record_table`/
`build_adt_table` for #1650, `build_trait_table` for #1635,
`validate_trait_coherence` for #1668); #1633 only closes the parser-level
gap where a *bound specifically* went from recognized to discarded to a
successful, weaker AST.

Records and ADTs are non-generic in the current Stable Foundation nominal
type contract: `build_record_table`/`build_adt_table` require
`type_params.is_empty()`, rejecting any declared type parameter regardless
of arity. This is stricter than the one-parameter function surface above,
not an extension of it — `Type::Record(SymbolId)`/`Type::Adt(SymbolId)`
carry no applied type arguments, no source syntax exists to write one for a
user-declared nominal name (`parse_type`'s `Foo`/`Foo(Args)` dispatch is
hardcoded per builtin family — `Option`/`Result`/`Sequence`/`Map`/
`Closure` — with no general nominal-application rule; a bare declared name
always parses as an unparameterized `Type::Record`), and every record
literal / ADT constructor already unconditionally rejects a declaration
`TypeVar` via the same non-generic canonicalizer ordinary field/payload
type-checking uses. A generic record/ADT declaration was therefore
previously admitted while every construction of it already failed — a
phase-inconsistent, false-ready surface, not a working one. The parser
still represents `record Box<T> { ... }` / `enum Maybe<T> { ... }` as raw
AST (mirroring the trait/impl precedent above); only canonical admission is
narrowed. No applied nominal type representation, source instantiation
syntax, or field/payload monomorphisation has been implemented — that
remains a distinct, larger undertaking, not attempted here.

Generic *function* declarations that pass the frontend admission described
above are not part of the Included executable surface either: no IR
monomorphisation or specialization pass exists in `crates/sm-ir/src/passes/`
(only `StructuralCleanup` and `CrystalFold`), so a generic function's
`type_params` had no counterpart admission check anywhere in
`crates/sm-ir/src/legacy_lowering.rs` prior to SSF-07 #1717. Before that fix,
whether a generic function's declaration reached IR lowering at all depended
on incidental structure rather than a deliberate boundary: a type parameter
referenced directly or nested in `params`/`ret` happened to fail because
lowering reused the non-generic `canonicalize_declared_type`, which rejects
any `TypeVar`, but a type parameter declared and never referenced in its own
signature (e.g. `fn marker<T>(x: i32) -> i32`) had no `TypeVar` for that
incidental path to catch and silently lowered as an ordinary, non-generic
`IrFunction` — `type_params` discarded, not preserved, checked, or
specialized. `ensure_function_is_ir_concrete`, the single admission check
`lower_function_to_ir_with_tables` now runs first (the shared choke point
every public `compile_program_to_ir*`/`compile_program_to_semcode*`/
`lower_function_to_ir` entrypoint funnels through), rejects every generic
function declaration deterministically and uniformly — used or unused type
parameter, called or uncalled, trait-bounded or not — with a diagnostic
naming the function and stating that concrete IR monomorphisation is not
implemented. This closes the false-ready surface without implementing
monomorphisation: generic function *declarations* and call-site *type
inference* remain frontend-admitted and closed (#1634, #1648, #1649), but
executing one is deterministically rejected, not silently erased, until a
real specialization pass exists.

## Deterministically unsupported forms

Forms with no admitted current implementation must fail before execution with a
canonical lexer, parser, semantic, lowering, verifier, or CLI diagnostic. They
must not be ignored, guessed, or reinterpreted. This includes:

- async/await and general concurrency syntax;
- macros;
- unrestricted reflection or dynamic dispatch;
- trait objects, associated types, blanket impls, specialization, and default
  methods where no explicitly qualified form exists;
- unrestricted host I/O, networking, and process effects;
- implicit numeric conversions and unsupported cross-family arithmetic;
- malformed imports, root escapes, cycles, and unknown symbols;
- `match` over a scrutinee type outside the admitted allowlist (`quad`,
  nominal enums/ADTs, `Option(T)`, `Result(T, E)`, `i32`, `u32`) — for
  example `text`, `bool`, tuple, or record scrutinees;
- non-exhaustive included-pattern matches and type-incompatible arms;
- an or-pattern match arm (`A | B`) over any admitted scrutinee family —
  `quad`, `i32`, `u32`, enum/ADT, `Option(T)`, or `Result(T, E)` — is
  rejected deterministically at typecheck time (`build_and_apply_match_plan`
  in `crates/sm-front/src/typecheck.rs`), before exhaustiveness or lowering
  ever run, with one diagnostic worded identically for every family and
  regardless of whether a wildcard arm is also present: "or-pattern match
  arms ('A | B') are not supported; split into separate arms with identical
  bodies instead";
- a genuine multi-value integer range match arm (`1..=5` inclusive or
  `1..5` exclusive) over an `i32` or `u32` scrutinee — typechecks, then is
  rejected deterministically at the lowering phase (M9.4 Wave 1 boundary)
  with "integer range match pattern lowering is not yet implemented in the
  IR backend", not the Included executable surface;
- the exclusive, degenerate equal-bound range form (`5..5`) over an `i32`
  or `u32` scrutinee — typechecks, then is rejected deterministically at
  the identical lowering phase and with the identical diagnostic as the
  multi-value case above (lowering's literal-equality fast path requires
  both an equal bound **and** `inclusive`, so the exclusive form falls
  through to the same "not yet implemented" rejection rather than being
  treated as the literal value);
- a single-value integer range match arm whose literal bound exceeds the
  scrutinee family's own maximum (`i32::MAX` = `2147483647`, or `u32::MAX`
  = `4294967295`) — parses and typechecks (the frontend checks only the
  scrutinee family and `start <= end`, not whether the literal actually
  fits), then is rejected at the same lowering phase as the multi-value
  case above, but with its own distinct, family-typed diagnostic —
  "integer match pattern literal is outside i32 range" or "...u32 range"
  — from the equal-bounds branch of `expect_int_match_pattern`, not the
  "range lowering is not yet implemented" one;
- malformed or unsupported SemCode headers at verifier admission.

The canonical diagnostic taxonomy remains `docs/spec/diagnostics.md`. SSF-09
owns a stable machine-readable schema; SSF-01 freezes deterministic rejection
behavior and existing diagnostic categories, not a new transport format.

### Diagnostic expectations

| Rejection owner | Foundation Source 1.2 expectation |
|---|---|
| Lexer/parser | Return `FrontendErrorKind::Syntax` with a source span; Logos parser families retain `E0200` through the documented `E0237` range. |
| Profile policy | Return `FrontendErrorKind::PolicyViolation`; never reinterpret a disabled surface as another grammar form. |
| Type/semantic analysis | Return the documented deterministic category/message for type mismatch, context misuse, exhaustiveness, unsupported operation, or unresolved symbol, with the best available source span. SSF-01 does not invent numeric codes where the current taxonomy has none. |
| Modules/linkage | Preserve `E0238` through `E0245` for cycles, load failures, policy violations, collisions, re-export cycles, missing selected symbols, and select/wildcard conflicts. |
| Lowering/emission | Reject unsupported source-to-IR/SemCode forms before producing a runnable artifact; no placeholder opcode or silent erasure. |
| Verifier | Reject malformed, capability-inconsistent, or unsupported-header SemCode before VM execution. |
| Runtime | Use a deterministic trap only for failure reached by an already admitted artifact, such as assertion failure, quota exhaustion, or supported arithmetic failure. |

Exact machine-readable transport, severity normalization, and long-term
diagnostic-code compatibility remain SSF-09/SSF-10 work.

## Source-to-SemCode relationship

Source contract version and SemCode header are separate dimensions:

```text
semantic.foundation.source/1.2
  -> compile using ParserProfile semantic.foundation/1.0
  -> select SemCode header: max(structural floor, opcode/capability floor)
  -> verify exact header, capabilities, structure, and instruction contract
  -> execute only after verifier admission
```

Foundation Source 1.2 does not mandate one universal SemCode header, and
header selection is no longer driven by opcode/capability requirements
alone. Two independent floors combine, and the emitter selects the header
for whichever is higher:

- an **opcode/capability floor**: the oldest member of the documented supported family — `SEMCODE0` through `SEMCOD14`, or `SEMCOD18` when the program uses the `std.quad` QTruth family — sufficient for the opcodes a
  program actually emits. `SEMCOD14` is the floor when a program actually
  uses the included `Map(K, V)` operations, and `SEMCOD18` is the floor when
  a program actually uses `qtruth_and`/`qtruth_or`/`qtruth_not`/`qtruth_impl`
  (see #1732 / FA-05-002: no header before `SEMCOD18` legitimately admits
  those opcodes).
- a **structural floor**: since #1773 / FA-09-005, every compiled function
  envelope unconditionally carries a canonical callable-signature record
  (`SIG0`), which only a header at or above `SEMCODE_SIGNATURE_MIN_REVISION`
  (`SEMCOD19`, revision 20) can structurally carry. This floor applies
  regardless of which opcodes a program uses, including an ordinary
  zero-argument function that uses neither `Map(K, V)` nor QTruth.

The structural floor (`SEMCOD19`) is currently higher than every documented
opcode/capability floor, so **the current compiler emits `SEMCOD19` unconditionally for every compiled artifact**.
`SEMCOD19` is a structural requirement, not evidence that a program used a
new opcode capability; the opcode/capability computation above is preserved
unchanged beneath the structural floor so a future opcode needing a
still-newer revision continues to promote correctly on top of it (mirrors
the #1732 precedent: a header revision closing a version-identity gap, not a
capability gap).

This changes what the current compiler emits, not what the decoder and
verifier still admit: `SEMCODE0` through `SEMCOD18` remain decodable and
verifiable historical artifacts — an artifact compiled by an older toolchain
build does not become invalid — but only `SEMCOD19` or newer carries the
current trusted callable-contract guarantee (arity checked by the verifier,
runtime family checked by the VM before `push_frame`; see
`docs/spec/semcode.md`, `## Callable Signature (SIG0)`). A profile
permission cannot add unused capabilities, and the VM cannot reinterpret an
unknown header.

SSF-10 owns the long-term source/SemCode compatibility window and artifact
trust policy. Until that phase closes, this section is a version relationship,
not a published retention guarantee.

## Qualification rule

An Included row remains a stable candidate only while its mapped positive,
negative, lowering/SemCode, verifier, VM, canonical-example, and adversarial
evidence stays green. The executable evidence map is maintained in:

- `docs/roadmap/stable_foundation/stable_public_language_contract.md`

For `match`/pattern forms specifically, a granular per-form, per-phase
supplement to that map's summary row lives in:

- `docs/roadmap/stable_foundation/ssf07_pattern_qualification_matrix.md`

If evidence regresses, the feature returns to experimental/unqualified status;
the contract must not be preserved by documentation alone.

## Explicit non-claims

This contract does not claim published stable status, production readiness, a
stable ABI/ISA, a broad or comprehensive stdlib, a package ecosystem, broad
host effects, Rust-equivalent ownership, executable Logos, or editor/tooling
completion.
