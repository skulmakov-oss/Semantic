# Semantic Language Wiki — Current Status

**Status:** current-main orientation page  
**Audience:** first-time readers, users, evaluators, and contributors  
**Last synchronized:** July 13, 2026, after the user-first README refresh

This page is the compact Wiki overview of Semantic Language. It explains what the platform is, why its execution model is different, what can be used today, and which parts remain deliberately limited.

For the shortest hands-on route, use the [Getting Started guide](../getting_started.md). For the repository landing page, return to the [README](../../README.md).

---

## 1. Semantic in one minute

Semantic is a deterministic, verifier-first language platform for programs that need explicit reasoning states, controlled execution, and visible uncertainty.

A Semantic program follows a staged path:

```text
.sm source
   -> frontend and semantic analysis
   -> deterministic IR
   -> SemCode (.smc)
   -> verifier admission
   -> deterministic VM
   -> optional capability-controlled host boundary
```

The important separation is:

```text
source describes
compiler lowers
verifier admits or rejects
VM executes
capability boundary controls effects
audit records controlled effects where supported
UI displays but does not become authority
```

Semantic is not presented as a finished general-purpose ecosystem or a production-ready replacement for mainstream languages. It is a serious R&D platform with a real executable pipeline, a published stable line, a qualified limited-release contour, and additional work already landed on current `main`.

---

## 2. Why four-state logic?

Traditional boolean logic usually offers two visible values:

```text
false
true
```

Real decision systems often need to distinguish two additional cases:

- insufficient evidence;
- conflicting evidence.

Semantic exposes those cases through the native `quad` type:

| Value | Meaning |
|---|---|
| `N` | unknown / insufficient evidence |
| `F` | false |
| `T` | true |
| `S` | conflict / incompatible evidence |

These values are semantic states, not boolean aliases.

A useful two-plane representation is:

```text
N = (0, 0)
F = (0, 1)
T = (1, 0)
S = (1, 1)
```

Branching remains explicit:

```sm
if state == T {
    // confirmed true
}

if state == S {
    // conflict is handled deliberately
}
```

The following is intentionally rejected:

```sm
if state {
    // quad values do not have implicit truthiness
}
```

Use explicit comparison or `match` when all four states matter.

---

## 3. Try the current toolchain

### Prerequisites

- a current Rust toolchain;
- Git;
- Windows, Linux, or macOS.

### Clone and build

```bash
git clone https://github.com/skulmakov-oss/Semantic.git
cd Semantic
cargo build --bin smc --bin svm
```

### Run a canonical example

```bash
cargo run --bin smc -- run examples/canonical/rule_state_decision/src/main.sm
```

This example demonstrates records, `quad`, explicit decision logic, `Result`, verifier-first execution, and deterministic assertions.

### Inspect the complete artifact route

```bash
cargo run --bin smc -- check examples/canonical/rule_state_decision/src/main.sm
cargo run --bin smc -- compile examples/canonical/rule_state_decision/src/main.sm -o decision.smc
cargo run --bin smc -- verify decision.smc
cargo run --bin smc -- run-smc decision.smc
cargo run --bin svm -- disasm decision.smc
```

This proves the full path:

```text
source check
  -> SemCode emission
  -> verifier admission
  -> verified artifact execution
  -> disassembly
```

---

## 4. A small Semantic program

```sm
fn decide(sensor: quad, ready: bool) -> quad {
    if sensor == N {
        return N;
    }

    if sensor == S {
        return S;
    }

    if ready == true {
        return T;
    }

    return F;
}

fn main() {
    let verdict: quad = decide(T, true);
    assert(verdict == T);
}
```

Save it as `decision.sm`, then run:

```bash
cargo run --bin smc -- check decision.sm
cargo run --bin smc -- run decision.sm
```

To create and admit a persisted artifact:

```bash
cargo run --bin smc -- compile decision.sm -o decision.smc
cargo run --bin smc -- verify decision.smc
cargo run --bin smc -- run-smc decision.smc
```

