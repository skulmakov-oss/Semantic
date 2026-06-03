# Scientific Value of Semantic

Status: research orientation document.

This document describes the scientific and research value of Semantic without widening the public implementation contract. Public behavior claims remain governed by `docs/spec/*`, `docs/roadmap/public_status_model.md`, and corresponding tests.

## 1. Core Thesis

Semantic investigates how reasoning-oriented programs can be represented, verified, admitted, bounded, executed, and audited under deterministic semantic rules.

The central research thesis is:

```text
reasoning programs should not merely run;
they should be represented, verified, admitted, bounded, executed,
and audited under explicit semantic, capability, and effect boundaries.
```

This shifts the main question from ordinary program execution:

```text
How do we execute code?
```

to a stricter execution-admission question:

```text
When is a reasoning program allowed to execute?
```

## 2. Problem Statement

Modern programming languages and runtimes are usually optimized for general computation. They can express logic, state transitions, effects, and control flow, but they do not usually make uncertainty, conflict, capability boundaries, and verifier-first admission central to the execution model.

For reasoning systems, AI-agent policies, semantic state transitions, and controlled host effects, this creates several problems:

- source code can be parsed or compiled before it is semantically safe to run;
- uncertainty and conflict are often represented indirectly through ad-hoc flags, exceptions, optional values, or external policy layers;
- runtime effects can become difficult to audit or replay;
- nondeterministic execution paths can weaken reproducibility;
- capability checks can be scattered across host code instead of being part of a disciplined execution boundary;
- UI or orchestration layers can accidentally become hidden sources of execution semantics.

Semantic addresses these problems by making the execution path explicit:

```text
source
  -> semantic analysis
  -> IR
  -> SemCode
  -> verifier admission
  -> deterministic VM
  -> controlled boundary
  -> audit / observation surface
```

## 3. Research Hypothesis

Semantic is based on the following working hypothesis:

```text
A reasoning-oriented execution platform becomes more controllable,
reproducible, and auditable when source programs are lowered into a
verifiable artifact boundary, admitted before execution, executed by a
deterministic VM, and restricted by explicit capability/effect boundaries.
```

This is not a claim that Semantic already solves all safety, verification, or AI-agent control problems. It is a research hypothesis that can be tested through implementation, negative tests, reproducibility tests, capability-denial tests, and runtime-contract experiments.

## 4. Scientific Novelty Candidates

The following areas are candidates for scientific or technical novelty. Each one should be treated as a research direction until supported by specification, implementation, tests, and comparative evaluation.

### 4.1 Verifier-first execution for reasoning programs

Semantic separates compilation from admission.

```text
compiled artifact != admitted program
```

A SemCode artifact is not trusted merely because it was emitted. It must pass the verifier before public execution.

Research value:

- formalizes admission as a first-class stage;
- separates producer behavior from execution permission;
- enables negative tests for malformed artifacts, unsupported capabilities, bad metadata, invalid control flow, or resource-budget violations;
- provides a basis for reproducible execution contracts.

### 4.2 Native quad logic as an execution-domain primitive

Semantic treats four-valued logic as a native semantic state domain:

| State | Meaning |
|---|---|
| `N` | unknown / no evidence |
| `F` | false evidence |
| `T` | true evidence |
| `S` | conflict / both true and false evidence |

The key research point is not merely using a four-valued logic table. The key point is integrating uncertainty and conflict into the execution model while keeping branch control explicit.

```text
quad value != implicit boolean branch authority
```

Research value:

- uncertainty and conflict become first-class runtime states;
- hidden coercion from uncertain/conflicting values into branch decisions can be restricted;
- reasoning state can be represented without collapsing early into `true` or `false`;
- the same state model can support semantic analysis, verifier rules, VM behavior, and controlled host effects.

### 4.3 SemCode as a verifiable artifact boundary

SemCode acts as the executable artifact boundary between source construction and VM execution.

```text
source constructs
  -> IR
  -> SemCode artifact
  -> verifier admission
  -> VM execution
```

Research value:

- makes the executable representation inspectable;
- supports versioned bytecode contracts;
- enables separate validation of emission, admission, and execution;
- creates a stable target for deterministic tests and future tooling.

### 4.4 Deterministic VM for semantic state transitions

Semantic treats VM execution as a deterministic state transition system.

```text
state[k + 1] = step(state[k], instruction[pc])
```

Given the same admitted artifact, runtime configuration, capability context, and input boundary, execution should remain reproducible.

Research value:

- makes runtime behavior testable by replay;
- supports golden tests and artifact hashing;
- reduces hidden nondeterminism in reasoning execution;
- creates a foundation for audit, rollback, and controlled observation.

### 4.5 Capability-gated PROMETHEUS boundary

Semantic separates VM execution from host authority.

```text
VM intent
  -> capability gate
  -> PROMETHEUS boundary
  -> controlled effect / denial
  -> audit
```

Research value:

- keeps host effects explicit;
- prevents UI/CLI/runtime orchestration from silently becoming authority;
- allows effect denial to be tested as a normal execution result;
- creates a controlled path for future agent-like or policy-driven execution.

### 4.6 Audit-oriented controlled observation

The controlled observation envelope and narrow print(text) path have landed:

```text
verified SemCode
  -> pure semantic construction (to_text)
  -> VM controlled observation event (print)
  -> ControlledObservationSink capability gate (CAP_STDOUT)
  -> audit decision
  -> CLI rendering envelope
```

Research value:

