---
name: semantic-verifier-runtime-guard
description: Domain guard for SemCode format, verifier admission, deterministic VM execution, runtime quotas, capability gates, and PROMETHEUS host/effect boundaries. Enforces verifier-first trusted route, Quad Logic determinism, effect isolation, and R3 verification routing.
---

# Semantic Verifier & Runtime Guard

Status: repository-native domain guard
Authority: subordinate to [`AGENTS.md`](../../../AGENTS.md), [`CONSTRAINTS.md`](../../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../../.harness/current.task.yaml)

---

## 1. Purpose & Domain Scope

This domain guard governs changes to:
- **`sm-format`**: SemCode binary format, opcode definitions, and decoding implementation;
- **`sm-verify`**: Verifier admission gate, structural validation, stack checking, and reject diagnostics;
- **`sm-vm`**: Deterministic execution engine, frame management, register operations, and runtime failure reporting;
- **`sm-runtime-core`**: Shared runtime vocabulary, resource quotas, and documented ownership transport;
- **`prom-*`**: PROMETHEUS capability gates (`prom-cap`), host ABI (`prom-abi`), gate bindings (`prom-gates`), runtime orchestration (`prom-runtime`), and audit logging (`prom-audit`).

---

## 2. Core Verifier & Runtime Invariants

### A. Verifier-First Trusted Route

1. **Admission Before Execution**: Canonical trusted execution passes through verifier admission to `VerifiedSemCode` and resolves `VerifiedEntrySemCode` where required before trusted VM execution.
2. **Compatibility Is Not Architectural Authority**: Supported byte-based `run_verified_semcode*` wrappers may perform admission, but do not redefine the canonical verified token/entry route.
3. **Raw Is Explicitly Non-Canonical**: `run_semcode*` and diagnostic/disassembly APIs may remain within their documented raw or diagnostic perimeter; they must not enter trusted, production, or capability-bearing routes.
4. **No Fail-Open**: Malformed, unsupported, or untrusted bytecode must be rejected with deterministic diagnostics. Best-effort execution of invalid bytecode is forbidden.

### B. Total VM Determinism

- Execution outcomes, register states, and runtime failure diagnostics must be bit-for-bit deterministic for identical inputs, verified SemCode, runtime configuration, capability context, and execution/resource budget.
- VM owns instruction execution, stack frames, register allocation, quota tracking, and safe runtime failure reporting.
- VM does **NOT** own SemCode binary format, verifier policy, capability policy semantics, host effect execution, or UI state.

### C. Quad Logic Invariant

Preserve the Quad Logic frame defined in [`docs/spec/quad_logic_frame_v1.md`](../../../docs/spec/quad_logic_frame_v1.md):
- Do not collapse `N` or `S` into boolean values silently.
- Do not erase conflict state.
- Conversions to `bool` must be explicit, localized, documented where public, and tested.

### D. Deterministic Core Isolation & Host Effects

1. **Deterministic Core**: `sm-front`, `sm-sema`, `sm-ir`, `sm-format`, `sm-emit`, `sm-verify`, `sm-runtime-core`, and `sm-vm` must not perform direct filesystem, network, process, or OS operations.
2. **Host Adapters**: Host-facing adapters (`smc-cli`, PROMETHEUS host bridges) may perform explicitly authorized host I/O, but must not own language semantics, verifier policy, or capability authority.

### E. Capability Gates & PROMETHEUS Effect Boundary

All external side effects must use the route defined by `CONSTRAINTS.md`:

```text
effect request
  -> capability check
  -> budget check
  -> gate policy
  -> audit decision
  -> execute / reject
  -> trace / record
```

- A missing or denied capability results in deterministic refusal.
- Capability bits must not be repurposed without compatibility review.
- External effects must produce deterministic audit/trace records owned by `prom-audit`.

### F. Resource Budgets & Quotas

- Execution must enforce declared and effective quotas, including VM execution steps, memory allocations, handle counts, effect calls, and audit record limits.
- Budget exhaustion must be reported deterministically as a safe runtime error.
- Quota modifications require limit and usage tests.

### G. Runtime Ownership Model

Preserve the documented ownership slice in [`docs/spec/runtime_ownership.md`](../../../docs/spec/runtime_ownership.md):
- `AccessPath` resolution and `Borrow`/`Write` events;
- structural `OWN0` admission before execution;
- `SEMCOD11` tuple and `SEMCOD12` direct record-field transport;
- frame-local borrow lifetimes and overlapping-write rejection.

Do not assume unsupported ownership forms without normative specification and test coverage.

---

## 3. R3 Critical Risk Routing

Tasks modifying SemCode format, verifier admission, VM execution semantics, capability gates, PROMETHEUS runtime, quotas, determinism, or Quad Logic are R3-Critical per [`docs/agents/WORKFLOW.md`](../../../docs/agents/WORKFLOW.md).

For R3 work, follow the current [`docs/agents/VERIFICATION.md`](../../../docs/agents/VERIFICATION.md) profile: all R2 requirements, `pwsh -File scripts/admission_guard.ps1 -FullPreflight`, `pwsh -File tools/7hell/run_ci.ps1`, and a fresh-context adversarial doubt review. Invoke security hardening when changing capability, sanitization, or effect boundaries.

A requirement to synchronize a normative specification does not grant authority to edit it. If the needed specification path is outside the active Harness, stop and obtain task-scoped authorization.

---

## 4. Stop Conditions

Stop execution and report a blocker immediately if:
- **Verifier Bypass Attempted**: A trusted, production, or capability-bearing route would execute unverified SemCode.
- **Fail-Open Behavior**: Malformed bytecode is accepted or processed through best-effort fallback.
- **Core Pollution**: Direct host I/O is introduced into a deterministic core crate.
- **Quad Collapse**: Code implicitly coerces `quad` to `bool` or erases conflict state.
- **Untracked Capability Shift**: Capability bits or gate policies change without the required contract authority and tests.
- **Unauthorized Specification Scope**: Required normative-spec work is outside the active Harness.
