# Host Runtime Effect Path Boundary for Semantic UI

Status: Draft
Track: POST-UI
Scope: host/runtime UI effect path boundary only
Implementation: out of scope

Related:

- `README.md`
- `../../architecture/ui_host_runtime_effect_boundary.md`
- `../../architecture/ui_full_effect_trace_ladder.md`
- `ui_runtime_adapter_boundary.md`
- `../abi.md`
- `../capabilities.md`
- `../audit.md`
- `../runtime.md`
- `../vm.md`

## 1. Purpose

This document defines the only allowed path for UI-visible host effects in the
Semantic stack.

The Semantic VM does not create windows, read host input, or draw to a platform
surface directly. It produces an effect request or host-call envelope.

The request then passes through a runtime boundary:

```text
capability -> budget -> audit -> prom-ui-runtime -> platform adapter
```

This document freezes that boundary before any UI implementation can bypass the
PROMETHEUS integration layers.

## 2. Canonical path

```text
Semantic Program
  -> Verified SemCode
  -> Semantic VM
  -> HostCallEnvelope / UiEffectEnvelope
  -> prom-runtime
  -> capability check
  -> budget check
  -> audit intent
  -> prom-ui-runtime
  -> platform adapter
  -> host window / event / draw surface
  -> audit outcome
  -> VM-visible result / trap / denial
```

Short form:

```text
VM requests effect.
Runtime admits effect.
UI runtime translates effect.
Platform adapter performs effect.
Audit records intent and outcome.
```

## 3. Ownership map

### 3.1 Semantic VM

Owns:

- deterministic instruction execution;
- frame/register/value model;
- quota trap integration;
- emission of host/effect request envelopes.

Must not own:

- window lifecycle;
- platform event queue;
- rendering backend;
- OS handles;
- UI capability policy;
- UI audit schema;
- platform-specific error handling.

### 3.2 `prom-runtime`

Owns:

- runtime session orchestration;
- effect admission flow;
- capability checker attachment;
- budget context;
- audit coordination;
- runtime result and error boundary.

Must not own:

- actual windowing;
- draw-command execution;
- widget tree ownership;
- renderer internals;
- platform event source ownership.

### 3.3 `prom-ui-runtime`

Owns:

- UI effect vocabulary;
- window and session UI state boundary;
- frame lifecycle contract;
- event polling abstraction;
- draw-command admission surface;
- translation from UI effects to platform adapter calls.

Must not own:

- VM instruction semantics;
- verifier rules;
- capability policy source of truth;
- audit storage format;
- platform-specific implementation details.

### 3.4 Platform adapter

Examples:

- `prom-ui-runtime-windows`
- `prom-ui-runtime-macos`
- `prom-ui-runtime-linux`

Owns:

- OS window handles;
- platform event loop integration;
- native surface access;
- platform draw submission;
- platform error mapping.

Must not own:

- Semantic language semantics;
- capability decision;
- audit decision;
- VM state;
- SemCode verification.

## 4. HostCallEnvelope role

`HostCallEnvelope` or `UiEffectEnvelope` is the explicit runtime request shape
that leaves VM execution and enters PROMETHEUS admission.

It is a boundary object, not a host call itself.

The envelope records intent so that:

- capability checks can fail closed;
- budget checks can run before execution;
- audit intent can be recorded before critical host effects;
- platform adapters can remain dumb executors of admitted effects.

The VM must not infer a hidden host path from this envelope.

## 5. Required admission order

The required order is:

```text
1. VM creates effect envelope
2. Runtime validates envelope shape
3. Capability check
4. Budget check
5. Audit intent for critical effect
6. UI runtime dispatch
7. Platform adapter execution
8. Audit outcome
9. Return result / trap / denial
```

This order is strict.

It is forbidden to reorder it into:

- platform call first;
- audit after success only;
- capability check inside the backend;
- budget check after the effect;
- VM direct access to OS APIs.

## 6. Effect classes

Minimal UI effect classes:

| Effect | Class | Requires capability | Audit |
| --- | --- | --- | --- |
| `WindowCreate` | host-visible | yes | intent/outcome |
| `WindowClose` | host-visible | yes | outcome |
| `PollEvents` | input read | yes | optional / sampled |
| `BeginFrame` | frame lifecycle | yes | trace-level |
| `SubmitDrawCommands` | output write | yes | trace / budgeted |
| `EndFrame` | frame lifecycle | yes | trace-level |
| `ClipboardRead` | sensitive input | yes, separate | required |
| `ClipboardWrite` | host write | yes, separate | required |

Clipboard effects are reserved for future work and are not admitted here.

## 7. Forbidden direct paths

The following direct paths are forbidden:

- `Semantic VM -> OS window API`
- `Semantic VM -> renderer backend`
- `Semantic VM -> input device API`
- `Semantic VM -> filesystem asset loading for UI`
- `prom-ui-runtime -> bypass capability checker`
- `prom-ui-runtime -> bypass audit for critical effects`
- `platform adapter -> mutate Semantic VM state`
- `platform adapter -> interpret SemCode`
- `UI layer -> call sm-front / sm-ir / sm-emit internals`

Core rule:

```text
No direct UI host effect from VM.
No UI host effect without runtime admission.
No runtime admission without capability path.
```

## 8. Determinism boundary

Semantic VM execution remains deterministic.

The UI event stream itself is external and nondeterministic, but its entry into
the VM must be explicit, recorded, and replayable.

That means:

```text
mouse click happened outside VM
  -> prom-ui-runtime normalizes it into UiEventEnvelope
  -> runtime/audit can record it
  -> VM consumes an explicit event value
```

The VM must not query the host directly for current pointer or window state.

## 9. Error and denial model

Future host effect admission should use explicit denial categories such as:

- `CapabilityDenied`
- `BudgetExceeded`
- `AuditRequired`
- `PlatformFailure`
- `UnsupportedEffect`
- `InvalidEnvelope`

Denied effects must remain visible.

## 10. Out of scope

This PR does not add:

- code changes;
- ABI widening;
- VM changes;
- renderer implementation;
- widget or layout framework;
- platform backend implementation;
- tests beyond docs or link checks;
- capability enforcement code;
- audit WAL implementation;
- new opcode definitions.

## 11. Future PR hooks

Future docs or implementation PRs may define:

- UI capability taxonomy;
- UI effect envelope v0;
- deterministic event envelope model;
- frame lifecycle contract;
- UI runtime smoke tests;
- platform-specific adapter boundaries.

## 12. Acceptance checklist

- canonical path diagram exists;
- ownership map exists;
- forbidden paths are listed;
- capability/budget/audit order is defined;
- deterministic event boundary is explained;
- future PR hooks are listed;
- no code changes are introduced;
- no ABI widening is claimed.

## 13. Relationship to runtime adapter boundary

The runtime adapter boundary is defined in:

```text
docs/spec/ui/ui_runtime_adapter_boundary.md
```

That document defines how normalized, admitted runtime requests move from
`prom-ui-runtime` into platform adapters.
