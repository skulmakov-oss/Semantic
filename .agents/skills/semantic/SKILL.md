---
name: semantic
description: Core Semantic router and subsystem architecture dispatcher. Routes tasks to specialized domain guards (source authoring, verifier/runtime, contract/release, UI boundary) and enforces high-level platform invariants.
---

# Semantic Domain Router & Subsystem Dispatcher

Status: repository-native domain router
Authority: subordinate to [`AGENTS.md`](../../AGENTS.md), [`CONSTRAINTS.md`](../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../.harness/current.task.yaml)

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
| **`.sm` source, fixtures, examples, negative diagnostic probes** | [`semantic-source-authoring-guard`](../semantic-source-authoring-guard/SKILL.md) | Syntax/type cribsheets, fixture-first authoring, spec alignment, fail-closed on spec-vs-fixture drift |
| **SemCode, verifier, VM, runtime quotas, capabilities, PROMETHEUS effects** | [`semantic-verifier-runtime-guard`](../semantic-verifier-runtime-guard/SKILL.md) | Verifier admission gate, deterministic VM, raw vs trusted paths, host-effect separation, R3 routing |
| **Public API/ABI, specs, serialization, release claims, compatibility** | [`semantic-contract-release-guard`](../semantic-contract-release-guard/SKILL.md) | Contract synchronization, stable vs main distinction, forward-only widening, release honesty |
| **UI presentation, renderer, interaction semantics, trace projection** | [`semantic-ui-boundary-guard`](../semantic-ui-boundary-guard/SKILL.md) | Presentation models, anti-compiler boundaries, event-to-effect pipeline, visual projection limits |

### Cross-Domain Tasks
If a task touches multiple surfaces (e.g., adding a language construct with `.sm` fixtures, compiler lowering, and verifier admission rules), **activate all relevant domain guards**. Never force a cross-boundary task into a single guard.

---

## 4. Universal Semantic Invariants

All tasks operating within the Semantic repository must respect these universal invariants:

1. **Total Determinism**: Execution, reject diagnostics, and state transitions must be strictly reproducible for identical inputs, capability contexts, and configurations.
2. **Quad Logic Integrity**: Preserve Quad Logic (`N` = Null, `F` = Strict False, `T` = Strict True, `S` = Conflict / Super per [`docs/spec/quad_logic_frame_v1.md`](../../docs/spec/quad_logic_frame_v1.md)). Never collapse `N` or `S` into boolean values silently; never erase conflict state; never treat unknown as false.
3. **Verifier-First Trusted Route**: Canonical execution consumes verifier-admitted SemCode (`emit -> verify -> run_verified_semcode*`). No public route may execute unadmitted bytecode.
4. **Deterministic Core Isolation**: Core libraries (`sm-front`, `sm-sema`, `sm-ir`, `sm-format`, `sm-emit`, `sm-verify`, `sm-runtime-core`, `sm-vm`) must never perform direct filesystem, network, process, or OS side effects. Host I/O is restricted to authorized host-facing adapters (`smc-cli`, PROMETHEUS).
5. **Release & Contract Honesty**: Code merged to `main` does not equal stable or released. Never retroactively widen published release claims.

---

## 5. Domain Stop Conditions

Stop and report a blocker immediately if:
- **Contract Drift**: Normative specifications (`docs/spec/*`) and executable implementation/fixtures conflict.
- **Ownership Conflict**: A task attempts to move core compiler/verifier/VM responsibilities into UI, CLI, or host layers.
- **Ambiguous Authority**: The active Harness envelope does not authorize the required files for the assigned task.
- **Unverified Fallback**: A mandatory tool (e.g., Codebase Memory MCP) is unavailable. Follow the fail-closed protocol in `CONSTRAINTS.md`.
