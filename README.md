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
- `docs/spec/index.md` - canonical spec bundle entrypoint
- `docs/spec/syntax.md` - source syntax contract
- `docs/spec/types.md` - source type contract
- `docs/spec/source_semantics.md` - source execution and binding semantics
- `docs/spec/semcode.md` - SemCode contract and version policy
- `docs/spec/verifier.md` - admission verifier contract
- `docs/spec/vm.md` - VM execution contract
- `docs/spec/runtime_ownership.md` - frozen tuple + direct record-field runtime ownership contract
- `docs/spec/cli.md` - public CLI surface
- `docs/LANGUAGE.md` - language overview and design intent
- `docs/NAMING.md` - naming rules and short forms

## What Is In The Repository
- Source frontend: lexer, parser, typing, and source-surface ownership work in `crates/sm-front`
- Semantic analysis and diagnostics in `crates/sm-sema`
- Lowering, IR, optimization passes, and canonical SemCode contract in `crates/sm-ir`
- Producer-facing SemCode facade in `crates/sm-emit`
- Structural SemCode admission verifier in `crates/sm-verify`
- Shared runtime vocabulary and quotas in `crates/sm-runtime-core`
- Verified-only VM execution in `crates/sm-vm`
- Canonical public CLI owner in `crates/smc-cli`
- Additional boundary/runtime crates currently present on `main`:
  - `crates/prom-abi`
  - `crates/prom-cap`
  - `crates/prom-gates`
  - `crates/prom-runtime`
  - `crates/prom-state`
  - `crates/prom-rules`
  - `crates/prom-audit`
  - `crates/prom-ui`
  - `crates/prom-ui-runtime`
  - `crates/prom-ui-demo`
- Compatibility perimeter:
  - `src/bin/ton618_core.rs`
  - `crates/ton618-core`

## Quickstart
Use these commands from repository root.

```powershell
# 1) Build the public entrypoints
cargo build --bin smc --bin svm

# 2) Create a minimal program
@'
fn main() {
    return;
}
'@ | Set-Content program.sm

# 3) Check source
cargo run --bin smc -- check program.sm

# 4) Compile source -> SemCode
cargo run --bin smc -- compile program.sm -o program.smc

# 5) Verify compiled SemCode
cargo run --bin smc -- verify program.smc

# 6) Run source directly
cargo run --bin smc -- run program.sm

# 7) Run precompiled SemCode through the standard CLI route
cargo run --bin smc -- run-smc program.smc

# 8) Disassemble SemCode
cargo run --bin svm -- disasm program.smc
```

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