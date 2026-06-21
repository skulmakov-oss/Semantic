# External Analysis Guardrails

Status: public-reading guardrail
Owner: release discipline / architecture wording
Related authority: `docs/roadmap/public_status_model.md`

## Purpose

This document exists to prevent external summaries, AI-generated reviews, and
public-facing writeups from converting historical names, roadmap shorthand, or
future-fit positioning into unsupported current claims.

It does not widen the release contour.
It does not change implementation behavior.
It does not promote current-`main` behavior into the published stable line.

It only records how the repository should be read when describing Semantic from
outside the project.

## Canonical Reading Order

External analysis should read repository claims in this order:

1. `README.md`
   - high-level project shape and first public orientation
2. `docs/roadmap/public_status_model.md`
   - status vocabulary authority
3. `docs/roadmap/v1_readiness.md`
   - release/readiness posture authority
4. `docs/spec/*`
   - canonical public contract bundle
5. `docs/legacy-map.md`
   - retained compatibility and legacy perimeter inventory
6. `docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md`
   - TON618 compatibility perimeter governance
7. `docs/roadmap/language_maturity/core_trust_freeze/index.md`
   - CTF / Core Trust Freeze lane
8. `docs/roadmap/language_maturity/7hell_report_contract.md`
   - 7hell qualification-report contract

Roadmap notes, issue discussions, visual diagrams, and historical names are not
allowed to override those sources.

## Preferred One-Sentence Description

Use this when a compact external description is needed:

```text
Semantic is a deterministic verified execution platform for Semantic Language
programs, built around source checking, IR lowering, SemCode emission,
verifier-first admission, deterministic VM execution, native quad logic, and a
controlled PROMETHEUS boundary.
```

This description is intentionally technical and bounded. It should not be
expanded into compliance, certification, cryptographic, financial, medical, or
zero-trust production claims unless a later release document explicitly supports
those claims.

## Correct Current Architecture Reading

The current public architecture should be read as an owner-split workspace:

```text
source / syntax / source semantics  -> sm-front, sm-sema
IR and lowering                     -> sm-ir
SemCode emission facade             -> sm-emit
SemCode admission                   -> sm-verify
runtime vocabulary                  -> sm-runtime-core
deterministic VM execution          -> sm-vm
host effects / capabilities / audit -> prom-* boundary crates
CLI orchestration                   -> smc-cli
UI / Workbench                      -> operator/application layer only
```

No single crate owns the whole platform.
No compatibility name should be promoted into a second platform owner.
No UI or tooling surface should be described as an execution authority.

## TON618 Wording Rule

Use the canonical phrase:

```text
retained non-owning TON618 compatibility perimeter
```

This currently covers:

- `src/bin/ton618_core.rs`
- `crates/ton618-core`
- `ton618_legacy/`

Do not describe TON618 as:

- the canonical execution core;
- the main owner of the VM;
- the memory manager for the platform;
- the owner of SemCode admission;
- the owner of PROMETHEUS integration;
- a second owner for `sm-*` platform contracts.

Allowed wording:

```text
TON618 names are retained for compatibility and low-level primitive history.
Canonical ownership for the current toolchain and execution contracts lives in
sm-* and prom-* crates.
```

## SemCode / VM Wording Rule

Correct wording:

```text
SemCode is the executable artifact boundary. Public `.smc` execution is
verifier-first: SemCode must be admitted before deterministic VM execution.
```

Avoid wording that implies:

- source executes directly;
- SemCode bypasses admission;
- the VM is the only safety boundary;
- malformed bytecode is expected to be handled only at runtime;
- host effects are raw VM authority.

## Quad Logic Wording Rule

Correct wording:

```text
Semantic has native quad logic with `N / F / T / S` semantic states.
Quad values carry unknown/conflict information, while branch control remains
explicit and boolean-facing.
```

Do not imply that current public docs promise:

- hardware acceleration for quad operations;
- a finalized SIMD backend;
- a graph database or RDF quad-store implementation;
- automatic conflict resolution through meta-rules;
- that `if quad_expr` is valid branch syntax.

If discussing future packed or accelerated representations, mark them as future
backend work or implementation strategy, not as a current public guarantee.

## CTF Wording Rule

`CTF` means:

```text
Core Trust Freeze
```

It is the parallel trust lane for Practical Core Completion. It exists to keep
runtime values, traps, verifier expectations, determinism, SymbolId assumptions,
capability behavior, and golden traces from drifting silently while language
surface expands.

Do not expand CTF as:

- Compile-Time Feature Guard;
- Capture-The-Flag;
- compiler-only feature gating;
- a final release phase that starts only after PCC.

## 7hell Wording Rule

`7hell` is a qualification/reporting concept, not a memory mapping system.

Correct reading:

```text
7hell = staged qualification gauntlet / report contract
```

Canonical stages:

1. Syntax Hell
2. Type Hell
3. Lowering Hell
4. Verifier Hell
5. VM Hell
6. Practical Hell
7. User Pain / Diagnostics Hell

Do not describe `7hell` as:

- memory mapping;
- graph flattening;
- VM address-space routing;
- a runtime state-layout algorithm;
- a production CI gate unless a later policy explicitly promotes it.

## Release / Readiness Wording Rule

Use the public status vocabulary:

- `published stable`
- `qualified limited release`
- `landed on main, not yet promised`
- `out of scope`

Do not use vague phrases such as:

- `ready`
- `supported`
- `done`
- `available`
- `production-ready`

unless the statement also says which status family it belongs to.

Important rule:

```text
landed on main != published stable
```

and:

```text
landed on main != qualified limited release
```

unless the relevant release/readiness document explicitly says so.

## Future-Fit Domains

External summaries may mention these as possible fit areas only when clearly
marked as future-fit or potential applications:

- compliance-oriented execution
- regulated workflows
- financial rule validation
- medical or safety-adjacent decision support
- smart-contract-like deterministic execution
- zero-knowledge or cryptographic proof systems
- agentic AI control planes

Do not claim that Semantic currently provides certification, regulatory
compliance, formal proof of business correctness, cryptographic proof systems,
or production-grade sandbox guarantees unless a later stable release document
explicitly supports that claim.

## Numbers And Benchmarks

Do not publish numeric claims such as:

- compression ratios;
- boilerplate reduction percentages;
- speedups;
- memory savings;
- throughput;
- safety coverage percentages;
- security assurance levels;

unless the claim points to a benchmark, report, test artifact, or release note
that actually measures it.

## External Analysis Checklist

Before publishing an external analysis, check:

```text
[ ] Did I separate published stable, qualified limited, main-only, and out-of-scope behavior?
[ ] Did I avoid promoting TON618 into the current canonical execution owner?
[ ] Did I describe CTF as Core Trust Freeze?
[ ] Did I describe 7hell as a qualification/report contract, not memory mapping?
[ ] Did I avoid claiming hardware/SIMD acceleration unless a source proves it?
[ ] Did I avoid compliance/security/certification claims unless release docs prove them?
[ ] Did I avoid numeric performance/compression claims without measurement?
[ ] Did I keep UI / Workbench as operator/application layer only?
[ ] Did I keep verifier-first admission before public `.smc` execution?
```

## Recommended Correction Pattern

When an external writeup overstates the project, correct it with this shape:

```text
The high-level direction is right, but the current repository uses a narrower
canonical reading:

- Semantic is a deterministic verified execution platform, not a general
  production compliance platform.
- TON618 is retained as a non-owning compatibility perimeter, not the current
  platform owner.
- CTF means Core Trust Freeze.
- 7hell is a staged qualification/report contract.
- Current `main` may contain landed work that is not yet part of the published
  stable or qualified limited release promise.
```