---

## 5. Current public status

Semantic uses four explicit status families. These must not be blurred together.

| Status | Meaning |
|---|---|
| **Published stable** | Promised by the published stable line. |
| **Qualified limited release** | Proven in a bounded practical contour by qualification evidence. |
| **Landed on current `main`, not yet promised** | Implemented or benchmark-qualified, but not promoted into the stable or qualified release promise. |
| **Out of scope** | Deliberately excluded from the current release contour. |

The published stable line is currently:

```text
v1.1.1
```

The key rule is:

```text
landed on main != qualified limited release != published stable
```

For release-sensitive decisions, use these authorities:

- [Public Status Model](../roadmap/public_status_model.md);
- [Semantic v1 Readiness](../roadmap/v1_readiness.md);
- [Feature Maturity Matrix](../status/feature_maturity_matrix.md);
- [Public Maturity Snapshot](../roadmap/public_maturity_snapshot.md).

---

## 6. What is qualified today?

The current qualified limited-release contour includes:

- single-file executable programs on the admitted source surface;
- narrow helper-module programs using direct local-path imports;
- records, native `quad`, and explicit `Option` / `Result` control flow;
- built-in `Sequence(T)` iteration;
- direct-record user-defined `Iterable` dispatch;
- verified execution through:

```text
source -> semantic analysis -> IR -> SemCode -> verifier -> VM
```

This is enough to demonstrate a real practical language pipeline. It is not a claim that every feature on `main` belongs to the published stable line.

---

## 7. What is benchmark-qualified on current `main`?

The application-completeness work on current `main` adds a wider, benchmark-qualified contour that has not yet been promoted into the stable or Gate-1 qualified promise:

- same-family `i32` arithmetic and comparisons;
- mutable locals and reassignment;
- `while`, `loop`, `break`, and `continue`;
- bounded `text`, concatenation, and explicit `to_text`;
- persistent `Sequence(T)` helper operations;
- functional `Map(K, V)` operations;
- deterministic seeded pseudo-random helpers;
- narrow capability-controlled `print(text)` observation;
- bounded project-root CLI routes.

The current text observation path is intentionally narrow:

```sm
fn main() {
    print("Hello, Semantic");
}
```

It must not be interpreted as unrestricted stdout, general formatting, file I/O, stdin, networking, or broad host authority.

---

## 8. Additional landed work that is not yet qualified

Current `main` also contains wider work that must remain visibly unpromoted until a later qualification or release decision. High-signal examples include:

- schema and boundary-core development;
- package-baseline work;
- ordered sequence and wider iterable surfaces;
- first-wave closures;
- first-wave generics;
- narrow runtime ownership for tuple and direct record-field paths;
- first-wave UI/application boundaries;
- additional module and import work beyond the currently qualified contour.

These areas may be implemented and tested without being part of the present release promise.

---

## 9. How execution is controlled

Semantic separates construction, admission, execution, and effects.

### Compiler boundary

Source code is parsed, checked, lowered, and emitted into SemCode. Source constructs do not execute directly.

### Verifier boundary

Persisted `.smc` artifacts must pass verifier admission before public execution.

Admission is responsible for rejecting malformed or unsupported artifacts, including problems such as:

- invalid opcode structure;
- bad control-flow targets;
- incompatible metadata;
- unsupported capability use;
- resource-budget violations;
- malformed function or section envelopes.

### Deterministic VM

VM execution is modeled as a deterministic state transition:

```text
state[k + 1] = step(state[k], instruction[pc])
```

Given the same admitted SemCode, runtime configuration, capability context, and input boundary, the result, trap class, and supported observable behavior should remain reproducible.

### Capability-controlled effects

The VM does not receive unrestricted host authority. Optional effects cross the PROMETHEUS integration boundary through explicit ABI, capability, gate, runtime, and audit contracts.

---

## 10. Architecture map

Semantic is a Rust workspace with narrow ownership boundaries.

