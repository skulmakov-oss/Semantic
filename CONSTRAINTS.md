# CONSTRAINTS.md

Status: normative repository invariant authority
Scope: non-negotiable architectural and operational invariants for `skulmakov-oss/Semantic`

This document defines the hard invariants that govern every component, tool, and contributor in the Semantic repository. These constraints are unconditional: they cannot be bypassed, downgraded, or overridden by convenience, tooling, subagents, or roadmap speed.

---

## 1. Authority Precedence & Conflict Resolution

Operational authority in Semantic follows a strict hierarchy:

```text
1. Platform / Safety Constraints
   ↓
2. AGENTS.md (Canonical Bootstrap & Router)
   ↓
3. CONSTRAINTS.md (Repository Invariants)
   ↓
4. .harness/current.task.yaml (Task Authorization Envelope)
   ↓
5. Relevant Semantic Skill / Issue Specification
   ↓
6. Agent Implementation Plan
```

- **Strictness Rule**: A lower layer may make rules stricter, but may never loosen or waive an upper-layer rule.
- **Effective Authority Formula**:
  $$\text{Effective Authority} = \text{Repository Invariants} \cap \text{Task Authority}$$
- **Blocked-by-Constraint Protocol**: If an instruction or situation conflicts with a constraint:
  1. **STOP immediately**.
  2. **Report**: Name the constraint, the blocker, the observed evidence, and the minimum repository owner decision required.
  3. **NEVER bypass** or improvise a workaround.

---

## 2. Hard Invariants

### A. Verifier-First Execution Pipeline
```text
source
  -> frontend (sm-front)
  -> semantic analysis (sm-sema)
  -> IR and lowering (sm-ir)
  -> emission (sm-emit over sm-format)
  -> SemCode binary format (sm-format)
  -> verifier admission (sm-verify)
  -> deterministic execution (sm-vm)
  -> PROMETHEUS capability and effect boundary
```
- **NO Verifier Bypass**: Every public execution path must run through `sm-verify`.
- **NO Unchecked SemCode Execution**: No runtime route may execute unverified SemCode bytecode.
- **Verifier Is an Admission Gate**: `sm-verify` checks structural, layout, quota, and bytecode constraints. It does not execute runtime policy, does not replace the VM, and does not parse source.
- **VM Consumes Verified SemCode**: `sm-vm` executes verifier-admitted SemCode deterministically and rejects malformed bytecode distinctly from runtime faults.

### B. Determinism & Total Representation
- **NO Nondeterminism in Core**: Given identical input, configuration, capability context, and execution budget, compilation and execution in deterministic core libraries must produce byte-for-byte and trace-for-trace deterministic outcomes.
- **Quad Logic Invariant**: Quad Logic (`quad`) is a native 4-valued domain:
  - `N` = Unknown (`00`)
  - `F` = False (`01`)
  - `T` = True (`10`)
  - `S` = Conflict (`11`)
- **NO Quad Collapse**: Quad states must never be implicitly collapsed into `bool` (`N` is not `false`; `S` is not `true` or `false`).
- **NO Conflict Erasure**: Conflict (`S`) and Unknown (`N`) states must remain visible across compiler, VM, diagnostics, and UI projections.
- **Distinction of Roles**: `bool` decides control flow; `quad` represents four-state reasoning truth. Conversions between `bool` and `quad` must be explicit, documented, and tested.

### C. Architectural Boundaries & Ownership
- **Deterministic Semantic Core Libraries (`sm-front`, `sm-sema`, `sm-ir`, `sm-format`, `sm-emit`, `sm-verify`, `sm-runtime-core`, `sm-vm`)**:
  - `sm-front`: parsing, AST, lexer, syntax.
  - `sm-sema`: semantic analysis, type checking, compile-time diagnostics.
  - `sm-ir`: Intermediate Representation data structures and lowering passes.
  - `sm-format`: canonical SemCode binary format, opcode definitions, and decoding contracts.
  - `sm-emit`: emission facade over `sm-format`.
  - `sm-verify`: admission gate (structure, layout, and bytecode verification).
  - `sm-runtime-core`: shared runtime vocabulary, errors, and quotas.
  - `sm-vm`: deterministic verified SemCode execution engine.
- **Host-Facing Adapters & CLI (`smc-cli`, platform adapters)**:
  - Owns CLI entrypoints, argument parsing, file reading/writing of source and compiled artifacts, process exit codes, and platform event bridges.
  - May perform explicitly authorized host I/O, but must **never** become owners of language semantics, verifier admission rules, capability policy, or deterministic core execution.
