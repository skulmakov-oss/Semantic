---
name: semantic-source-authoring-guard
description: Use when writing Semantic `.sm` source files, fixtures, examples, or negative diagnostic probes. Enforce fixture-first and spec-first authoring for current Semantic syntax, types, and diagnostics; prevent invented syntax or unconfirmed language behavior.
---

# Semantic Source Authoring Guard

## Purpose

Help agents write valid Semantic `.sm` source from the **current admitted surface**.

Hard rules:
- Do not invent Semantic syntax.
- Search existing fixtures and tests before writing new `.sm`.
- Prefer repo evidence over memory, intuition, or roadmap prose.
- Do not widen compiler, runtime, parser, verifier, or stdlib behavior to make invented source compile.

## Relationship to Semantic Admission Guardian

`Semantic Admission Guardian` controls:
- repository safety;
- mutation discipline;
- gates;
- PR workflow;
- admission checks.

`Semantic Source Authoring Guard` controls:
- `.sm` source authoring;
- fixture-first syntax selection;
- examples and probes;
- negative diagnostic fixture discipline;
- no invented language.

The two skills are complementary and non-overlapping.

## Boundary With Repository Architecture Work

This skill is only for Semantic `.sm` source authoring, fixtures, examples, and diagnostic probes.

Use the main `semantic` skill instead for:
- Rust source changes;
- repository architecture;
- UI / renderer work;
- roadmap / closeout / ledger PRs;
- Project #2 metadata;
- capability / runtime / verifier / VM ownership;
- Workbench / Studio boundaries;
- agent skill governance.

If a task combines `.sm` fixtures with architecture, renderer, roadmap, verifier, runtime, capability, or Project #2 workflow:
1. Use the main `semantic` skill for repository and architectural boundaries.
2. Use this source authoring guard only for the `.sm` fixture/source portion.
3. Do not let `.sm` fixture convenience widen compiler, parser, verifier, runtime, or UI behavior.

## Required Source Authorities

Read these canonical docs first:
- `docs/spec/syntax.md`
- `docs/spec/types.md`
- `docs/spec/source_semantics.md`
- `docs/spec/diagnostics.md`
- `docs/spec/modules.md`
- `docs/spec/logos.md`
- `docs/LANGUAGE.md`

Treat these live paths as stronger evidence than intuition and stronger than future-facing prose:
- `tests/frontend_lexer_qualification.rs`
- `tests/frontend_parser_qualification.rs`
- `tests/frontend_sema_qualification.rs`
- `tests/pcc6_option_result_diagnostics.rs`
- `tests/pcc7_collections_diagnostics.rs`
- `tests/pcc8_stdlib_diagnostics.rs`
- `tests/pcc9_project_model_diagnostics.rs`
- `tests/fixtures/**/*.sm`
- `examples/**/*.sm`

If docs and fixtures disagree, prefer the fixture/test evidence for current authoring and stop if the conflict cannot be resolved safely.

## DNA-Aware Source Authoring

For `.sm` authoring that touches public examples, doctrine, roadmap-facing examples, UI examples, capability examples, verifier examples, runtime examples, or language identity examples, inspect `docs/dna` before authoring.

DNA-sensitive `.sm` work must report:
- docs/dna inspected: YES/NO
- DNA files inspected:
- DNA alignment:
- DNA conflicts detected:
- DNA-driven constraints applied:

If docs/dna conflicts with the proposed `.sm` example or fixture framing, stop and report the contradiction.

If docs/dna is not relevant to a tiny local negative fixture, say so explicitly.

## Mandatory Authoring Workflow

1. Identify the target construct.
2. Search for the closest existing `.sm` fixture or example.
3. Copy the nearest valid pattern.
4. Modify minimally.
5. Run `smc check <file>`.
6. If `smc check` fails, fix the `.sm` source first.
7. Do not change compiler, runtime, parser, verifier, or stdlib code to make invented source compile.
8. If no fixture/spec confirms a construct, do not use it.

## Syntax Cribsheet

Use only constructs confirmed by the current spec/tests/fixtures.

### Top-level declarations
- `fn` - confirmed by spec/test.
- `record` - confirmed by spec/test.
- `schema` - confirmed by spec/test.
- `enum` - use only after confirming current fixture/spec support.
- Logos `System`, `Entity`, `Law` - use only after confirming current fixture/spec support.

### Function forms
- `fn name(...) { ... }` - confirmed by spec/test.
- Explicit return types - confirmed by spec/test.
- Expression-bodied forms - use only after confirming current fixture/spec support.
- Default parameters - use only after confirming current fixture/spec support.
- `requires`, `ensures`, `invariant` - use only after confirming current fixture/spec support.

### Statements
- `let` - confirmed by spec/test.
- `let mut` - use only after confirming current fixture/spec support.
- `const` - use only after confirming current fixture/spec support.
- tuple destructuring - confirmed by spec/test.
- record destructuring - confirmed by spec/test.
- `let-else` - use only after confirming current fixture/spec support.
- assignment and compound assignment - confirmed by spec/test.
- `for` - use only after confirming current fixture/spec support.
- `guard` - use only after confirming current fixture/spec support.
- `assert` - confirmed by spec/test.
- `if` - confirmed by spec/test.
- `match` - confirmed by spec/test.
- `return` - confirmed by spec/test.

