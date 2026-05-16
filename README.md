<p align="center">
  <img src="assets/brand/semantic-logo.png" alt="Semantic" width="860">
</p>

# Semantic Language

Semantic is a deterministic verified execution platform with native quad logic, SemCode VM, verifier-first admission, and controlled PROMETHEUS boundary integration.

## Visual Architecture Render

[![Semantic Snake AI Demo](https://img.youtube.com/vi/SKV-TfaJ0Fg/maxresdefault.jpg)](https://www.youtube.com/watch?v=SKV-TfaJ0Fg)

> Visual prototype for rendering execution pipelines, verifier gates, capability boundaries, runtime-state overlays, and architecture graphs.

## What Semantic Is

Semantic is built for programs that must be checked before they are allowed to run.

A Semantic program is compiled into SemCode, admitted by a verifier, and then executed by a deterministic VM under explicit runtime limits and capability boundaries.

The core execution path is:

```text
source
  -> frontend / semantic analysis
  -> IR / deterministic passes
  -> SemCode
  -> verifier admission
  -> deterministic VM
  -> optional PROMETHEUS boundary
```

Semantic is designed for:

- reasoning rules;
- semantic state transitions;
- native `quad` logic: `N / F / T / S`;
- deterministic execution;
- verifier-first admission;
- bounded runtime behavior;
- controlled host effects through explicit capabilities.

The repository `main` may contain work that is newer than the currently published stable line. Public claims should therefore be read through the canonical status model in `docs/roadmap/public_status_model.md`.

The public contract is centered in `docs/spec/*`. Historical roadmap notes and legacy compatibility shims remain in the repository, but they are not the primary source of truth for the current toolchain surface.

## Current Status

Status: **post-v1 contract-stabilized platform in controlled expansion**.

Semantic already has the core staged architecture in place:

```text
source
  -> frontend / semantic analysis
  -> IR / deterministic passes
  -> SemCode
  -> verifier admission
  -> deterministic VM
  -> optional PROMETHEUS boundary
```

Current `main` should be read as an active development line, not as a blanket stable-release promise. The rule remains:

```text
landed on main != published stable
```

The stable public contract lives in `docs/spec/*` and the release/status language is governed by `docs/roadmap/public_status_model.md`.

### Current active focus

The active implementation focus is **M-Hello**:

```text
#477 — M-Hello: admit minimal verified Hello World observation surface
#673 — PR-M-HELLO-12A-5: cli: render controlled observation envelope
```

This track is deliberately narrow. It is not adding general stdout or broad I/O. It is building one verified observation path:

```text
source
  -> check
  -> compile
  -> verify
  -> run
  -> controlled text observation
```

Recent M-Hello work has moved the observation path through the required lower layers:

- VM controlled observation event seam;
- verifier-side controlled observation admission seam;
- explicit `ControlledObservationSink` capability gate;
- controlled observation audit policy;
- CLI rendering envelope for `smc run` and `smc run-smc`.

The current open PR keeps source-run and verified-artifact routes separate and renders only after verifier admission, VM collection, capability allow, and audit decision.

### Current guarantees / design posture

Semantic currently prioritizes:

- verifier-first execution;
- deterministic VM state transitions;
- SemCode version/header discipline;
- native quad logic: `N / F / T / S`;
- explicit boolean branch control for quad values;
- runtime quotas / bounded execution;
- capability-gated PROMETHEUS boundary;
- auditability for controlled effects;
- source-to-artifact pipeline separation;
- no silent stable-claim widening.

The runtime ownership slice remains intentionally narrow and frozen around:

- tuple access paths;
- direct record-field access paths;
- frame-local borrow lifetime;
- runtime write rejection on overlapping active borrow paths.

Unsupported in that slice:

- ADT payload paths;
- schema paths;
- partial borrow release before frame exit;
- advanced alias/region reasoning;
- inter-frame borrow persistence;
- indirect field selection.

### UI / application boundary

The repository contains UI/application crates and Workbench-related surfaces, but UI is not the owner of compiler, verifier, VM, SemCode, capability, audit, or runtime semantics.

Current position:

```text
UI / Workbench / Studio = operator and application layer
Semantic Core           = language, SemCode, verifier, VM contracts
PROMETHEUS boundary     = capability, host effects, runtime/audit integration
```

`Semantic Studio` is tracked as a future unified control environment. It must use canonical pipeline APIs and must not bypass verifier admission.

### Current non-goals

The current status does not claim:

- general stdout;
- formatted printing;
- implicit scalar-to-text conversion;
- file / stdin / network I/O;
- broad Host ABI widening;
- UI-owned execution semantics;
- stable release promotion for every feature landed on `main`.

### Engineering rule

The repository discipline is:

```text
one logical change
  -> one PR
  -> tests where behavior changes
  -> docs/spec sync where contract changes
  -> no silent release claim widening
```

If a cleanup or UI task starts requiring new language/runtime behavior, it must move into the appropriate feature track instead of widening scope silently.

## Primary References

### Status and release posture
- `docs/roadmap/public_status_model.md` - canonical status vocabulary: stable, limited, main-only, out of scope
- `docs/roadmap/v1_readiness.md` - readiness posture and v1-oriented status model
- `reports/g1_execution_integrity.md` - execution-integrity gate report

### Canonical specification bundle
- `docs/spec/index.md` - spec bundle entrypoint
- `docs/spec/syntax.md` - source syntax contract
- `docs/spec/types.md` - source type contract
- `docs/spec/source_semantics.md` - source execution and binding semantics
- `docs/spec/semcode.md` - SemCode contract and version policy
- `docs/spec/verifier.md` - admission verifier contract
- `docs/spec/vm.md` - VM execution contract
- `docs/spec/runtime_ownership.md` - frozen tuple + direct record-field runtime ownership contract
- `docs/spec/cli.md` - public CLI surface

### Current M-Hello / controlled observation track
- `docs/language/semantic_hello_controlled_observation_encoding.md` - controlled observation encoding decision
- `docs/language/semantic_hello_observation_admission_runtime_path.md` - admission/runtime path map
- `docs/language/semantic_hello_observation_admission_shape.md` - verifier-facing observation shape
- `docs/language/semantic_hello_vm_observation_execution_route.md` - VM observation execution route
- `docs/language/semantic_hello_observation_capability_gate.md` - controlled observation capability gate
- `docs/language/semantic_hello_observation_audit_policy.md` - audit policy for controlled observation
- `docs/language/semantic_hello_cli_smoke_path.md` - CLI smoke path for controlled observation

### Onboarding and project orientation
- `docs/getting_started.md` - first practical onboarding path
- `docs/examples_index.md` - examples index
- `docs/LANGUAGE.md` - language overview and design intent
- `docs/NAMING.md` - naming rules and short forms
- `docs/NO_STD.md` - `no_std` / `alloc` / `std` support boundaries

### Architecture and compatibility perimeter
- `ARCHITECTURE.md` - repository-level architecture overview
- `docs/legacy-map.md` - retained compatibility and legacy perimeter inventory
- `docs/release_artifact_model.md` - release artifact model

## What Is In The Repository

The repository is organized as a layered workspace. No single crate owns the whole system; each layer has a narrow responsibility.

### Language construction layer

This layer turns `.sm` source into checked intermediate forms.

- `crates/sm-front` — lexer, parser, AST-facing source model, source-surface typing helpers
- `crates/sm-profile` — parser/profile policy, feature gates, compatibility profile surface
- `crates/sm-sema` — semantic analysis, diagnostics, imports/exports, symbol/type policy
- `crates/sm-ir` — IR model, lowering, deterministic passes, IR validation
- `crates/sm-emit` — producer-facing SemCode emission facade

### Execution layer

This layer owns admission, execution vocabulary, and verified VM behavior.

- `crates/sm-verify` — SemCode admission verifier
- `crates/sm-runtime-core` — runtime-safe shared vocabulary: quotas, traps, execution config, runtime IDs
- `crates/sm-vm` — deterministic SemCode VM and disassembly/runtime execution path

### PROMETHEUS boundary layer

This layer owns controlled interaction with host state, capabilities, gates, audit, and runtime integration.

- `crates/prom-abi` — host-call ABI vocabulary
- `crates/prom-cap` — capability policy and capability-denial model
- `crates/prom-gates` — gate descriptors and gate binding layer
- `crates/prom-runtime` — runtime session orchestration
- `crates/prom-state` — semantic state store
- `crates/prom-rules` — deterministic rule agenda and rule evaluation
- `crates/prom-audit` — audit, trace, and replay-oriented records

### Tooling layer

This layer is the user/operator-facing command surface. It may orchestrate many crates, but it does not own their internal semantics.

- `crates/smc-cli` — canonical public CLI owner for `check`, `compile`, `verify`, `run`, `run-smc`, `disasm`, diagnostics, snapshots, and related tooling
- root binary shims — public entrypoints such as `smc`, `svm`, and retained compatibility launchers

### UI / application layer

This layer is an operator/application shell. It must use canonical pipeline APIs and must not bypass verifier admission.

- `crates/prom-ui` — UI-facing model surface
- `crates/prom-ui-runtime` — UI/runtime bridge layer
- `crates/prom-ui-demo` — demo surface for UI/runtime integration
- `apps/workbench` — Workbench / future Studio-facing application shell

### Docs, tests, assets, and reports

These paths carry the public contract, active design records, regression coverage, and project assets.

- `docs/spec/*` — canonical public contract bundle
- `docs/language/*` — language-track and active feature-track design records
- `docs/roadmap/*` — status, readiness, and roadmap control documents
- `reports/*` — gate reports and release/readiness evidence
- `tests/*` — integration, contract, fixture, and regression tests
- `assets/*` — brand and retained non-core assets

### Compatibility perimeter

The repository intentionally retains a narrow compatibility perimeter. It is not a second owner of the Semantic platform contracts.

- `crates/ton618-core` — compatibility-named low-level primitive crate
- `src/bin/ton618_core.rs` — retained compatibility launcher
- `docs/legacy-map.md` — authoritative inventory for retained legacy/compatibility paths

New architecture must land in the appropriate owner crate, not in compatibility paths.

## Quickstart

This quickstart is a **zero-effect verifier smoke path**. It proves the core pipeline:

```text
source -> check -> compile -> verify -> run -> disasm
```

It does not demonstrate general stdout, formatting, file I/O, network I/O, or the M-Hello controlled observation track.

### 1. Build the public entrypoints

```powershell
cargo build --bin smc --bin svm
```

### 2. Create a minimal Semantic source file

```powershell
@'
fn main() {
    return;
}
'@ | Set-Content smoke_zero.sm
```

This program has no host effects. It is intentionally minimal.

### 3. Check source

```powershell
cargo run --bin smc -- check smoke_zero.sm
```

This validates the source through the frontend / semantic-analysis route.

### 4. Compile source to SemCode

```powershell
cargo run --bin smc -- compile smoke_zero.sm -o smoke_zero.smc
```

This emits a SemCode artifact from the checked source path.

### 5. Verify the SemCode artifact

```powershell
cargo run --bin smc -- verify smoke_zero.smc
```

This is the admission gate. Public `.smc` execution is verifier-first.

### 6. Run from source

```powershell
cargo run --bin smc -- run smoke_zero.sm
```

This exercises the source-run route.

### 7. Run the verified artifact route

```powershell
cargo run --bin smc -- run-smc smoke_zero.smc
```

This exercises the precompiled SemCode route.

### 8. Disassemble SemCode

```powershell
cargo run --bin svm -- disasm smoke_zero.smc
```

This confirms the artifact can be inspected through the VM tooling route.

### Expected result

The smoke path should:

- accept the minimal source file;
- emit `smoke_zero.smc`;
- admit the SemCode artifact through `smc verify`;
- run both source and `.smc` routes without host effects;
- produce disassembly containing the compiled `main` function.

For controlled text observation / Hello World work, follow the active M-Hello documents under `docs/language/semantic_hello_*`.

For a fuller onboarding path, see:

- `docs/getting_started.md`
- `docs/examples_index.md`

## Current CLI Surface

`smc` is the canonical public CLI for the Semantic toolchain. It is the preferred route for source checks, SemCode emission, verifier admission, and public execution flows.

### Core pipeline commands

These commands represent the standard source-to-execution path.

| Command | Role |
|---|---|
| `smc check <file.sm>` | Parse and semantically check source without emitting an artifact. |
| `smc compile <file.sm> -o <file.smc>` | Compile source into a SemCode artifact. |
| `smc verify <file.smc>` | Admit or reject a SemCode artifact before execution. |
| `smc run <file.sm>` | Run from source through the standard CLI route. |
| `smc run-smc <file.smc>` | Run a precompiled SemCode artifact through the standard CLI route. |

### Inspection and artifact commands

These commands expose intermediate artifacts and stable inspection surfaces.

| Command | Role |
|---|---|
| `smc dump-ast <file.sm>` | Show the parsed source structure. |
| `smc dump-ir <file.sm>` | Show the lowered IR path. |
| `smc dump-bytecode <file.sm>` | Show emitted bytecode-oriented information. |
| `smc disasm <file.smc>` | Disassemble a SemCode artifact through the CLI route. |
| `smc hash-ast <file.sm>` | Produce a stable AST-oriented hash. |
| `smc hash-ir <file.sm>` | Produce a stable IR-oriented hash. |
| `smc hash-smc <file.smc>` | Produce a stable SemCode artifact hash. |
| `smc snapshots ...` | Work with snapshot-oriented regression artifacts. |

### Diagnostics and developer tooling

These commands support editing, diagnostics, and toolchain discovery.

| Command | Role |
|---|---|
| `smc lint <file.sm>` | Run lint-style checks. |
| `smc fmt <path>` | Format source files or check formatting. |
| `smc explain <code>` | Explain a diagnostic or toolchain code. |
| `smc features` | Show exposed feature/profile information. |
| `smc watch ...` | Watch files and rerun selected checks. |
| `smc repl` | Start the interactive Semantic REPL route. |

### Low-level VM entrypoint

`svm` is the lower-level VM-oriented entrypoint. It is useful for VM-focused inspection, but `smc` remains the canonical public toolchain route.

| Command | Role |
|---|---|
| `svm run <file.smc>` | Run a SemCode artifact through the VM entrypoint. |
| `svm disasm <file.smc>` | Disassemble a SemCode artifact through the VM entrypoint. |

### CLI boundary rule

The CLI must not become a second execution semantics owner.

```text
smc orchestrates
verifier admits or rejects
VM executes
PROMETHEUS boundary controls effects
```

Public `.smc` execution remains verifier-first. Controlled observation / Hello World output is tracked separately in the active M-Hello path and must not be read as general stdout support.

## Current SemCode And Runtime Notes

SemCode is the executable artifact boundary of Semantic. It is not source syntax and it is not the host runtime. A `.smc` artifact must be structurally valid, admitted by the verifier, and executed under the runtime contract.

### SemCode ownership

Current repository ownership is split deliberately:

| Area | Owner |
|---|---|
| Source syntax and source semantics | `sm-front`, `sm-sema` |
| IR and lowering path | `sm-ir` |
| Producer-facing SemCode emission facade | `sm-emit` |
| SemCode admission / rejection | `sm-verify` |
| Runtime vocabulary: quotas, traps, execution config | `sm-runtime-core` |
| Deterministic execution and disassembly | `sm-vm` |
| Host effects, capabilities, gates, audit | `prom-*` boundary crates |

The important rule is simple:

```text
source constructs do not execute directly
SemCode does not bypass admission
VM execution does not own host authority
```

### Verifier-first execution

Public `.smc` execution is verifier-first.

```text
SemCode bytes
  -> structural / capability / resource admission
  -> verified program
  -> deterministic VM execution
```

The VM is not expected to be the only safety boundary. Invalid opcodes, malformed function envelopes, unsupported capability use, bad jump targets, resource-budget violations, and incompatible SemCode metadata belong at the admission boundary before public execution.

### Runtime model

The runtime is treated as a deterministic state transition system.

```text
state[k + 1] = step(state[k], instruction[pc])
```

The runtime state includes, at minimum:

- program counter and current instruction;
- frame / call-stack state;
- runtime values and registers;
- quotas / bounded execution limits;
- traps and stop reasons;
- active ownership paths;
- capability context;
- optional PROMETHEUS host-boundary state.

Given the same admitted SemCode, runtime configuration, capability context, and input boundary, execution must remain deterministic.

### Runtime ownership slice

The currently documented ownership slice is intentionally narrow and frozen.

Supported:

- tuple access paths;
- direct record-field access paths;
- frame-local borrow lifetime;
- exact overlap rejection;
- parent-child overlap rejection;
- child-parent overlap rejection;
- sibling writes allowed when paths do not overlap.

Unsupported in the current slice:

- ADT payload paths;
- schema paths;
- partial borrow release before frame exit;
- advanced alias / region reasoning;
- inter-frame borrow persistence;
- indirect projections;
- smart path normalization.

This is deliberate: the current runtime ownership model prefers a small verified contract over broad but ambiguous alias semantics.

### Controlled observation note

The active M-Hello work adds a narrow controlled text observation route. It must not be read as general stdout.

The intended controlled observation path is:

```text
verified SemCode
  -> VM controlled observation event
  -> ControlledObservationSink capability gate
  -> audit decision
  -> CLI rendering envelope
```

Out of scope for this route:

- general stdout;
- formatted printing;
- implicit scalar-to-text conversion;
- file / stdin / network I/O;
- broad host ABI widening.

### Stability rule

SemCode/runtime behavior must not be promoted from `main` to public stable language without matching spec, verifier, VM, CLI, and test coverage.

```text
implementation landed
  != public contract widened
```

## Testing

Tests are treated as contract evidence, not only as regression checks. A behavior should not be promoted in README, examples, or specs unless the corresponding tests cover the pipeline stage that owns it.

### Minimal pre-PR gate

Run this before any normal code or docs PR:

```powershell
cargo fmt --check
cargo test -q
```

### Public contract gates

Run these when a change touches public API, CLI behavior, runtime ownership, SemCode, verifier admission, or README/spec language:

```powershell
cargo test -q --test public_api_contracts
cargo test -q --test runtime_ownership_e2e
```

### Layer-focused checks

Use focused package tests when working inside a specific owner layer:

```powershell
cargo test -q -p sm-verify
cargo test -q -p sm-vm
cargo test -q -p smc-cli
cargo test -q -p prom-cap
cargo test -q -p prom-audit
```

Run only the relevant subset when the change is isolated. Run the broader workspace test when the change crosses ownership boundaries.

### M-Hello / controlled observation checks

For the active controlled-observation track, use the focused tests that cover the source route, verifier route, VM observation route, capability gate, audit policy, and CLI envelope:

```powershell
cargo test -q --test hello_cli_observation_envelope
cargo test -q --test hello_cli_smoke_pipeline_harness
cargo test -q --test hello_real_semcode_admission
cargo test -q --test hello_real_semcode_negative_encodings
cargo test -q --test hello_observation_capability_skeleton
cargo test -q --test public_api_contracts
```

These tests are for the narrow controlled text observation path. Passing them does not imply general stdout, formatting, file I/O, stdin, network I/O, or broad host ABI support.

### Test selection rule

```text
changed layer
  -> run that layer's focused tests
  -> run public contract tests if behavior is visible
  -> run full cargo test if ownership boundaries are crossed
```

No PR should widen a public claim without matching tests and documentation updates.

## no_std Smoke Check
Core library supports `no_std` mode.

```powershell
cargo check --no-default-features
```

Reference:
- `docs/NO_STD.md`

## License
Apache License 2.0

Copyright (c) 2026 Said Kulmakov

See `LICENSE` for the repository license text.