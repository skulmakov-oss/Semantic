---
name: semantic
description: Core Semantic router and subsystem architecture dispatcher. Routes tasks to specialized domain guards (source authoring, verifier/runtime, contract/release, UI boundary) and enforces high-level platform invariants.
---

# Semantic Domain Router & Subsystem Dispatcher

Status: repository-native domain router
Authority: subordinate to [`AGENTS.md`](../../../AGENTS.md), [`CONSTRAINTS.md`](../../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../../.harness/current.task.yaml)

---

## 1. Semantic Platform Identity

Semantic is a verifier-first, deterministic execution platform designed for reasoning logic, semantic state transitions, and verifiable AI-agent policies.

Semantic is:
- **NOT** a general-purpose scripting or application runtime.
- **NOT** an uncontrolled host execution engine.
- **NOT** the PROMETHEUS host capability runtime itself.
- **NOT** the UI/presentation layer.
- **NOT** ALM (Association Language Model / external models; no ALM mixing without dedicated integration contracts).

---

## 2. Subsystem Ownership Boundaries

Repository crate boundaries define clear ownership:

### A. Deterministic Core Libraries
- **`sm-front`**: Frontend lexer, parser, AST, source surface, and syntax diagnostics.
- **`sm-sema`**: Semantic analysis, type checking, exhaustiveness, and compile-time diagnostics.
- **`sm-ir`**: Intermediate Representation (IR), lowering passes, and AST-to-IR transformation.
- **`sm-format`**: SemCode binary format definitions, opcode tables, and decoding implementation (implementation owner; normative spec tracked in `docs/spec/*`).
- **`sm-emit`**: Producer-facing bytecode emission facade over the SemCode format.
- **`sm-verify`**: Verifier admission gate (structural, layout, stack, and bytecode validation).
- **`sm-runtime-core`**: Shared runtime vocabulary, common execution types, and quotas.
- **`sm-vm`**: Deterministic execution engine consuming verifier-admitted SemCode.

### B. Host-Facing Adapters & Platform Boundaries
- **`smc-cli`**: Canonical public CLI owner; performs authorized host I/O without owning language semantics or verifier policy.
- **`prom-*`**: PROMETHEUS host ABI, capability policy (`prom-cap`), gate descriptors (`prom-gates`), runtime sessions (`prom-runtime`), semantic state (`prom-state`), rules (`prom-rules`), and audit logging (`prom-audit`).
- **`prom-ui*`**: Platform-neutral UI orchestration (`prom-ui-runtime`), presentation models (`prom-ui`), native backend facade (`prom-ui-backend-native`), and operator tooling (`examples/workbench_semantic`).

---

## 3. Specialized Domain Guard Dispatch Table

Before proposing or executing changes, route the task to the appropriate specialized domain guard(s):

| Target Surface / Task Nature | Primary Domain Guard | Subsystem Responsibility |
|---|---|---|
| **`.sm` source, fixtures, examples, negative diagnostic probes** | [`semantic-source-authoring-guard`](../semantic-source-authoring-guard/SKILL.md) | Source evidence, fixture-first authoring, spec alignment, fail-closed on spec-vs-fixture drift |
| **SemCode, verifier, VM, runtime quotas, capabilities, PROMETHEUS effects** | [`semantic-verifier-runtime-guard`](../semantic-verifier-runtime-guard/SKILL.md) | Verifier admission, deterministic VM, trusted/raw perimeter, host-effect separation, R3 routing |
| **Public API/ABI, specs, serialization, release claims, compatibility** | [`semantic-contract-release-guard`](../semantic-contract-release-guard/SKILL.md) | Contract synchronization, status layers, release honesty |
| **UI presentation, renderer, interaction semantics, trace projection** | [`semantic-ui-boundary-guard`](../semantic-ui-boundary-guard/SKILL.md) | Presentation models, anti-authority boundaries, local versus effectful interaction |

### Cross-Domain Tasks

If a task touches multiple surfaces, activate all relevant domain guards. Never force a cross-boundary task into a single guard.

---

## 4. Universal Semantic Invariants

All tasks operating within the Semantic repository must respect these universal invariants:

1. **Total Determinism**: Compilation and execution must be reproducible for identical inputs, configurations, capability contexts, and execution budgets.
2. **Quad Logic Integrity**: Preserve Quad Logic according to [`docs/spec/quad_logic_frame_v1.md`](../../../docs/spec/quad_logic_frame_v1.md). Do not collapse `N` or `S` into boolean values or erase conflict state.
3. **Verifier-First Trusted Route**: Canonical trusted, production, and capability-bearing execution uses verifier-admitted SemCode and the verified token/entry path. Supported byte-based verified wrappers are compatibility surfaces; explicitly raw or diagnostic APIs remain outside the trusted route.
4. **Deterministic Core Isolation**: `sm-front`, `sm-sema`, `sm-ir`, `sm-format`, `sm-emit`, `sm-verify`, `sm-runtime-core`, and `sm-vm` must not perform direct filesystem, network, process, or OS side effects. Authorized host I/O belongs to host-facing adapters and PROMETHEUS boundaries.
5. **Release & Contract Honesty**: Code merged to `main` does not equal stable or released. Never retroactively widen published release claims.

---

## 5. Domain Stop Conditions

Stop and report a blocker immediately if:
- **Contract Drift**: Normative specifications (`docs/spec/*`) and executable implementation/fixtures conflict.
- **Ownership Conflict**: A task attempts to move core compiler/verifier/VM responsibilities into UI, CLI, or host layers.
- **Ambiguous Authority**: The active Harness envelope does not authorize the required files for the assigned task.
- **Unverified Fallback**: A mandatory tool (for example, Codebase Memory MCP) is unavailable. Follow the fail-closed protocol in `CONSTRAINTS.md`.
