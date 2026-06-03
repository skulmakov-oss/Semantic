# Public Maturity Snapshot

Status: current-main orientation document.

This document exists to make the public state of Semantic easier to read from the outside. It does not widen the stable contract by itself. Public behavior claims remain governed by `docs/spec/*`, `docs/roadmap/public_status_model.md`, and corresponding tests.

## One-line position

Semantic is a deterministic verified execution platform for source-to-SemCode workflows, verifier-first admission, deterministic VM execution, native quad logic, and controlled PROMETHEUS boundary integration.

## What is already materially present on `main`

| Area | Current state |
|---|---|
| Workspace structure | Multi-crate Rust workspace with separated frontend, IR, emission, verifier, VM, runtime core, CLI, PROMETHEUS boundary, UI/application, docs, tests, and compatibility perimeter. |
| Source pipeline | `.sm` source and directory project roots can flow through check / compile-oriented tooling routes. |
| SemCode artifact model | SemCode exists as the executable artifact boundary and is documented as a versioned contract surface. |
| Verifier-first posture | Public `.smc` execution is treated as verifier-first; verifier admission is a first-class architecture boundary. |
| Deterministic VM | VM execution and disassembly are separated from source construction and CLI orchestration. Includes deterministic seeded PRNG (xorshift64). |
| Quad logic | `N / F / T / S` are treated as native semantic states rather than ad-hoc comments or external flags. |
| Imperative Core | Landed same-family `i32` arithmetic (`+`, `-`, `*`, `/`, `%`), mutable locals (`let mut` + reassignment), and loop control exits (`while`, `loop`, `break`, `continue`). |
| Data Collections | Persistent `Sequence(T)` with utility functions (`len`, `push`, `pop`, `prepend`, `contains`) and persistent `Map(K, V)` persistent lookups. |
| Runtime ownership slice | Tuple and direct record-field access paths are documented as the current narrow/frozen ownership contract. |
| PROMETHEUS boundary | Capability, ABI, gate, runtime, state, rule, and audit crates exist as a controlled host-boundary layer. Narrow `print(text)` observation enabled via `CAP_STDOUT`. |
| CLI tooling | `smc` and `svm` expose check, compile, verify, run, run-smc, disassembly, inspection, hashes (ast), snapshots, diagnostics, and related routes for both single files and project-roots. |
| Testing discipline | Contract, ownership, M-Hello, package-focused, project-root acceptance tests, and workspace-level test routes are documented. |
| no_std posture | A core-library `no_std` smoke check exists, scoped explicitly away from CLI/UI/full-workspace claims. |
| Licensing | Repository licensing is aligned around Apache-2.0 with separate `NOTICE` attribution / project-scope notes. |

## Active development focus

The current active hardening focus is **project-root CLI/status reconciliation and application-completeness evidence alignment**.

```text
source (file / project-root)
  -> check
  -> compile
  -> verify
  -> run
  -> output / observation
```

The CLI pipeline has been updated to resolve entrypoints and execute routes directly from the project root under a bounded project-root baseline:

```text
smc check <project-root>
smc run <project-root>
smc compile <project-root>
smc dump-ast <project-root>
smc dump-ir <project-root>
smc dump-bytecode <project-root>
smc hash-ast <project-root>
```

## Explicit non-claims

Current public documentation does not claim:

- broad standard library or arbitrary host effects;
- general file / stdin / network I/O;
- broad host ABI widening;
- UI-owned execution semantics;
- full-workspace `no_std`;
- stable promotion of every feature that has landed on `main`.

## Roadmap shape

| Phase | Goal | Boundary |
|---|---|---|
| MVP / contract core | Keep the source -> SemCode -> verifier -> VM path stable, testable, and explainable. | No broad I/O or unstable feature promotion. |
| Application Completeness | Benchmark-class execution (Q-learning headless Snake) with PRNG, Maps, same-family arithmetic, loops, and narrow stdout observation. | Controlled observation does not become general stdout by accident. |
| Project root model | Bounded project-root CLI baseline (`semantic.toml` entrypoint resolution) and local validation workflows. Excludes full package ecosystem, registry, multi-package resolution, and package manager semantics. | Keep project manifest boundaries strict and failure diagnostics deterministic. |

## Custody and visibility note

Repository visibility is an access-control posture, not a change to the technical maturity model.

When the repository is private, `docs/roadmap/private_custody_mode.md` defines the custody discipline used to keep branch protection, PR traceability, release-facing non-claims, and public communication boundaries intact.

Private work can be broader than public claims, but public-facing language still remains constrained by specs, tests, status documents, and explicit non-claims.

## How to read this repository

A first-time reader should not treat the repository as a finished general-purpose language product. It is better read as a serious R&D / platform project with a real staged architecture and active contract-hardening work.

The practical reading order is:

1. `README.md`
2. `docs/status/feature_maturity_matrix.md`
3. `docs/roadmap/public_status_model.md`
4. `docs/spec/index.md`
5. `docs/spec/semcode.md`
6. `docs/spec/verifier.md`
7. `docs/spec/vm.md`
8. `docs/spec/runtime_ownership.md`
9. `docs/language/semantic_hello_*`
10. `docs/roadmap/private_custody_mode.md`, when repository custody / visibility policy is relevant

## Public communication rule

Use precise language:

```text
implemented on main
  != published stable
  != production-ready
  != general-purpose runtime support
```

Semantic should be presented as a deterministic verified execution platform in controlled expansion, not as a broad language ecosystem that already promises every planned feature.