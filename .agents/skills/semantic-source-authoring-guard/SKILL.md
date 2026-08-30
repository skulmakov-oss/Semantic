---
name: semantic-source-authoring-guard
description: Domain guard for authoring Semantic `.sm` source, fixtures, examples, and negative diagnostic probes. Enforces fixture-first and spec-first authoring, confirmed syntax/type cribsheets, and fail-closed stop on spec-vs-fixture drift.
---

# Semantic Source Authoring Guard

Status: repository-native domain guard
Authority: subordinate to [`AGENTS.md`](../../AGENTS.md), [`CONSTRAINTS.md`](../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../.harness/current.task.yaml)

---

## 1. Purpose & Scope

This domain guard governs the creation and modification of:
- Semantic `.sm` source files;
- Positive and negative test fixtures (`tests/fixtures/**/*.sm`);
- Language examples (`examples/**/*.sm`);
- Negative diagnostic probes.

### Core Authoring Laws
1. **Never Invent Syntax**: Author exclusively from the admitted syntax, types, and library functions confirmed by normative specs and verified fixtures.
2. **Fixture-First Selection**: Search existing test suites and fixtures for the closest valid pattern before authoring new `.sm` code.
3. **No Compiler Widening for Guessed Source**: Never alter compiler, parser, sema, or runtime code simply to make invented or unconfirmed syntax compile.
4. **Diagnostic Integrity**: Negative fixtures must probe exact, deterministic diagnostic boundaries with stable error codes and message fragments.

---

## 2. Specification & Fixture Authority

### Primary Normative Specifications
- [`docs/spec/syntax.md`](../../docs/spec/syntax.md)
- [`docs/spec/types.md`](../../docs/spec/types.md)
- [`docs/spec/source_semantics.md`](../../docs/spec/source_semantics.md)
- [`docs/spec/diagnostics.md`](../../docs/spec/diagnostics.md)
- [`docs/spec/modules.md`](../../docs/spec/modules.md)
- [`docs/spec/logos.md`](../../docs/spec/logos.md)
- [`docs/LANGUAGE.md`](../../docs/LANGUAGE.md)

### Executable Fixture & Test Grounding
- `tests/frontend_lexer_qualification.rs`
- `tests/frontend_parser_qualification.rs`
- `tests/frontend_sema_qualification.rs`
- `tests/pcc6_option_result_diagnostics.rs`
- `tests/pcc7_collections_diagnostics.rs`
- `tests/pcc8_stdlib_diagnostics.rs`
- `tests/pcc9_project_model_diagnostics.rs`
- `tests/fixtures/**/*.sm`
- `examples/**/*.sm`

### Spec vs. Fixture Conflict Rule (Fail-Closed)
If normative specifications (`docs/spec/*`) and executable fixtures/tests disagree on syntax, type rules, or diagnostics:
$$\text{Specification} \neq \text{Fixture Evidence} \implies \text{\textbf{STOP \& Report Contract Drift}}$$
**Neither source may be silently chosen over the other.** Stop, report the discrepancy, and await repository-owner clarification or open a tracked contract-drift defect.

---

## 3. Mandatory Authoring Workflow

```text
1. Identify target construct
        ↓
2. Search nearest existing .sm fixture/spec
        ↓
3. Copy and adapt pattern minimally
        ↓
4. Validate with `cargo run --bin smc -- check <file.sm>`
        ↓
5. If syntax fails: fix .sm source (do not hack compiler/runtime)
        ↓
6. If construct is unconfirmed by spec/fixtures: DO NOT USE IT
```

---

## 4. Confirmed Syntax Cribsheet

Use only constructs confirmed by current normative specs and active tests:

### Top-Level Declarations
- `fn name(...) -> Type { ... }` — confirmed.
- `record Name { ... }` — confirmed.
- `schema Name { ... }` — confirmed.
- `enum Name { ... }` — confirmed where supported by current AST/sema fixtures.
- Logos `System`, `Entity`, `Law` — declarative surface; use only after confirming fixture support.

### Statements & Bindings
- `let x = ...;`, `let x: Type = ...;` — confirmed.
- `let (a, b) = ...;` (tuple destructuring) — confirmed.
- `let Record { f1, f2 } = ...;` (record destructuring) — confirmed.
- `assert ...;` — confirmed.
- `if ... { ... } else { ... }` — confirmed.
- `match ... { ... }` — confirmed.
- `return ...;` — confirmed.
- `let mut`, `const`, `let-else`, `for`, `guard` — use only if confirmed by current fixtures for the task.

### Expressions & Operators
- Literals (numeric, text, boolean, quad) — confirmed.
- Variables and call expressions — confirmed.
- Tuple literals `(a, b)` — confirmed.
- Sequence literals `[a, b, c]` — confirmed.
- Record literals `Record { f: v }` and field access `r.f` — confirmed.
- Block expressions `{ ... }`, `if` expressions, `match` expressions — confirmed.
- Arithmetic, logical, and relational operators — confirmed.

---

## 5. Confirmed Types Cribsheet

| Type | Status | Description / Limitations |
|---|---|---|
| `quad` | Confirmed | Four-state logic (`N` Null, `F` Strict False, `T` Strict True, `S` Conflict). Never treat as `bool`. |
| `bool` | Confirmed | Binary boolean (`true`, `false`). |
| `text` | Confirmed | UTF-8 validated text value. |
| `i32`, `u32`, `f64`, `fx` | Confirmed | Numeric primitives. `fx` represents fixed-point arithmetic. |
| `unit` | Confirmed | `()` unit type. |
| Tuples `(T1, T2)` | Confirmed | Fixed-length heterogenous product types. |
| Records `record R { ... }` | Confirmed | Named-field product types. |
| `Option(T)` | Confirmed | Tagged optional value (`Some(v)`, `None`). |
| `Result(T, E)` | Confirmed | Tagged outcome (`Ok(v)`, `Err(e)`). |
| `Sequence(T)` | Confirmed | Homogenous ordered collection. |
| `Map(K, V)` | Task-gated | Key-value associative mapping; use only when confirmed by fixtures. |
| `Closure(T -> U)` | Task-gated | Single-argument closure; use only when confirmed by fixtures. |

---

## 6. Forbidden Patterns & Anti-Patterns

Unless explicitly confirmed by normative specs and active tests for the assigned task, the following are strictly forbidden:
- Invented standard library functions or pseudo-methods on collections.
- Rust-style traits, generic syntax (`impl`, `<T: Trait>`), or macro invocations (`println!`).
- Implicit type coercions (e.g. `quad -> bool` or `i32 -> f64`).
- Class/OOP inheritance patterns, exceptions (`throw`, `try/catch`, `panic`).
- Direct filesystem, network, or OS calls from `.sm` source.
- Multi-line strings, string interpolation, or unescaped control characters.

---

## 7. Stop Conditions

Stop execution and report a blocker immediately if:
- **No Confirming Evidence**: The requested language construct has no backing fixture or normative spec.
- **Spec Drift**: A conflict is found between `docs/spec/*` and fixture behaviors.
- **Language Widening Required**: Validating the fixture would require unauthorized changes to compiler, parser, sema, or runtime crates.
- **Scope Misalignment**: The task is actually an architectural or compiler change masquerading as a source-only fixture task.
