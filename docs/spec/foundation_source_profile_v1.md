# Semantic Foundation Source Profile 1.1

Status: stable-candidate source contract; not published stable
Contract identifier: `semantic.foundation.source/1.1`
Evidence base: `main` at `4de0b6eb1cd5d8e5dc37989e9b9b95a5a8e07e57`
Parser acceptance envelope: `semantic.foundation` profile version `1.0`

Version `1.1` is a backward-compatible clarification per the version policy
below: it documents the already-enforced `match` scrutinee allowlist and
range-pattern exhaustiveness carve-out (SSF-07), with no grammar, semantic,
or rejection-behavior change from `1.0`.

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
later phase or roadmap and is not required for Foundation Source 1.1.

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
- `match` scrutinees are admitted only for `quad`, nominal enums/ADTs,
  `Option(T)`, `Result(T, E)`, `i32`, and `u32`; every other scrutinee type
  (including `text`, `bool`, tuples, and records) is rejected deterministically
  at typecheck time (SSF-07);
- exhaustiveness checking over sum-family scrutinees (enum/ADT, `Option(T)`,
  `Result(T, E)`) at typecheck time covers wildcard and or-pattern
  expansion, but **or-pattern arms (`A | B`) are typecheck-only and not
  part of the Included executable surface**: confirmed empirically (enum
  and `Option(T)` scrutinees, with and without an accompanying wildcard
  arm) that lowering rejects them deterministically — `compile` itself
  fails, not just `run` — with a lowering-side re-check that cannot expand
  an or-pattern, producing a confusing "non-exhaustive match" diagnostic
  even though `check` already accepted the program as exhaustive; see
  "Deterministically unsupported forms" below;
- integer range match arms are typecheck-only for `i32`/`u32` scrutinees at
  the same allowlist above, but lowering support is narrower still and
  differs by scrutinee family (SSF-07, confirmed empirically against the
  actual CLI for each case below — none of this is assumed):
  - `i32` scrutinee, single-value range (`5..=5`): Included and executable;
    lowers as a literal-equality match;
  - `i32` scrutinee, genuine multi-value range (`1..=5`): typecheck-only —
    a known M9.4 Wave 1 boundary, rejected deterministically at the
    lowering phase ("integer range match pattern lowering is not yet
    implemented in the IR backend", `crates/sm-ir/src/legacy_lowering.rs`);
  - `u32` scrutinee, any range (single- or multi-value): typecheck-only and
    **not currently safe to rely on** — a small-value range (`5..=5`)
    compiles successfully but then traps unpredictably at runtime with an
    internal "runtime type mismatch: CmpEq/CmpNe operands must have same
    runtime type" error rather than a deterministic rejection at any
    earlier phase; a large-value range (e.g. `u32::MAX..=u32::MAX`) does
    fail deterministically, but only because the pattern-bound conversion
    unconditionally goes through `i32::try_from` regardless of scrutinee
    type ("integer match pattern literal is outside i32 range");
  - an incomplete range match without a wildcard `_` arm is still rejected
    deterministically at typecheck time through the same "match requires
    default arm '_'" check every non-exhaustive match falls back to,
    independent of the lowering gaps above.
  There is no tuple match-arm
  pattern at all — tuples are already excluded from the scrutinee allowlist
  above, and tuple destructuring is the separate, `let`/assignment-only
  mechanism already covered by the tuple bullet earlier in this list, not a
  `match`-arm concept;
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
| `u32` | Literals and equality only. General arithmetic, conversions, and overflow policy are deferred to SSF-07. |
| `f64` | Literals and same-family arithmetic/order. Cross-family coercion is excluded. Transcendental math builtins remain experimental until their cross-platform compatibility/determinism policy is qualified. |
| `fx` | Explicit fixed-point literals, equality/order, and the qualified same-family arithmetic contour. Cross-family and measured arithmetic remain excluded. |
| `unit` | Function/result unit value and `return;`. |

`i32` overflow policy is now frozen (see the table row above). u32 arithmetic
policy, cross-family conversion, measured numeric forms, UTF-8 indexing,
collection ordering, and advanced abstraction decisions remain owned by
SSF-07; this contract does not fill those gaps by implication.

## Experimental but currently accepted extensions

The following may parse, typecheck, or execute on current `main`, but are not
Foundation Source 1.1 compatibility promises:

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
- an or-pattern match arm (`A | B`) over an enum/ADT or `Option(T)`/`Result(T, E)`
  scrutinee — typechecks (including as sole exhaustiveness coverage), then
  is rejected deterministically at the lowering phase regardless of whether
  a wildcard arm is also present, not the Included executable surface;
- a genuine multi-value integer range match arm (`1..=5`, as opposed to a
  single-value arm like `5..=5`) over an `i32` scrutinee — typechecks, then
  is rejected deterministically at the lowering phase (M9.4 Wave 1
  boundary), not the Included executable surface;
- any integer range match arm over a `u32` scrutinee — typechecks, then
  either fails to lower (large values) or, more concerning, compiles
  successfully but traps at runtime with an internal type-mismatch error
  (small values) rather than a deterministic rejection; not the Included
  executable surface pending a proper fix;
- malformed or unsupported SemCode headers at verifier admission.

The canonical diagnostic taxonomy remains `docs/spec/diagnostics.md`. SSF-09
owns a stable machine-readable schema; SSF-01 freezes deterministic rejection
behavior and existing diagnostic categories, not a new transport format.

### Diagnostic expectations

| Rejection owner | Foundation Source 1.1 expectation |
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
semantic.foundation.source/1.1
  -> compile using ParserProfile semantic.foundation/1.0
  -> select the oldest sufficient supported SemCode header from actual emitted use
  -> verify exact header, capabilities, structure, and instruction contract
  -> execute only after verifier admission
```

Foundation Source 1.1 does not mandate one universal SemCode header. Current
qualified programs may select a member of the documented supported family from
`SEMCODE0` through `SEMCOD14` according to actual emitted features — `SEMCOD14`
is the header selected when a program actually uses the included `Map(K, V)`
operations. A profile permission cannot add unused capabilities, and the VM
cannot reinterpret an unknown header.

SSF-10 owns the long-term source/SemCode compatibility window and artifact
trust policy. Until that phase closes, this section is a version relationship,
not a published retention guarantee.

## Qualification rule

An Included row remains a stable candidate only while its mapped positive,
negative, lowering/SemCode, verifier, VM, canonical-example, and adversarial
evidence stays green. The executable evidence map is maintained in:

- `docs/roadmap/stable_foundation/stable_public_language_contract.md`

If evidence regresses, the feature returns to experimental/unqualified status;
the contract must not be preserved by documentation alone.

## Explicit non-claims

This contract does not claim published stable status, production readiness, a
stable ABI/ISA, a broad or comprehensive stdlib, a package ecosystem, broad
host effects, Rust-equivalent ownership, executable Logos, or editor/tooling
completion.