### Language construction

```text
sm-front
sm-profile
sm-sema
sm-ir
sm-emit
```

Responsibilities:

- lexical and syntax analysis;
- source models and semantic checks;
- imports, symbols, and type policy;
- deterministic IR and lowering;
- SemCode emission.

### Execution

```text
sm-verify
sm-runtime-core
sm-vm
```

Responsibilities:

- artifact admission;
- runtime quotas and trap vocabulary;
- deterministic execution;
- SemCode inspection and disassembly.

### Tooling

```text
smc-cli
smc
svm
```

`smc` is the canonical user-facing toolchain entrypoint. `svm` is the lower-level VM-oriented entrypoint.

### PROMETHEUS integration

```text
prom-abi
prom-cap
prom-gates
prom-runtime
prom-state
prom-rules
prom-audit
```

Responsibilities:

- host-call vocabulary;
- capability decisions;
- controlled gates;
- runtime orchestration;
- semantic state and deterministic rule execution;
- audit and replay-oriented records.

### UI and applications

The repository contains UI, native rendering, and Workbench-related development. These surfaces are operator/application layers.

They may request operations and display results, but they must not become the owner of:

- source semantics;
- SemCode;
- verifier admission;
- VM execution;
- capability policy;
- runtime truth.

### Core capsule and laboratories

The workspace also contains low-level execution-core, quad substrate, benchmarking, and isolated laboratory crates. Their purpose is to qualify implementation substrates without creating a second public language or execution authority.

Read:

- [Architecture overview](../../ARCHITECTURE.md);
- [Architecture blueprint](../architecture/blueprint.md);
- [Module Ownership Map](../architecture/module_ownership_map.md);
- [Dependency and Boundary Rules](../architecture/dependency_boundary_rules.md).

---

## 11. CLI reference

| Command | Purpose |
|---|---|
| `smc check <file.sm|project-root>` | Parse and semantically check source. |
| `smc run <file.sm|project-root>` | Compile and execute from source. |
| `smc compile <input> -o app.smc` | Produce a SemCode artifact. |
| `smc verify app.smc` | Admit or reject an artifact without running it. |
| `smc run-smc app.smc` | Execute a persisted artifact through the verified route. |
| `smc disasm app.smc` | Inspect SemCode instructions. |
| `smc dump-ast <input>` | Inspect the parsed source model. |
| `smc dump-ir <input>` | Inspect lowered IR. |
| `smc lint <file.sm>` | Run lint-oriented checks. |
| `smc fmt <path>` | Format Semantic source. |
| `smc explain <code>` | Explain a diagnostic code. |
| `smc repl` | Start the interactive check-oriented REPL. |
| `smc 7hell <file.sm> [--json]` | Run the diagnostic/readiness qualification path. |
| `svm run app.smc` | Run SemCode through the VM entrypoint. |
| `svm disasm app.smc` | Disassemble SemCode through the VM entrypoint. |

See the [CLI Specification](../spec/cli.md) for the complete current command contract.

---

## 12. Project-root workflow

Current `main` supports a bounded project-root baseline using the repository's admitted manifest layouts.

From a supported project root:

```bash
smc check .
smc run .
smc compile . -o app.smc
```

When running from the Semantic repository without installing the binaries:

```bash
cargo run --bin smc -- check .
cargo run --bin smc -- run .
cargo run --bin smc -- compile . -o app.smc
```

This is not yet a complete package ecosystem. It does not claim:

- a public package registry;
- dependency solving;
- multi-package workspace management;
- a package manager;
- `smc new` scaffolding.

---

## 13. Canonical examples

Start with the curated programs under `examples/canonical/`.

| Example | Demonstrates |
|---|---|
| `rule_state_decision` | `quad`, records, `Result`, explicit decisions |
| `text_core` | bounded text, concatenation, `to_text`, controlled output |
| `loop_control_flow` | `while`, `loop`, `break`, `continue` |
| `collections_core` | practical collection operations |
| `option_result_control_flow` | explicit absence and failure paths |
| `cli_batch_core` | sequence-driven batch classification |
| `data_audit_record_iterable` | direct-record iteration and audit-style processing |

