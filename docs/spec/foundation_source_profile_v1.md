# Semantic Foundation Source Profile 1.0

Status: stable-candidate source contract; not published stable
Contract identifier: `semantic.foundation.source/1.0`
Evidence base: `main` at `4de0b6eb1cd5d8e5dc37989e9b9b95a5a8e07e57`
Parser acceptance envelope: `semantic.foundation` profile version `1.0`

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
later phase or roadmap and is not required for Foundation 1.0.

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
trait/protocol conformance, trait objects, associated types, blanket impls,
specialization, and default methods are experimental or deferred to SSF-07.

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
| `i32` | Literals, equality/order, unary minus, and same-family `+`, `-`, `*`, `/`, `%`. Cross-family implicit conversion is excluded. Overflow policy is frozen: `+`, `-`, `*` wrap silently on two's-complement overflow (no trap); unary minus lowers through the same `-` (`SubI32`) path and wraps identically, so `-i32::MIN` evaluates to `i32::MIN` rather than trapping; `/` and `%` trap deterministically on division/modulo by zero and on the `i32::MIN / -1` (and `i32::MIN % -1`) overflow edge case. See `crates/sm-vm/src/semcode_vm.rs`'s `vm_wraps_i32_*`/`vm_traps_on_i32_*` tests for the frozen contract evidence. |
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
Foundation 1.0 compatibility promises:

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
- non-exhaustive included-pattern matches and type-incompatible arms;
- malformed or unsupported SemCode headers at verifier admission.

The canonical diagnostic taxonomy remains `docs/spec/diagnostics.md`. SSF-09
owns a stable machine-readable schema; SSF-01 freezes deterministic rejection
behavior and existing diagnostic categories, not a new transport format.

### Diagnostic expectations

| Rejection owner | Foundation 1.0 expectation |
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
semantic.foundation.source/1.0
  -> compile using ParserProfile semantic.foundation/1.0
  -> select the oldest sufficient supported SemCode header from actual emitted use
  -> verify exact header, capabilities, structure, and instruction contract
  -> execute only after verifier admission
```

Foundation Source 1.0 does not mandate one universal SemCode header. Current
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
