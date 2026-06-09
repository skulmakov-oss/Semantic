# Semantic Full Readiness — Non-UI Track

Status: proposed readiness track  
Scope: language/runtime/project readiness only  
Explicit exclusion: UI application boundary and Workbench work are tracked separately.

## Goal

Define the non-UI completion path that moves Semantic from a strong limited-release platform toward a fuller language readiness posture.

The readiness formula for this track is:

```text
Semantic Full Readiness =
  Stable Language Surface
+ Everyday Expressiveness
+ Minimal Standard Library
+ Project Model v0
+ Verified Execution
+ Deterministic Runtime
+ Canonical Examples
+ External Onboarding
+ Release Qualification
```

## Non-goals

This track does not include:

- Semantic-created UI applications;
- Workbench / IDE behavior;
- graphics or rendering surfaces;
- browser or mobile targets;
- macro system;
- async/concurrency;
- broad package registry;
- broad generics/traits beyond readiness needs;
- runtime/platform expansion without a separate scope decision.

## Phase FR-0 — Readiness Truth Freeze

Purpose: freeze the release/status truth before any completion work widens claims.

Tasks:

- freeze release/status vocabulary;
- sync README, Wiki-facing status, and roadmap status language;
- mark main-only features explicitly;
- verify that no docs overclaim stable readiness;
- create a final readiness matrix.

Deliverables:

- `docs/roadmap/full_readiness_non_ui.md`;
- `docs/roadmap/full_readiness_matrix.md`;
- README status block follow-up if needed.

Acceptance:

- external readers can understand the project status quickly;
- stable/current-main distinction remains honest;
- roadmap and README do not conflict;
- UI is explicitly outside this readiness track.

## Phase FR-1 — Stable Language Surface

Purpose: freeze the public source contour that Semantic promises for this readiness track.

Required contour to classify:

- `fn`, `let`, mutable locals, assignment;
- `if` / `else`, `match`, loop forms, `break`, `continue`, `return`;
- `record`, `schema`, ADT/enum surface;
- `Option`, `Result`, `Sequence`;
- import/export surface;
- function contracts;
- `quad`, `bool`, `i32`, `u32`, `f64`, `fx`, `text`, `unit`.

Acceptance:

- every public source feature has a spec entry or an explicit out-of-scope note;
- unsupported syntax produces deterministic diagnostics;
- no feature is both promised and undocumented.

## Phase FR-2 — Everyday Expressiveness

Purpose: close the practical programming layer.

Capabilities to finish or classify:

- public integer arithmetic;
- `while`;
- statement loops and control exits;
- block expression consistency;
- text usability;
- closure usability hardening;
- diagnostics for unsupported everyday patterns.

Acceptance:

- a normal stateful algorithm can be written without workaround-heavy code;
- mutable state is explicit and deterministic;
- behavior travels through source, IR, SemCode, verifier, and VM where applicable.

## Phase FR-3 — Minimal Standard Library

Purpose: define and implement a narrow standard-library baseline.

Candidate modules:

- `core` — assert, compare helpers, quad helpers;
- `math` — base numeric operations;
- `text` — concat, length, minimal formatting/to-text;
- `seq` — length, emptiness, get, contains, mutation helpers if admitted;
- `map` — get, set, contains, remove;
- `result` — helpers for `Option` / `Result`;
- `rand` — deterministic seeded pseudo-random source;
- `io` — narrow debug/stdout output.

Acceptance:

- standard library entries are explicit and documented;
- host-bound operations remain capability-aware;
- no stdlib helper bypasses verifier/runtime discipline.

## Phase FR-4 — Project Model v0

Purpose: make Semantic projects reproducible rather than just collections of files.

Candidate layout:

```text
semantic.toml
src/
  main.sm
  lib.sm
examples/
tests/
```

Tasks:

- define manifest v0;
- define package identity;
- define project entrypoint behavior;
- define examples/tests discovery;
- add CLI support for project check/run where needed;
- keep project creation for this pass grounded in manual project-root layouts.

Acceptance:

- a new project can be created manually from the documented project-root layout, then checked and run by the documented path;
- imports resolve relative to explicit project/package rules;
- the project model does not conflict with the module system.

Non-goal for this pass:

- `smc new` / project scaffolding is out of scope for the current FR-4 readiness pass.

## Phase FR-5 — Verified Execution Closure

Purpose: close verifier-first execution as a public contract.

Tasks:

- freeze opcode admission matrix;
- freeze capability manifest validation;
- freeze section integrity checks;
- freeze jump/call/register validation;
- preserve verified-only default execution path;
- add malformed SemCode rejection goldens.

Acceptance:

- standard VM execution does not run unverified SemCode;
- malformed binaries are rejected before VM execution;
- capability-gated operations have explicit admission rules.

## Phase FR-6 — Deterministic Runtime Closure

Purpose: close deterministic bounded runtime behavior.

Tasks:

- freeze runtime value set;
- freeze symbol identity model;
- freeze quota/fuel taxonomy;
- freeze trap taxonomy;
- freeze trace/audit event shape for core execution;
- add deterministic rerun and quota-exhaustion tests.

Invariant:

```text
same source
+ same compiler config
+ same SemCode
+ same runtime config
+ same capability manifest
+ same input stream
= same result / same trap / same trace class
```

## Phase FR-7 — Canonical Examples

Purpose: prove the language through small, stable programs.

Example families:

- hello/text;
- quad decision;
- records and match;
- ADT / Option / Result;
- sequence processing;
- map lookup;
- modules/imports;
- contracts;
- deterministic pseudo-random flow;
- rule/state decision;
- one benchmark-class program.

Acceptance:

- examples check/run through public CLI where applicable;
- examples are marked stable/current-main/experimental;
- at least one example is non-trivial;
- no example depends on UI.

## Phase FR-8 — External Onboarding

Purpose: make the project understandable without author assistance.

Documents:

- Getting Started;
- Language Tour;
- Semantic by Example;
- Project Model Guide;
- CLI Guide;
- Diagnostics / explain guide;
- Troubleshooting;
- Release Status.

Acceptance:

- clone, build, check example, run example path is documented;
- common errors are explained;
- docs do not require private project knowledge.

## Phase FR-9 — Release Qualification

Purpose: convert readiness into a reproducible release candidate.

Required gates:

- workspace build;
- standard tests;
- no-std checks where applicable;
- verifier tests;
- runtime gates;
- public API guard;
- boundary enforcement;
- examples smoke;
- docs consistency;
- release bundle process.

Acceptance:

- all gates pass;
- release notes are honest;
- stable/current-main distinction is preserved;
- release candidate can be reproduced.

## Dependency graph

```text
FR-0 Truth Freeze
  -> FR-1 Stable Language Surface
  -> FR-2 Everyday Expressiveness
  -> FR-3 Minimal Standard Library
  -> FR-4 Project Model v0
  -> FR-7 Canonical Examples
  -> FR-8 External Onboarding
  -> FR-9 Release Qualification

Parallel hardening:
FR-5 Verified Execution Closure
FR-6 Deterministic Runtime Closure
  -> FR-9 Release Qualification
```

## Closure statement

This non-UI readiness track is complete when Semantic can be cloned, built, learned, used to write ordinary stateful programs, verified before execution, run deterministically, tested through canonical examples, and qualified through a reproducible release gate without depending on Workbench or UI capability.