### Expressions
- literals - confirmed by spec/test.
- variables - confirmed by spec/test.
- calls - confirmed by spec/test.
- named-argument calls - use only after confirming current fixture/spec support.
- method/UFCS sugar - use only after confirming current fixture/spec support.
- pipeline - use only after confirming current fixture/spec support.
- where suffix - use only after confirming current fixture/spec support.
- tuple literals - confirmed by spec/test.
- sequence literals/usages - confirmed by spec/test.
- record literals - confirmed by spec/test.
- record field access - confirmed by spec/test.
- record copy-with - use only after confirming current fixture/spec support.
- block expressions - confirmed by spec/test.
- if expressions - confirmed by spec/test.
- match expressions - confirmed by spec/test.
- loop/break expressions - use only after confirming current fixture/spec support.
- unary/binary operators - confirmed by spec/test.
- range literals - use only after confirming current fixture/spec support.

### Logos
- treat Logos as a separate declarative surface.
- do not mix Logos forms into executable source unless the current spec/tests confirm the pattern.

## Type Cribsheet

### Confirmed core types
- `quad` - confirmed by spec/test.
- `bool` - confirmed by spec/test.
- `text` - confirmed by spec/test.
- `i32` - confirmed by spec/test.
- `u32` - confirmed by spec/test.
- `f64` - confirmed by spec/test.
- `fx` - confirmed by spec/test.
- `unit` - confirmed by spec/test.
- measured numeric forms such as `i32[unit]`, `u32[unit]`, `f64[unit]`, `fx[unit]` - use only after confirming current fixture/spec support.
- tuples - confirmed by spec/test.
- records - confirmed by spec/test.
- enums / ADTs - confirmed by spec/test.
- `Option(T)` - confirmed by spec/test.
- `Result(T, E)` - confirmed by spec/test.
- `Sequence(T)` - confirmed by spec/test.
- `Map(K, V)` - use only when confirmed by current fixtures/tests/specs for the task.
- `Closure(T -> U)` - use only when confirmed by current fixtures/tests/specs for the task.
- `qvec(N)` - reserved parser-level family; do not author unless the task explicitly confirms it.

For each type, confirm:
- where it may appear;
- how it is constructed;
- how it is matched;
- which operations are legal;
- which limitations are explicit.

## Confirmed Source Patterns

Use only tiny examples that are already confirmed by spec or existing tests/fixtures.

- minimal `fn main` - confirmed.
- typed `let` - confirmed.
- `assert` - confirmed.
- `quad` match - confirmed when backed by spec/test.
- `Option(T)` construction and match - confirmed.
- `Result(T, E)` construction and match - confirmed.
- record literal and field access - confirmed.
- `Sequence(T)` literal/usage - confirmed.
- `Map(K, V)` usage - only when confirmed by fixture/test.
- one negative diagnostic fixture pattern - confirmed when the diagnostic is already stable.

Keep examples tiny. Do not build app-style samples.

## Forbidden Patterns

Ban these unless the current fixtures/specs explicitly confirm them:
- invented Semantic syntax;
- invented stdlib calls;
- invented methods on `Option`, `Result`, `Sequence`, or `Map`;
- Rust traits/generics syntax;
- `impl`;
- macros;
- `async` / `await`;
- Python/Rust/TypeScript import styles;
- implicit type coercions;
- text interpolation;
- multi-line strings;
- file, network, or host I/O;
- classes / OOP patterns;
- `panic`, `throw`, `try/catch`;
- hidden runtime effects;
- large app-style examples.

## Diagnostic Fixture Rules

When writing negative `.sm` fixtures:
- one fixture must represent one diagnostic boundary;
- prefer stable message fragments over full text;
- do not add broad snapshots unless the existing suite style requires it;
- do not change production code unless the task explicitly asks for implementation work;
- if behavior is unclear, stop and report the uncertainty.

## Verification Workflow

Use the narrowest checks that match the task:
- `smc check <file>`
- `cargo test --test <targeted_test_name> --quiet`
- `pwsh scripts\admission_guard.ps1 -PRReady`
- `pwsh scripts\admission_guard.ps1 -Readiness`

Use `FullPreflight` only when the current repo workflow requires it.

If the task is a small `.sm` probe, do not force broad gates unless the repository workflow demands them.

## Stop Conditions

Stop and report if:
- no confirming fixture/spec exists;
- `smc check` fails because the syntax is unsupported;
- the requested fixture requires language widening;
- the task would require compiler/runtime/parser/verifier changes but was scoped as test-only;
- you are about to invent syntax to satisfy the task;
- the `.sm` source would conflict with docs/dna project identity;
- the task is actually repository architecture / renderer / roadmap work and should use the main `semantic` skill first.

## Acceptance Criteria

A skill-use is correct when it:
- stays fixture-first and spec-first;
- uses current `.sm` evidence instead of intuition;
- prevents invented syntax;
- keeps `.sm` examples minimal;
- separates source authoring from repo admission and PR workflow;
- avoids language/runtime/CI changes.
