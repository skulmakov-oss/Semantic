# Semantic DNA

This document defines the architectural DNA of Semantic: what the project intentionally learns from other systems, what it deliberately rejects, and what remains unique to Semantic itself.

## Principles

- Do not copy syntax for its own sake; extract strong engineering decisions and adapt them to Semantic's execution model.
- Prioritize readable Logos, strict semantics, verifier-first execution, and a predictable VM.
- Every feature must fit the staged pipeline: `Source -> AST -> IR -> SemCode -> Verifier -> VM`.
- Language design must remain deterministic at compile time and at runtime.
- Public concepts must stay stable, minimal, and explainable.

## Borrowed Engineering Ideas

### Python

Semantic adopts:

- `INDENT` / `DEDENT` as a disciplined block model for the Logos frontend.
- Logical lines and continuation depth for readable multi-line expressions.
- Friendly diagnostics: `Expected X, got Y`, caret rendering, and actionable hints.
- Docstring-like metadata comments for `Law` and `Entity` declarations.
- A REPL culture for short development cycles and fast feedback.

Semantic does not adopt:

- Dynamic typing as the default execution model.
- Runtime monkey-patching.
- Implicit execution magic.
- Hidden mutation behind convenient syntax.

### Rust

Semantic adopts:

- `Span` / `SourceMark` tracking across all compiler stages.
- String interning through `SymbolId` instead of raw `String` usage in hot compiler and runtime paths.
- Parser discipline, including Pratt-style expression parsing where appropriate.
- Diagnostics with labels, context, and stable error codes.
- `no_std` awareness and a low-overhead implementation style.
- `enum` + `match` as a foundation for AST, IR, and VM models.
- RAII-style guard patterns where they improve correctness.
- Feature-gated architecture for controlled surface expansion.

Semantic does not adopt:

- A full lifetime model exposed in the user-facing language surface.
- Macro-heavy design as the foundation of the language.
- Compile-time cleverness that makes diagnostics or verification opaque.

### Java

Semantic adopts:

- Bytecode as a stable execution contract.
- A versioned binary format.
- Magic headers and structured metadata sections.
- Constant-pool style separation where it improves determinism and compactness.
- Strict validation between compile and execution stages.

Semantic does not adopt:

- Object-oriented inheritance as the main language frame.
- Verbose enterprise-style structure.
- Checked-exception style control flow.

### C++

Semantic adopts:

- Explicit layout control where ABI or VM boundaries require it.
- Minimal runtime overhead.
- Compile-time folding as an optimization discipline.
- Predictable memory behavior.
- A strong preference for explicit ownership of low-level representations.

Semantic does not adopt:

- Template or metaprogramming complexity for its own sake.
- Operator overloading that creates ambiguous meaning.
- Undefined behavior as an acceptable optimization tool.

### ML / Haskell

Semantic adopts:

- Algebraic data types as a clear data-modeling tool.
- Pattern matching with explicit coverage expectations.
- A preference for total and checkable semantic forms.

Semantic does not adopt:

- Laziness as the default execution model.
- Monads as a mandatory programming model.
- Highly abstract notation that weakens operational clarity.

### Lisp

Semantic may later adopt:

- Treating structured programs as data.
- A carefully bounded macro layer.

Semantic does not adopt in the near term:

- Fully open-ended macro expansion.
- Runtime code rewriting as a normal programming pattern.
- Language-level flexibility that bypasses verifier guarantees.

### Erlang / Elixir

Semantic adopts as architectural inspiration:

- Fault isolation.
- Supervisor-style thinking.
- Event-driven orchestration.
- Clear boundaries between a failing unit and the surrounding system.

Semantic does not adopt:

- Actor-style concurrency as an early core requirement.
- Distributed runtime semantics as part of the initial language core.

### ECS / Engine Architecture

Semantic adopts:

- `Entity = data`, `Law = system-like rule` as a useful modeling analogy.
- Deterministic ticks.
- Fixed-step execution where reproducibility matters.
- Separation between state, rule evaluation, and effects.

Semantic does not adopt:

- Frame-driven game-engine assumptions as a universal model.
- Implicit global mutable state.
- Performance shortcuts that break deterministic replay.

## What Remains Unique to Semantic

- Native quad logic (`N` / `F` / `T` / `S`) as a first-class semantic domain.
- Conflict and unknown states represented explicitly instead of being collapsed into boolean logic.
- Merge-oriented computation as a core design philosophy.
- Verifier-first execution: a program is not executed merely because it was parsed or compiled.
- SemCode as a deterministic, versioned execution contract.
- Runtime profiles for controlled feature activation.
- A VM designed for deterministic semantic programs rather than general scripting convenience.
- A strict boundary between compiler semantics, execution semantics, and host-side effects.
- Capability-gated integration with external systems.
- Local auditability and reproducible execution traces as part of the system model.

## Architectural Invariants

- Every new extension must preserve deterministic compilation and deterministic runtime behavior.
- Diagnostics must be source-anchored through `SourceMark` / `Span` and reproducible across runs.
- `no_std` and `alloc` boundaries must be considered during design, not added after implementation.
- IR and SemCode versions may evolve only through explicit compatibility and migration rules.
- The VM must execute only verified SemCode.
- Host effects must pass through explicit capability and ABI boundaries.
- Runtime hot paths must avoid string-heavy representations where compact IDs are available.
- Any feature that weakens verification, determinism, or auditability is outside the core DNA.
