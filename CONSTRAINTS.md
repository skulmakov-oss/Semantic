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
4. Explicit Repository-Owner Governance Authorization (only for governance-envelope transitions)
   ↓
5. .harness/current.task.yaml (Normal Task Authorization Envelope)
   ↓
6. Normative Specs / Issue Authority / Relevant Semantic Skill
   ↓
7. Agent Implementation Plan
```

- **Strictness Rule**: A lower layer may make rules stricter, but may never loosen or waive an upper-layer rule.
- **Effective Authority Formula**:
  $$\text{Effective Authority} = \text{Repository Invariants} \cap \text{Task Authority}$$
- **Blocked-by-Constraint Protocol**: If an instruction or situation conflicts with a constraint:
  1. **STOP immediately**.
  2. **Report**: Name the constraint, the blocker, the observed evidence, and the minimum repository owner decision required.
  3. **NEVER bypass** or improvise a workaround.
- **No Autonomous Fallback**: The absence of an existing compliant solution is not permission to weaken a constraint or autonomously fall back. If a required capability, tool, evidence source, or architectural dependency is unavailable, an agent must STOP immediately, report the exact blocker and what cannot be proven, and await explicit repository-owner decision. Only the repository owner may authorize a task-scoped, visible, temporary fallback.

### Governance-Maintenance Transition

The active Harness envelope governs normal repository work. An agent may **not** broaden, replace, or reinterpret that envelope merely because the current task would otherwise be blocked.

If the task itself must change `AGENTS.md`, `CONSTRAINTS.md`, `.harness/current.task.yaml`, or the repository agent-governance layer and the active envelope does not authorize those paths:

1. require explicit repository-owner authorization for a named governance task (recorded in the issue/PR or equivalent durable task evidence);
2. use a dedicated governance branch and a narrowly scoped temporary governance envelope for the authorized paths;
3. preserve the authorization/envelope transition in auditable branch/PR history;
4. if the governance task is out-of-band from the active development track, restore the tracked main-development envelope before merge;
5. never treat the ability to edit the Harness as permission to self-authorize unrelated work.

This is a controlled envelope transition, not a waiver of repository invariants.

---

## 2. Hard Invariants

### A. Verifier-First Trusted Execution Pipeline

Canonical/trusted execution follows:

```text
source
  -> frontend (sm-front)
  -> semantic analysis (sm-sema)
  -> IR and lowering (sm-ir)
  -> emission (sm-emit over sm-format)
  -> SemCode binary format (sm-format)
  -> verifier admission / verified token (sm-verify)
  -> deterministic execution (sm-vm)
  -> PROMETHEUS capability and effect boundary
```

- **NO Verifier Bypass on Canonical/Trusted Routes**: Production runtime paths, user-facing trusted execution commands, and capability-bearing execution must use verifier-admitted SemCode / the canonical verified-token path.
- **Raw / Diagnostic Perimeter Is Explicitly Non-Canonical**: Intentionally raw APIs such as documented `run_semcode*` / diagnostic analysis surfaces may execute unverified SemCode only when they remain explicitly classified as raw/diagnostic or compatibility/testing surfaces. They must never be used to bypass admission on production/trusted execution routes or presented as verifier-admitted execution.
- **Compatibility Shims Are Not Canonical Authority**: Supported byte-based compatibility wrappers may remain where repository contracts require them, but they do not redefine the canonical verified execution policy.
- **Verifier Is an Admission Gate**: `sm-verify` checks structural, layout, quota, and bytecode constraints. It does not execute runtime policy, does not replace the VM, and does not parse source.
- **VM Trusted Execution Consumes Verified SemCode**: Canonical VM execution consumes verifier-admitted SemCode deterministically and distinguishes verifier rejection from runtime faults.

The documented raw/diagnostic perimeter must remain narrow, explicit, tested, and unable to drift into trusted production execution.

### B. Determinism & Total Representation
- **NO Nondeterminism in Core**: Given identical input, configuration, capability context, and execution budget, compilation and execution in deterministic core libraries must produce byte-for-byte and trace-for-trace deterministic outcomes.
- **Quad Logic Invariant**: Quad Logic (`quad`) is a native 4-valued domain strictly adhering to normative specification [`docs/spec/quad_logic_frame_v1.md`](docs/spec/quad_logic_frame_v1.md):
  - `N` = Null (`00`)
  - `F` = Strict False (`01`)
  - `T` = Strict True (`10`)
  - `S` = Conflict / Super (`11`)
- **NO Quad Collapse**: Quad states must never be implicitly collapsed into `bool` (`N` is not `false`; `S` is not `true` or `false`).
- **NO Conflict Erasure**: Conflict (`S`) and Null (`N`) states must remain visible across compiler, VM, diagnostics, and UI projections.
- **Distinction of Roles**: `bool` decides control flow; `quad` represents four-state reasoning truth. Conversions between `bool` and `quad` must be explicit, documented, and tested.

### C. Architectural Boundaries & Ownership
- **Deterministic Semantic Core Libraries (`sm-front`, `sm-sema`, `sm-ir`, `sm-format`, `sm-emit`, `sm-verify`, `sm-runtime-core`, `sm-vm`)**:
  - `sm-front`: parsing, AST, lexer, syntax.
  - `sm-sema`: semantic analysis, type checking, compile-time diagnostics.
  - `sm-ir`: Intermediate Representation data structures, lowering passes, and optimizer logic (retains baseline format ownership in historical `docs/spec/*` contracts).
  - `sm-format`: crate containing SemCode binary format definitions, opcode tables, and decoding implementation.
  - `sm-emit`: emission facade over `sm-format`.
  - `sm-verify`: admission gate (structure, layout, and bytecode verification).
  - `sm-runtime-core`: shared runtime vocabulary, errors, and quotas.
  - `sm-vm`: deterministic execution engine, with canonical trusted execution verifier-admitted and explicitly documented raw/diagnostic APIs kept outside the trusted route.
  - *Ownership Authority Synchronization*: `docs/spec/*` remains normative contract truth. The formal synchronization of spec documents and skills from `sm-ir` format ownership to `sm-format` is tracked as explicit follow-up work for #1846; ordinary SemCode implementation work is not blocked while governance and historical documentation undergo this planned convergence.
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
- **NO Fail-Open Admission**: The system must fail closed on invalid inputs, quota exhaustion, capability denial, missing state, or communication faults on routes where admission/policy applies.
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