The benchmark pack also includes a deterministic headless Snake program:

```bash
cargo run --bin smc -- run examples/benchmarks/snake_core.sm
```

Use the [Examples Index](../examples_index.md) for the complete curated list, qualification reading, and intentional boundary example.

---

## 14. Runtime ownership status

The currently documented ownership slice is deliberately small and frozen.

Supported:

- tuple access paths;
- direct record-field access paths;
- frame-local borrow lifetime;
- rejection of overlapping active write paths;
- sibling writes when their paths do not overlap.

Not part of the current frozen ownership slice:

- ADT payload paths;
- schema paths;
- partial borrow release before frame exit;
- advanced alias or region reasoning;
- inter-frame borrow persistence;
- indirect projections;
- smart path normalization.

This is a deliberate engineering choice: a narrow verified contract is preferable to broad but ambiguous alias semantics.

Read the [Runtime Ownership Specification](../spec/runtime_ownership.md).

---

## 15. Compatibility perimeter

The repository intentionally retains a narrow historical and compatibility perimeter.

It is not a second owner of the Semantic language, SemCode, verifier, VM, or PROMETHEUS contracts. New architecture must land in the correct owner crate rather than in retained compatibility paths.

The exact inventory is maintained in the [Legacy Map](../legacy-map.md).

---

## 16. Current explicit limits

Do not infer support for the following from neighboring features:

- unrestricted stdout or general formatting;
- arbitrary file, stdin, process, or network I/O;
- broad host ABI access;
- a complete standard library;
- a public package registry or dependency solver;
- a frozen runtime ABI or binary ISA;
- full-workspace `no_std`;
- production-ready deployment;
- stable promotion of every feature on current `main`;
- UI-owned execution semantics.

Semantic should currently be read as:

```text
a deterministic verified execution platform
with native quad logic
and controlled expansion
```

—not as a language ecosystem that already promises every planned feature.

---

## 17. Where to go next

| Goal | Start here |
|---|---|
| Build and run Semantic | [Getting Started](../getting_started.md) |
| Browse working programs | [Examples Index](../examples_index.md) |
| Understand the language | [Language Overview](../LANGUAGE.md) |
| Learn quad syntax | [Semantic Quad Surface](../language/semantic_quad_surface.md) |
| Read the public contract | [Specification Index](../spec/index.md) |
| Understand architecture | [ARCHITECTURE.md](../../ARCHITECTURE.md) |
| Check feature maturity | [Feature Maturity Matrix](../status/feature_maturity_matrix.md) |
| Check release posture | [Semantic v1 Readiness](../roadmap/v1_readiness.md) |
| Understand status vocabulary | [Public Status Model](../roadmap/public_status_model.md) |
| Check `no_std` boundaries | [no_std Support Matrix](../NO_STD.md) |
| Review compatibility paths | [Legacy Map](../legacy-map.md) |

Recommended reading order for a new technical reader:

1. [README](../../README.md);
2. [Getting Started](../getting_started.md);
3. [Examples Index](../examples_index.md);
4. [Feature Maturity Matrix](../status/feature_maturity_matrix.md);
5. [Public Status Model](../roadmap/public_status_model.md);
6. [Specification Index](../spec/index.md);
7. [Architecture overview](../../ARCHITECTURE.md).

---

## 18. Engineering rule

Repository changes follow a strict scope discipline:

```text
one logical change
  -> one PR
  -> tests when behavior changes
  -> spec and docs synchronization when contracts change
  -> no silent widening of release claims
```

Tests are treated as contract evidence, not only as regression checks.

If a documentation or UI task starts requiring new language, verifier, VM, runtime, or capability behavior, it must leave the documentation/UI scope and move into the appropriate owner track.