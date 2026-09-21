<p align="center">
  <img src="assets/brand/semantic-logo.png" alt="Semantic Language" width="860">
</p>

# Semantic Language

<p align="center">
  <strong>A deterministic, verifier-first programming language and execution platform with native four-state logic.</strong>
</p>

<p align="center">
  <a href="docs/getting_started.md"><img src="https://img.shields.io/badge/Start-Quickstart-2563eb?style=for-the-badge" alt="Quickstart"></a>
  <a href="docs/spec/index.md"><img src="https://img.shields.io/badge/Read-Specification-7c3aed?style=for-the-badge" alt="Specification"></a>
  <a href="ARCHITECTURE.md"><img src="https://img.shields.io/badge/View-Architecture-0e7a72?style=for-the-badge" alt="Architecture"></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Logic-N%20%2F%20F%20%2F%20T%20%2F%20S-7c3aed?style=flat-square" alt="Quad logic N/F/T/S">
  <img src="https://img.shields.io/badge/Execution-Verifier_First-2563eb?style=flat-square" alt="Verifier-first execution">
  <img src="https://img.shields.io/badge/Runtime-Deterministic-16a34a?style=flat-square" alt="Deterministic runtime">
  <img src="https://img.shields.io/badge/Artifact-SemCode-f59e0b?style=flat-square" alt="SemCode">
  <img src="https://img.shields.io/github/license/skulmakov-oss/Semantic?style=flat-square" alt="License">
  <img src="https://img.shields.io/github/last-commit/skulmakov-oss/Semantic?style=flat-square" alt="Last commit">
</p>

Semantic compiles `.sm` source into versioned SemCode (`.smc`), admits that artifact through an explicit verifier boundary, and executes admitted code in a deterministic virtual machine.

```text
Semantic source (.sm)
        │
        ▼
frontend + semantic analysis
        │
        ▼
deterministic lowering / IR
        │
        ▼
SemCode (.smc)
        │
        ▼
verifier admission
        │
        ▼
deterministic VM
        │
        ▼
capability-controlled external boundary
```

Semantic is not a syntax experiment or a parser prototype. The complete source-to-artifact-to-execution path exists today and is exercised by the repository's compiler, verifier, VM, CLI, examples, tests, and qualification suites.

> [!IMPORTANT]
> Semantic is an active systems-language and verified-execution R&D project. Current `main` is broader than the currently qualified limited-release contour. Implemented behavior, qualified behavior, and published-stable behavior are deliberately treated as different states.

---

## Why Semantic?

Most programming languages reduce logical state to two values:

```text
true / false
```

Semantic has a native four-state domain:

| State | Encoding | Meaning |
|---|---|---|
| `N` | `00` | no sufficient evidence / unknown |
| `F` | `01` | evidence for false |
| `T` | `10` | evidence for true |
| `S` | `11` | evidence for both / conflict |

The distinction is structural, not cosmetic.

`quad` stores independent true and false evidence planes. The core algebra is deterministic:

- `join(a, b)` = bitwise OR
- `meet(a, b)` = bitwise AND
- `inverse(a)` = swap true and false planes

That means conflict survives instead of being silently overwritten.

For example:

```sm
fn main() {
    let yes: quad = T;
    let no: quad = F;

    let conflict: quad = yes || no;

    assert(conflict == S);
}
```

In ordinary Boolean reasoning, combining true and false often forces one value to win.

Semantic can preserve the fact that both pieces of evidence exist.

This makes the model useful for systems involving:

- reasoning over incomplete information;
- conflicting observations;
- rule and policy evaluation;
- multi-source evidence;
- deterministic state machines;
- safety and admission decisions;
- agent and tool orchestration;
- systems where uncertainty must remain explicit.

A `quad` is therefore not another spelling of `bool`.

---

## A Small Semantic Program

```sm
record Sensor {
    id: i32,
    state: quad,
    reading: i32,
}

fn evaluate(sensor: Sensor) -> quad {
    return if sensor.reading < 0 {
        N
    } else if sensor.reading >= 10 && sensor.reading <= 90 {
        T
    } else {
        F
    };
}

fn main() {
    let sensor: Sensor = Sensor {
        id: 101,
        state: N,
        reading: 45,
    };

    let evaluated: Sensor =
        sensor with { state: evaluate(sensor) };

    assert(evaluated.state == T);
}
```

This uses real current Semantic surface features:

- nominal records;
- typed functions;
- native `quad`;
- `i32`;
- expression-oriented `if`;
- record copy-with;
- deterministic assertions.

---

## Try It

### Requirements

- current Rust toolchain;
- Git;
- Windows, Linux, or macOS.

Clone and build:

```bash
git clone https://github.com/skulmakov-oss/Semantic.git
cd Semantic

cargo build --bin smc --bin svm
```

Run a canonical program:

```bash
cargo run --bin smc -- run \
  examples/canonical/rule_state_decision/src/main.sm
```

Or inspect the entire pipeline explicitly:

```bash
cargo run --bin smc -- check \
  examples/canonical/rule_state_decision/src/main.sm

cargo run --bin smc -- compile \
  examples/canonical/rule_state_decision/src/main.sm \
  -o decision.smc

cargo run --bin smc -- verify decision.smc

cargo run --bin smc -- run-smc decision.smc

cargo run --bin smc -- disasm decision.smc
```

Conceptually:

```text
check source
   ↓
compile SemCode
   ↓
verify artifact
   ↓
execute admitted artifact
   ↓
inspect VM instructions
```

No installed language SDK is required for repository development; the toolchain can be run directly through Cargo.

---

## Verifier-First Execution

Semantic deliberately separates four different responsibilities.

### 1. Source semantics

The frontend and semantic layers decide what source code means and whether it is admissible.

### 2. Artifact construction

Accepted source is lowered through deterministic IR and emitted as versioned SemCode.

### 3. Admission

Persisted `.smc` artifacts cross a dedicated verifier boundary before execution.

The verifier is not merely a helper hidden inside the VM.

It is an architectural boundary responsible for rejecting invalid executable artifacts.

### 4. Execution

`sm-vm` executes admitted SemCode under explicit runtime rules and quotas.

External authority is not implicitly granted to the VM.

Host-facing effects cross controlled PROMETHEUS boundaries.

```text
source
  │
  ▼
sm-front
  │
  ▼
sm-sema
  │
  ▼
sm-ir
  │
  ▼
sm-emit
  │
  ▼
SemCode
  │
  ▼
sm-verify
  │
  ▼
sm-vm
  │
  ▼
PROMETHEUS capability boundary
```

---

## What Works Today

Semantic currently contains a real executable programming surface, not just planned syntax.

Among the implemented and exercised areas are:

### Language

- functions and typed parameters;
- locals and mutable locals;
- `if / else`;
- `match`;
- `while` and `loop`;
- `break` and `continue`;
- records;
- immutable record copy-with;
- tuples;
- native `quad`;
- `bool`;
- `i32`;
- `u32`;
- `f64`;
- `fx`;
- bounded `text`;
- `Option(T)` and `Result(T, E)`;
- `Sequence(T)`;
- persistent `Map(K, V)` operations;
- imports and bounded project-root workflows;
- deterministic seeded pseudo-random helpers;
- assertions and bounded contract-oriented source forms.

Not every landed surface is part of the same release promise. See the maturity and readiness documents for the precise classification.

### Compiler and execution

- source parsing;
- semantic analysis;
- deterministic lowering;
- IR inspection;
- SemCode generation;
- SemCode verification;
- deterministic VM execution;
- disassembly;
- runtime quotas;
- diagnostic catalog and `smc explain`;
- deterministic hashes and inspection routes;
- contract and boundary tests.

### Toolchain

`smc` is the canonical user-facing command.

```text
smc check
smc run
smc compile
smc verify
smc run-smc
smc test

smc dump-ast
smc dump-ir
smc dump-bytecode
smc disasm

smc hash-ast
smc hash-ir
smc hash-smc

smc lint
smc fmt
smc explain
smc repl
smc 7hell
```

The exact CLI contract lives in:

[`docs/spec/cli.md`](docs/spec/cli.md)

---

## Semantic Hub

Semantic also contains a governed boundary for external computational tools.

The Semantic Hub is designed so external engines do not become alternate owners of language or execution semantics.

```text
Semantic / application logic
          │
          ▼
      Semantic Hub
          │
    admission boundary
          │
   ┌──────┴──────┐
   │ capability  │
   │ resource    │
   │ provenance  │
   │ audit       │
   └──────┬──────┘
          │
          ▼
    external tool
```

Current CLI surface includes:

```bash
smc hub tools
smc hub describe <tool-id>
smc hub invoke <tool-id> <operation-id> --input request.json
smc hub session --requests requests.ndjson
smc hub audit --request <request-id>
```

The current reference integration is:

TurboVec (`vector.turbovec`)

with a bounded typed request/reply path for vector index and search operations.

Hub invocation is capability-gated and resource-bounded. Tools do not receive an unrestricted bypass into Semantic execution.

See:

[`docs/architecture/semantic_hub_v0.md`](docs/architecture/semantic_hub_v0.md)

---

## PROMETHEUS Boundary

Semantic Core owns program construction and verified execution.

