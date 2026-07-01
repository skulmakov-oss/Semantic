# Semantic Language

Semantic Language is not just a programming language and not just syntax sugar over an existing VM.
It is a utilitarian language for describing reasoning processes, inference rules, and semantic state transitions inside the PROMETHEUS system model.

The canonical public language name is `Semantic Language`. `Semantic` is the short form used in repository prose.

Its purpose is to express meaning-oriented logic with the same rigor that ordinary languages apply to computation.

Semantic is built as a minimal compiler stack that ends in a deterministic virtual machine.

The canonical language-level public contract is now centered in the spec
bundle:

- `docs/spec/syntax.md`
- `docs/spec/types.md`
- `docs/spec/source_semantics.md`
- `docs/spec/diagnostics.md`
- `docs/spec/modules.md`
- `docs/spec/logos.md`

Supporting overview notes such as this page, `docs/imports.md`, and
`docs/exports.md` should stay aligned with that bundle rather than define a
separate competing source contract.

## Core Idea

Semantic is aimed at programs that encode:

- rules of inference;
- transitions between semantic states;
- explicit logic over `quad`-style values;
- deterministic executable behavior suitable for kernel-adjacent or system-level environments.

The project is opinionated:

- deterministic compilation matters;
- deterministic execution matters;
- validation happens before execution;
- the VM must fail in isolation rather than destabilize the host system.

## Stack

The path from source text to execution is split into strict phases.

### 1. Frontend

The frontend parses Semantic source into structured syntax and performs early semantic work:

- parsing;
- syntax normalization;
- type checking;
- alias resolution.

In the current workspace this layer lives primarily in:

- `crates/sm-front`
- `crates/sm-sema`

The language is intentionally centered on a constrained set of explicit values and operations. In the current public toolchain, `quad`, `bool`, and integer-oriented execution are central to the model, with the VM and bytecode format kept deliberately simple.

At the public language-contract level, the source surface is currently split
into:

- a Rust-like executable function surface
- a declarative Logos surface for `System`, `Entity`, and `Law`
- a deterministic file/module import and re-export surface

### 2. Lowering to IR

After parsing and analysis, the program is lowered into an intermediate representation.

In the current workspace this responsibility is centered in:

- `crates/sm-ir`

Higher-level control flow such as:

- `if` / `else`
- `match`
- quad-state branching

is flattened into explicit control-flow instructions and labels. The result is a more mechanical form that is easier to validate, optimize, serialize, and execute.

### 3. Optimization

Before bytecode emission, the IR can be simplified by deterministic optimization passes.

This includes ideas such as:

- constant folding;
- removal of unnecessary intermediate work;
- simplification of control-flow structure.

The important rule is not maximal optimization, but predictable optimization. Semantic prefers transformations that preserve clarity, stability, and reproducibility.

### 4. SemCode Backend

After IR is finalized, the program is emitted into SemCode bytecode.

In the current repository this is represented by:

- `crates/sm-emit`
- `src/semcode_format.rs`

SemCode is the binary contract between compiler and VM. The current format family uses versioned headers such as:

- `SEMCODE0`
- `SEMCODE1`

Before execution, the VM validates the header and layout so that malformed or incompatible bytecode is rejected before it can run.

### 5. Semantic VM

Execution happens inside a deterministic register-oriented virtual machine.

In the current repository this layer is represented by:

- `crates/sm-vm`
- `src/bin/svm.rs`

The VM is designed to isolate execution failures:

- malformed bytecode is rejected;
- invalid runtime states are surfaced as VM errors;
- execution failure should stay inside the VM boundary instead of taking down the surrounding system.

## Design DNA

Semantic is not an attempt to invent everything from zero. It is a synthesis of strong ideas from several traditions.

### Rust

Semantic borrows the discipline of:

- explicit syntax;
- strong typing;
- structured diagnostics;
- `enum`/`match`-oriented modeling;
- compiler architecture that values correctness before convenience.

### Java

Semantic borrows the idea of a versioned bytecode contract enforced by a VM. The comparison is architectural, not aesthetic:

- compile first;
- validate bytecode;
- run inside an isolated runtime boundary.

### Python

Semantic borrows some of the ecosystem mindset rather than Python's runtime model:

- flexible tooling culture;
- profile-driven workflows;
- emphasis on readable source forms.

Semantic does not aim to inherit Python-style dynamic execution semantics.

### Verilog / VHDL

The quad-state model is philosophically close to hardware logic. Semantic takes inspiration from four-valued logic, but lifts it from wires and signals into semantic reasoning.

The central model is:

- `F` - false
- `T` - true
- `N` - unknown
- `S` - conflict

### eBPF

Semantic shares the systems goal that user-defined logic should run inside a constrained execution environment without threatening host stability.

The key idea is:

- controlled execution;
- deterministic behavior;
- isolation of failure.

### Prolog

Semantic also inherits part of the reasoning mindset: implication and rule expression are treated as first-class language concerns rather than incidental library patterns.

## Parser Profiles

One of the language's more distinctive directions is the idea of adaptive parsing through parser profiles.

The design goal is that syntax aliases can be learned or configured without rebuilding the entire system. In that model, different surface forms can map onto the same semantic operator.

Examples of the intended idea:

- `AND`
- `&`
- domain-specific aliases

all becoming equivalent at the parser-profile layer.

This is best understood as a language adaptation mechanism, not as permission for uncontrolled syntax drift.

Current public policy-gated areas include:

- `f64` math availability
- Logos surface availability
- selected legacy compatibility paths

## What Makes Semantic Different

Semantic tries to combine:

- the structural rigor of Rust;
- the bytecode contract mentality of Java;
- the isolation mindset of eBPF;
- the four-valued logic heritage of hardware description languages;
- the inference-oriented spirit of Prolog.

The goal is not just to compute values.
The goal is to represent and execute reasoning in a deterministic, machine-checkable form.

## Current Repository Reality

The repository already reflects part of this architecture directly:

- `smc` is the compiler/tooling entrypoint;
- `svm` is the VM entrypoint;
- `.sm` is Semantic source;
- `.sem` is textual machine/intermediate form;
- `.smc` is compiled SemCode bytecode;
- `TON618 Core` is the low-level core identity;
- `SemCode` is the bytecode/runtime format family.

For naming conventions, see `docs/NAMING.md`.
