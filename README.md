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
Current command families exposed by `smc`:
- `compile`
- `check`
- `lint`
- `watch`
- `fmt`
- `dump-ast`
- `dump-ir`
- `dump-bytecode`
- `hash-ast`
- `hash-ir`
- `hash-smc`
- `snapshots`
- `features`
- `explain`
- `repl`
- `verify`
- `run`
- `run-smc`
- `disasm`

Low-level VM entrypoint:
- `svm run <input.smc>`
- `svm disasm <input.smc>`

## Current SemCode And Runtime Notes
- The SemCode contract is owned by `sm-ir` and surfaced through `sm-emit`.
- The current spec documents a versioned SemCode family and capability-gated emission.
- Standard `.smc` execution is verifier-first; verified admission is not optional on the public route.
- The current runtime ownership slice is intentionally narrow:
  - tuple paths
  - direct record-field paths
  - frame-local borrow lifetime
  - exact overlap rejection
  - parent-child rejection
  - child-parent rejection
  - sibling writes allowed
  - unsupported: ADT payload paths, schema paths, partial release, aliasing graphs, inter-frame persistence, and indirect projections

## Testing
```powershell
cargo fmt --check
cargo test -q
cargo test -q --test public_api_contracts
cargo test -q --test runtime_ownership_e2e
```

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