- **PROMETHEUS Boundary (`prom-abi`, `prom-cap`, `prom-gates`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-audit`)**: Owns host ABI, capability checks, gate descriptors, runtime sessions, state transitions, deterministic rule agendas, and audit/replay logging.
- **Semantic UI (`prom-ui*`, `prom-ui-runtime`, `prom-ui-backend-native`)**: Owns UI model vocabulary, platform-neutral UI orchestration, and native backend facades. UI is an admitted presentation surface; UI is never compiler, verifier, VM, capability policy authority, or audit authority.
- **Developer / Operator Tooling (Workbench / Studio)**: Tooling surfaces over admitted contracts; never architectural owners of compiler, verifier, VM, or language semantics.
- **External Integrations (ALM / Andromeda)**: Independent projects that must not be mixed into Semantic core without a formal integration contract and capability boundary.

### D. Capability Gating & Host Effects
- **NO Direct External Effects in Semantic Core**: The deterministic core libraries contain zero direct filesystem, network, process, or OS effects, zero hidden telemetry, and zero unaudited side channels.
- **Authorized Host Operations in Adapters**: Host-facing adapters (`smc-cli`) legitimately perform explicit host operations (reading files, writing output artifacts, standard I/O) as CLI tooling, but must not smuggle unaudited effects into core libraries or bypass capability checks when runtime effects are invoked.
- **Required External Effect Route**:
  $$\text{effect request} \rightarrow \text{capability check} \rightarrow \text{budget check} \rightarrow \text{gate policy} \rightarrow \text{audit decision} \rightarrow \text{execute/reject} \rightarrow \text{trace/record}$$
- **Explicit Capability Checks**: Missing capability strictly means no effect. Capability denials must be observable, testable, and safely reported.
- **Mandatory Auditability**: Every external runtime effect must be traceable to a deterministic audit record. `prom-audit` owns audit and replay record contracts.

### E. Fail-Closed Posture & Anti-Downgrade
- **NO Fail-Open Admission**: The system must fail closed on invalid inputs, quota exhaustion, capability denial, missing state, or communication faults.
- **NO Silent Fallback or Semantic Downgrade**: Unsupported versions, malformed bytecode, or missing features must fail with explicit errors.
- **NO Hidden Compatibility Shims**: All compatibility paths must be explicitly declared, strictly bounded, and tested.

### F. Testing, CI & Implementation Discipline
- **NO Weakening, Deleting, or Skipping Tests**: Never delete, weaken, ignore, or skip tests or assertions to achieve a passing CI run.
- **NO TODO/Stub Represented as Implementation**: Incomplete work must be explicitly reported as unadmitted/unimplemented, not masked.
- **NO Invented Source Syntax**: Language syntax must strictly conform to existing specification contracts (`docs/spec/*`).
- **NO Unauthorized Dependency or Workflow Changes**: No modifying dependencies in `Cargo.toml` or CI pipelines in `.github/workflows/*` without explicit issue authorization.
- **NO Completion Claim Without Fresh Evidence**: Exact commands, exit codes, and output logs must be recorded before asserting task completion.

### G. Public Contract & Status Integrity
- **Contract Authority**: Public contracts are defined strictly by `docs/spec/*`.
- **Status Vocabulary**: Public status terms are governed strictly by `docs/roadmap/public_status_model.md`.
- **Era Sequencing Authority**: Implementation-era sequencing is governed by `docs/architecture/bootstrap_transition.md`.
- **Landed on Main != Stable**: Code landed on the `main` branch is not automatically stable or release-promised.
- **NO Release Claim Widening**: Release claims, performance metrics, and maturity levels must reflect verified evidence, not forward-looking intentions.

### H. Legacy Perimeter Confinement
- **Explicit Legacy Perimeter**: Legacy compatibility names, types, and shims are strictly confined to the explicit allowlists enforced by `tests/legacy_guards.rs`.
- **NO Legacy Proliferation**: No new legacy shims, paths, or naming patterns may be introduced into the root or core crates.

---

## 3. Risk Classification Model

Every change in the repository must be classified by its architectural risk level:

| Level | Classification | Scope | Required Scrutiny & Verification |
|---|---|---|---|
| **R0** | Trivial / Informational | Comments, typo fixes, non-normative documentation | Formatting check, git diff check |
| **R1** | Private / Isolated | Internal crate refactoring, private module bugfixes | PR-ready check (`scripts/admission_guard.ps1 -PRReady`), unit tests |
| **R2** | Boundary / Contract | Parser, public API signatures, type rules, IR, serialization format, cross-crate contracts | CI parity gate (`-CIParity`), public API contract tests (`public_api_contracts`), golden test fixtures |
| **R3** | Critical / Systemic | Verifier admission, SemCode binary format, VM execution, capability gates, PROMETHEUS runtime, determinism, cryptographic/security features, release compatibility | Fresh-context adversarial doubt review, full preflight check (`-FullPreflight`), comprehensive regression suite, release bundle verification |
