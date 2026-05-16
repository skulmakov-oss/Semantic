# Public Maturity Snapshot

Status: current-main orientation document.

This document exists to make the public state of Semantic easier to read from the outside. It does not widen the stable contract by itself. Public behavior claims remain governed by `docs/spec/*`, `docs/roadmap/public_status_model.md`, and corresponding tests.

## One-line position

Semantic is a deterministic verified execution platform for source-to-SemCode workflows, verifier-first admission, deterministic VM execution, native quad logic, and controlled PROMETHEUS boundary integration.

## What is already materially present on `main`

| Area | Current state |
|---|---|
| Workspace structure | Multi-crate Rust workspace with separated frontend, IR, emission, verifier, VM, runtime core, CLI, PROMETHEUS boundary, UI/application, docs, tests, and compatibility perimeter. |
| Source pipeline | `.sm` source can flow through check / compile-oriented tooling routes. |
| SemCode artifact model | SemCode exists as the executable artifact boundary and is documented as a versioned contract surface. |
| Verifier-first posture | Public `.smc` execution is treated as verifier-first; verifier admission is a first-class architecture boundary. |
| Deterministic VM | VM execution and disassembly are separated from source construction and CLI orchestration. |
| Quad logic | `N / F / T / S` are treated as native semantic states rather than ad-hoc comments or external flags. |
| Runtime ownership slice | Tuple and direct record-field access paths are documented as the current narrow/frozen ownership contract. |
| PROMETHEUS boundary | Capability, ABI, gate, runtime, state, rule, and audit crates exist as a controlled host-boundary layer. |
| CLI tooling | `smc` and `svm` expose check, compile, verify, run, run-smc, disassembly, inspection, hashes, snapshots, diagnostics, and related routes. |
| Testing discipline | Contract, ownership, M-Hello, package-focused, and workspace-level test routes are documented. |
| no_std posture | A core-library `no_std` smoke check exists, scoped explicitly away from CLI/UI/full-workspace claims. |
| Licensing | Repository licensing is aligned around Apache-2.0 with separate `NOTICE` attribution / project-scope notes. |

## Active development focus

The active public focus is **M-Hello**: a narrow, verified, controlled text observation path.

```text
source
  -> check
  -> compile
  -> verify
  -> run
  -> controlled text observation
```

This is deliberately not general stdout. The intended route is:

```text
verified SemCode
  -> VM controlled observation event
  -> ControlledObservationSink capability gate
  -> audit decision
  -> CLI rendering envelope
```

## Explicit non-claims

Current public documentation does not claim:

- general stdout;
- broad `print` / formatting support;
- implicit scalar-to-text conversion;
- file / stdin / network I/O;
- broad host ABI widening;
- UI-owned execution semantics;
- full-workspace `no_std`;
- stable promotion of every feature that has landed on `main`.

## Roadmap shape

| Phase | Goal | Boundary |
|---|---|---|
| MVP / contract core | Keep the source -> SemCode -> verifier -> VM path stable, testable, and explainable. | No broad I/O or unstable feature promotion. |
| Controlled observation | Finish M-Hello as a narrow verified observation path with capability and audit boundaries. | Controlled observation does not become general stdout by accident. |
| Production hardening | Harden specs, tests, diagnostics, release artifacts, runtime contracts, and public status labels. | No public claim widens without spec + tests + verifier/VM/CLI coverage. |

## How to read this repository

A first-time reader should not treat the repository as a finished general-purpose language product. It is better read as a serious R&D / platform project with a real staged architecture and active contract-hardening work.

The practical reading order is:

1. `README.md`
2. `docs/roadmap/public_status_model.md`
3. `docs/spec/index.md`
4. `docs/spec/semcode.md`
5. `docs/spec/verifier.md`
6. `docs/spec/vm.md`
7. `docs/spec/runtime_ownership.md`
8. `docs/language/semantic_hello_*`

## Public communication rule

Use precise language:

```text
implemented on main
  != published stable
  != production-ready
  != general-purpose runtime support
```

Semantic should be presented as a deterministic verified execution platform in controlled expansion, not as a broad language ecosystem that already promises every planned feature.