PROMETHEUS owns controlled interaction with the environment.

Current integration crates include:

| Layer | Owner |
|---|---|
| ABI | `prom-abi` |
| capabilities | `prom-cap` |
| gates | `prom-gates` |
| semantic state | `prom-state` |
| rules / agenda | `prom-rules` |
| orchestration | `prom-runtime` |
| audit / replay metadata | `prom-audit` |

The boundary exists to prevent external effects from becoming implicit language semantics.

The architectural rule is simple:

> Construction, admission, execution, and external authority are separate concerns.

---

## Rust-like Semantic and Logos

The repository currently contains two source profiles with deliberately different authority.

### Rust-like Semantic

The executable programming surface.

It owns the source path that can proceed through:

```text
source → IR → SemCode → verifier → VM
```

### Logos

A separate experimental declarative profile.

Logos may be parsed and projected for inspection, but it does not share the Rust-like SemCode execution authority.

The two profiles must not silently fall through into one another.

This boundary is intentional.

---

## Project-Root Workflow

Semantic supports a bounded project-root model in addition to individual `.sm` files.

Typical workflow:

```bash
smc check .
smc run .
smc compile . -o app.smc
smc verify .
smc test .
```

The current project/package model provides deterministic local discovery and provenance-oriented behavior.

It does not yet imply:

- a public package registry;
- remote dependency installation;
- a complete package solver;
- arbitrary multi-package workspace semantics;
- a mature package ecosystem.

See:

[`docs/spec/project_model_v0.md`](docs/spec/project_model_v0.md)

and:

[`docs/spec/package_baseline_v0.md`](docs/spec/package_baseline_v0.md)

---

## Architecture

Semantic is a Rust workspace with explicit ownership boundaries.

```text
Semantic/
├── crates/
│   ├── sm-*                 language, IR, SemCode, verifier, VM, CLI
│   ├── prom-*               capability-controlled integration boundary
│   └── semantic-core-*      low-level execution substrate
│
├── examples/                executable Semantic programs
├── docs/
│   ├── spec/                normative contracts
│   ├── architecture/        ownership and system design
│   ├── roadmap/             maturity and phase governance
│   └── core/                low-level algebra and substrate contracts
│
├── tests/                   integration and public-contract evidence
├── reports/                 qualification evidence
└── assets/                  repository assets
```

Core owners:

| Crate | Responsibility |
|---|---|
| `sm-profile` | parser/profile policy |
| `sm-front` | lexer, parser, AST, source typing |
| `sm-sema` | semantic analysis and diagnostics |
| `sm-ir` | lowering, deterministic IR and artifact contract ownership |
| `sm-emit` | SemCode producer facade |
| `sm-verify` | executable artifact admission |
| `sm-runtime-core` | shared runtime vocabulary and quotas |
| `sm-vm` | verified execution and disassembly |
| `smc-cli` | canonical CLI |

The fundamental repository rule is:

> One public concept has one owner.

The repository intentionally retains a narrow compatibility perimeter (`crates/ton618-core`, `src/bin/ton618_core.rs`, `ton618_legacy/`). These paths are not second owners of Semantic architecture; new language, execution, and integration work belongs in canonical `sm-*`, `semantic-core-*`, or `prom-*` owners.

See [`ARCHITECTURE.md`](ARCHITECTURE.md) and [`docs/architecture/blueprint.md`](docs/architecture/blueprint.md).

---

## Determinism and Contract Discipline

Semantic is developed around explicit contracts rather than accidental behavior.

Repository principles include:

- specification before widening
- deterministic behavior
- explicit ownership
- verifier-first execution
- fail-closed boundaries
- tests as contract evidence
- no silent contract mutation

A green test suite does not automatically promote a feature into the public stable contract.

Implementation, qualification, and release promotion are separate states.

---

## Current Status

Semantic uses four explicit status classes:

| Status | Meaning |
|---|---|
| Published stable | explicitly published and supported by validated release evidence |
| Qualified limited release | proven inside a bounded qualification contour |
| Landed on `main`, not yet promised | implemented but not promoted into the release promise |
| Out of scope | intentionally outside the current contour |

Current repository posture:

- there is no currently evidenced Published Stable feature line;
- a bounded practical programming contour is Qualified Limited Release;
- current `main` contains substantially more implementation than that qualified contour;
- those additional features remain landed on `main`, not yet promised until explicitly promoted;
- Semantic is not presented as production-ready or as a complete general-purpose ecosystem.

Current prerelease:

`v1.2.0-beta.1`

For exact status, do not infer from this README alone.

Read:

