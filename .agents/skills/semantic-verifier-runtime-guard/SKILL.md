---
name: semantic-verifier-runtime-guard
description: Domain guard for SemCode format, verifier admission, deterministic VM execution, runtime quotas, capability gates, and PROMETHEUS host/effect boundaries. Enforces verifier-first trusted route, Quad Logic determinism, effect isolation, and R3 verification routing.
---

# Semantic Verifier & Runtime Guard

Status: repository-native domain guard
Authority: subordinate to [`AGENTS.md`](../../AGENTS.md), [`CONSTRAINTS.md`](../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../.harness/current.task.yaml)

---

## 1. Purpose & Domain Scope

This domain guard governs changes to:
- **`sm-format`**: SemCode binary format, opcode definitions, and decoding implementation;
- **`sm-verify`**: Verifier admission gate, structural validation, stack checking, and reject diagnostics;
- **`sm-vm`**: Deterministic execution engine, frame management, register operations, and runtime failure reporting;
- **`sm-runtime-core`**: Shared runtime vocabulary, common types, resource quotas, and ownership lattices;
- **`prom-*`**: PROMETHEUS capability gates (`prom-cap`), host ABI (`prom-abi`), gate bindings (`prom-gates`), runtime orchestration (`prom-runtime`), and audit logging (`prom-audit`).

---

## 2. Core Verifier & Runtime Invariants

### A. Verifier-First Trusted Route
1. **Admission Before Execution**: Canonical/trusted execution consumes strictly verifier-admitted SemCode:
   $$\text{Emit SemCode} \longrightarrow \text{verify\_semcode} \longrightarrow \text{run\_verified\_semcode*}$$
2. **No Verifier Bypass**: No production execution path may execute unverified SemCode or bypass `sm-verify`.
3. **No Fail-Open**: Malformed, unsupported, or untrusted bytecode must be rejected with deterministic diagnostics. Best-effort execution of invalid bytecode is strictly forbidden.
4. **Trusted vs. Raw Boundary**:
   - Canonical execution requires verifier admission.
   - Explicitly documented raw/diagnostic APIs (`run_semcode*`-style diagnostic entry points) may remain unverified only within their narrow diagnostic/compatibility perimeter.
   - Raw/diagnostic APIs must never drift into canonical trusted execution routes.
   - Do not demand removal of intentionally raw diagnostic/compatibility APIs that are properly documented and isolated.

### B. Total VM Determinism
- Execution outcomes, register states, and runtime failure diagnostics must be bit-for-bit deterministic for identical inputs, verified SemCode, runtime configuration, and capability context.
- VM owns: instruction execution, stack frames, register allocation, quota tracking, and safe runtime failure reporting.
- VM does **NOT** own: SemCode binary format, verifier policy, capability policy semantics, host effect execution, or UI state.

### C. Quad Logic Invariant
Preserve the Quad Logic frame (`N` = Null, `F` = Strict False, `T` = Strict True, `S` = Conflict / Super per [`docs/spec/quad_logic_frame_v1.md`](../../docs/spec/quad_logic_frame_v1.md)):
- Packed encoding: `N = 00`, `F = 01`, `T = 10`, `S = 11`.
- Never collapse `N` (Null) or `S` (Conflict) into boolean values silently.
- Never treat unknown/Null as false; never erase conflict state.
- Conversions to `bool` must be explicit, localized, documented where public, and tested.

### D. Deterministic Core Isolation & Host Effects
1. **Deterministic Core**: `sm-front`, `sm-sema`, `sm-ir`, `sm-format`, `sm-emit`, `sm-verify`, `sm-runtime-core`, and `sm-vm` must **never** perform direct filesystem, network, process, or OS operations.
2. **Host Adapters**: Host-facing adapters (`smc-cli`, PROMETHEUS host bridges) may perform explicitly authorized host I/O, but must not own language semantics, verifier policy, or capability authority.

### E. Capability Gates & PROMETHEUS Effect Boundary
All external side effects must traverse the authorized PROMETHEUS pipeline:
$$\text{Effect Request} \longrightarrow \text{Capability Check} \longrightarrow \text{Budget Check} \longrightarrow \text{Gate Evaluation} \longrightarrow \text{Audit Log} \longrightarrow \text{Execute / Reject}$$
- Missing or denied capability results in immediate deterministic refusal.
- Capability bits must never be repurposed without compatibility review.
- External side effects must produce deterministic audit/trace records (`prom-audit`).

### F. Resource Budgets & Quotas
- Execution must enforce declared and effective quotas: VM execution steps, memory allocations, handle counts, effect calls, and audit record limits.
- Budget exhaustion must be reported deterministically as a safe runtime error.
- Quota modifications require limit and usage unit tests.

### G. Runtime Ownership Model
Preserve the active runtime ownership model:
- `AccessPath` resolution, `Borrow` / `Write` lifecycle events, `OWN0`, `SEMCOD11` tuple ownership, `SEMCOD12` direct record-field ownership, frame-local borrow lifetimes, and overlapping write rejection.
- Advanced features (ADT payload paths, inter-frame borrow persistence, arbitrary aliasing) must not be assumed without formal specification and test coverage.

---

## 3. R3 Critical Risk Routing

Any task that modifies:
- SemCode binary format or opcode definitions (`sm-format`);
- Verifier admission logic or structural rules (`sm-verify`);
- VM execution semantics, memory, or registers (`sm-vm`);
- Capability checks, gate descriptors, or effect dispatch (`prom-*`);
- Quotas, determinism, or Quad Logic semantics;

is classified as **R3-Critical** per [`docs/agents/WORKFLOW.md`](../../docs/agents/WORKFLOW.md).

### Mandatory R3 Requirements:
1. **Adversarial Doubt Review**: Perform a dedicated doubt-driven review challenging assumptions, probing failure modes, and checking edge cases.
2. **Security & Hardening**: Invoke `security-and-hardening` when touching capability, sanitization, or effect boundaries.
3. **Comprehensive Verification**: Execute full preflight verification (`pwsh -File scripts/admission_guard.ps1 -FullPreflight` / `-CIParity`) including positive admission and negative rejection suites.
4. **Specification Synchronization**: Synchronize corresponding specifications (`docs/spec/verifier.md`, `docs/spec/quad_logic_frame_v1.md`, etc.).

---

## 4. Stop Conditions

Stop execution and report a blocker immediately if:
- **Verifier Bypass Attempted**: Code introduces a route that executes unverified SemCode on a canonical production path.
- **Fail-Open Behavior**: Malformed bytecode is accepted or processed via best-effort fallbacks.
- **Core Pollution**: Direct host I/O (file, network, env) is introduced into core crates.
- **Quad Collapse**: Code implicitly coerces `quad` to `bool` or erases conflict state.
- **Untracked Capability Shift**: Capability bits or gate policies are modified without spec updates and compatibility tests.