- demonstrates a safe observation path without broad stdout;
- separates program observation from unrestricted I/O;
- provides a small testable model for effect admission, capability gating, and CLI rendering;
- allows negative tests to prove that unsupported output paths remain rejected.

### 4.7 Minimal runtime ownership slice

Semantic currently favors a narrow ownership contract over broad ambiguous alias semantics.

Current supported slice:

- tuple access paths;
- direct record-field access paths;
- frame-local borrow lifetime;
- overlap rejection;
- parent-child / child-parent rejection;
- sibling writes when paths do not overlap.

Research value:

- provides a small formal surface for verified mutation control;
- avoids overclaiming advanced alias analysis before it is specified and tested;
- creates a staged path for future ownership expansion.

## 5. Research Questions

The project can be evaluated through the following research questions.

### RQ1 — Admission

Can SemCode admission reject malformed, unsupported, or unsafe artifacts before public execution without relying on VM traps as the primary safety boundary?

### RQ2 — Determinism

Given the same admitted artifact, runtime configuration, capability context, and input boundary, does the VM produce stable behavior across repeated runs?

### RQ3 — Quad-state execution

Can native `N / F / T / S` values represent uncertainty and conflict without implicit, unsafe collapse into boolean branch control?

### RQ4 — Capability boundaries

Can host effects and observation paths be represented as capability-gated operations with explicit allow/deny behavior and audit records?

### RQ5 — Runtime ownership

Can a small ownership slice prevent conflicting writes and unsafe overlap while remaining explainable, testable, and extensible?

### RQ6 — Public contract discipline

Can the repository maintain a strict separation between implementation landed on `main`, published stable behavior, experimental tracks, and explicit non-goals?

### RQ7 — Toolchain explainability

Can users trace a program from source to SemCode, verifier admission, VM execution, and controlled observation without relying on hidden runtime behavior?

## 6. Evaluation Plan

Scientific value should be supported by evidence. The following evaluation routes can turn the architecture into testable claims.

### 6.1 Golden pipeline tests

Use fixed source programs and fixed expected artifacts/results:

```text
source fixture
  -> expected check result
  -> expected SemCode shape/hash
  -> expected verifier result
  -> expected VM result
```

### 6.2 Negative admission tests

Construct intentionally invalid or unsupported SemCode artifacts:

- bad headers;
- invalid opcode sequences;
- malformed function envelopes;
- invalid jump targets;
- unsupported capability declarations;
- budget/resource mismatches;
- illegal observation encodings.

Expected result:

```text
verifier rejects before public execution
```

### 6.3 Deterministic replay tests

Run the same admitted artifact multiple times under the same runtime configuration and compare:

- stop reason;
- VM state summary;
- output/observation envelope, if enabled;
- audit records, where applicable;
- artifact hash and disassembly output.

### 6.4 Capability denial tests

Execute programs or artifacts that request controlled effects without the required capability.

Expected result:

```text
capability gate denies effect
VM / runtime reports controlled failure or denial
no unauthorized host effect occurs
```

### 6.5 Quad logic tests

Test all native quad-state operations for:

- `N / F / T / S` preservation;
- explicit branch restrictions;
- conflict propagation;
- unknown propagation;
- no hidden coercion into boolean branch control.

### 6.6 Runtime ownership tests

Use focused fixtures for:

- sibling paths;
- parent-child overlap;
- child-parent overlap;
- direct record-field access;
- tuple access;
- unsupported path rejection.

### 6.7 Documentation-to-test traceability

Every public behavior claim should map to at least one of:

- spec section;
- verifier test;
- VM test;
- CLI test;
- public API contract test;
- negative fixture;
- roadmap/status document.

## 7. Expected Contributions

If the research hypothesis continues to hold, Semantic can contribute in the following areas.

| Area | Potential contribution |
|---|---|
| Programming languages | A verifier-first source-to-artifact-to-VM execution pipeline for reasoning-oriented programs. |
| Runtime systems | Deterministic VM execution under explicit runtime budgets and capability context. |
| AI-agent safety | Capability-gated host effects and controlled observation paths for reasoning policies. |
| Formal methods / verification | A staged admission model that separates parse/compile success from execution permission. |
| Knowledge/state systems | Native four-valued semantic state representation for unknown, false, true, and conflicting evidence. |
| Tooling | Inspectable artifacts, hashes, disassembly, snapshots, and public contract gates. |

## 8. Scope Boundaries

This document does not claim that Semantic already provides:

- complete formal verification;
- production-ready safety for arbitrary AI agents;
- general-purpose language maturity;
- unrestricted I/O;
- general stdout;
- complete ownership / alias analysis;
- complete effect governance;
- a finished public ecosystem.

The current scientific value is in the architecture, implementation trajectory, and testable research hypotheses.

## 9. Communication Guidance

Use conservative language.

Prefer:

```text
Semantic explores verifier-first deterministic execution for reasoning-oriented programs.
```

Avoid:

```text
Semantic solves AI safety.
Semantic is a finished AGI language.
Semantic is production-ready for arbitrary autonomous systems.
```

Prefer:

```text
The current public track demonstrates controlled observation, not general stdout.
```

Avoid:

```text
Semantic already supports general printing and broad host I/O.
```

## 10. Short Abstract

Semantic is a deterministic verified execution platform that explores how reasoning-oriented programs can be represented as verifiable artifacts, admitted before execution, executed by a deterministic VM, and constrained by explicit semantic, capability, ownership, and effect boundaries. Its scientific value lies in treating reasoning execution as a staged, auditable, and testable process rather than as direct source-code execution.