- [`docs/roadmap/v1_readiness.md`](docs/roadmap/v1_readiness.md)
- [`docs/roadmap/public_status_model.md`](docs/roadmap/public_status_model.md)
- [`docs/status/feature_maturity_matrix.md`](docs/status/feature_maturity_matrix.md)
- [`docs/roadmap/stable_foundation/semantic_stable_foundation_matrix.md`](docs/roadmap/stable_foundation/semantic_stable_foundation_matrix.md)

---

## Current Engineering Direction

The active program is the Semantic Stable Foundation.

Its purpose is not to rapidly add syntax.

Its purpose is to reconcile and harden the contracts already present across:

```text
source
→ diagnostics
→ semantic authority
→ IR
→ SemCode
→ verifier
→ VM
→ runtime boundaries
→ compatibility
→ qualification
```

Current work is focused on closing inconsistencies before later compatibility, migration, self-hosting, and wider ecosystem work build on top of them.

The roadmap is intentionally sequential: foundation first, widening later.

---

## Examples

Start with the curated programs in:

[`examples/canonical/`](examples/canonical/)

Useful entry points include:

| Example | Demonstrates |
|---|---|
| `rule_state_decision` | `quad`, records, `Result`, explicit decisions |
| `text_core` | bounded text and controlled output |
| `loop_control_flow` | imperative loop control |
| `collections_core` | collection operations |
| `option_result_control_flow` | explicit absence and failure |
| `cli_batch_core` | deterministic batch classification |

There is also a deterministic headless Snake benchmark:

```bash
cargo run --bin smc -- run examples/benchmarks/snake_core.sm
```

See:

[`docs/examples_index.md`](docs/examples_index.md)

---

## Explicit Non-Claims

Do not infer the following from adjacent implemented features.

Semantic does not currently claim:

- unrestricted host access;
- unrestricted stdout;
- arbitrary filesystem access;
- arbitrary stdin;
- arbitrary networking;
- process execution;
- a broad unfrozen host ABI;
- a complete standard library;
- a public package registry;
- a mature dependency solver;
- a frozen universal binary ISA;
- full-workspace `no_std`;
- production-ready deployment;
- stable status for everything landed on `main`.

Capabilities are widened deliberately rather than inherited implicitly.

---

## Documentation

For first-time readers:

| Goal | Document |
|---|---|
| Run Semantic | [Getting Started](docs/getting_started.md) |
| Language surface | [Syntax](docs/spec/syntax.md) |
| Native Quad algebra | [Quad Algebra](docs/core/quad_algebra.md) |
| Full public contracts | [Specification Index](docs/spec/index.md) |
| Architecture | [ARCHITECTURE.md](ARCHITECTURE.md) |
| Detailed architecture | [Blueprint](docs/architecture/blueprint.md) |
| Module ownership | [Module Ownership Map](docs/architecture/module_ownership_map.md) |
| CLI contract | [CLI Specification](docs/spec/cli.md) |
| Feature maturity | [Feature Maturity Matrix](docs/status/feature_maturity_matrix.md) |
| Release posture | [Semantic v1 Readiness](docs/roadmap/v1_readiness.md) |

For coding agents, repository-local `.agents/skills` should be treated as workflow guidance, while normative syntax and semantic truth remain grounded in `docs/spec/*`, executable examples, and tests.

Do not invent Semantic syntax from analogy with another language.

---

## Development

Basic qualification:

```bash
cargo fmt --check
cargo test --workspace
```

Representative contract checks:

```bash
cargo test --test public_api_contracts
cargo test --test canonical_examples
cargo test --test runtime_ownership_e2e
```

Repository development discipline:

```text
one logical change
        ↓
one focused PR
        ↓
behavioral tests
        ↓
owning spec sync when contracts change
        ↓
qualification
        ↓
review
```

Tests are evidence of a contract.

They are not permission to silently redefine one.

---

## Contributing

Contributions are welcome when they preserve the project's ownership and contract boundaries.

Before opening a change:

1. identify the owning layer;
2. keep the patch focused;
3. update the owning specification when public behavior changes;
4. add evidence for visible behavior;
5. avoid adding new architecture to compatibility paths;
6. distinguish implemented behavior from qualified or published behavior.

Architecture-sensitive contributors should read:

- [`docs/architecture/module_ownership_map.md`](docs/architecture/module_ownership_map.md)
- [`docs/architecture/dependency_boundary_rules.md`](docs/architecture/dependency_boundary_rules.md)
- [`docs/roadmap/public_status_model.md`](docs/roadmap/public_status_model.md)

---

## License

Semantic is licensed under the [Apache License 2.0](LICENSE).

Copyright 2026 Said Kulmakov.

Third-party dependencies and external assets remain under their respective licenses.

See [NOTICE](NOTICE) for attribution and project-scope notes.
