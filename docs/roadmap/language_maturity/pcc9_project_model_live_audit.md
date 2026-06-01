# PCC-9 Project Model v0 Live Audit

Status: PCC-9 closed / evidence synced for current admitted manifest baseline
Owner: language maturity stream
Scope: Project Model v0 readiness before PCC-9 implementation or fixture work
Non-goal: code changes

## Purpose

This document audits current Project Model v0 readiness after PCC-8 closeout.

It is docs-only and does not add project behavior.

Current repo evidence is split:

- the single-file `.sm` CLI baseline is already stable
- the admitted lower-level manifest baseline still uses `Semantic.package`
  rather than a landed `semantic.toml` project flow
- `semantic.toml` and `src/main.sm` remain roadmap-level project-model targets
- project-root `check` / `run`, `smc new`, and project fixtures are still
  follow-up work

## Current Known Status

- single-file CLI flow exists and remains the stable baseline
- project-root conventions are documented, but not yet closed as admitted
  behavior
- manifest support exists at the current package-manifest baseline
- `src/main.sm` is still a roadmap target rather than a closed project-root
  contract
- `smc new` remains a follow-up candidate, not a closed requirement
- project check/run are not yet closed
- deterministic module roots are partially evidenced through the current
  manifest baseline, but project-root policy is still open
- import/module interaction exists at the current manifest baseline, but
  project-root resolution is not yet closed
- project diagnostics are documented, but project-level diagnostic fixtures are
  still missing
- project fixtures are still missing
- docs now include a project-model contract freeze boundary, but that does not
  claim implementation
- 7hell project-level readiness is still follow-up only

## Readiness Matrix

| Layer | Required for PCC-9 | Current state | Ready? | Next action |
| --- | --- | --- | --- | --- |
| CLI | single-file check/run baseline | confirmed-working | yes | none |
| CLI | project-root check | documented-only | no | contract freeze + implementation follow-up |
| CLI | project-root run | documented-only | no | contract freeze + implementation follow-up |
| CLI | `smc new` | documented-only | no | keep as follow-up candidate |
| manifest | `semantic.toml` schema | documented-only | no | freeze contract, then implement if admitted |
| manifest | manifest parser/loading | confirmed-partial | no | keep current baseline separate from project-root work |
| layout | `src/main.sm` convention | documented-only | no | freeze contract, then implement if admitted |
| layout | deterministic source discovery | documented-only | no | define project-root discovery policy |
| modules | project-root module resolution | documented-only | no | implement or explicitly defer |
| modules | import path interaction | confirmed-partial | no | keep deterministic module-root policy narrow |
| diagnostics | missing manifest / missing main diagnostics | documented-only | no | stabilize categories in PCC-9D |
| diagnostics | invalid project layout diagnostics | documented-only | no | stabilize categories in PCC-9D |
| execution | project check pipeline | documented-only | no | add implementation or fixture evidence later |
| execution | project run pipeline | documented-only | no | add implementation or fixture evidence later |
| determinism | stable module root ordering | documented-only | no | define deterministic ordering policy |
| fixtures | positive project fixtures | documented-only | no | PCC-9C |
| fixtures | negative project diagnostics fixtures | documented-only | no | PCC-9D |
| 7hell | project-level 7hell readiness | documented-only | no | keep out of closeout |
| docs | public project model contract | confirmed-partial | partial | keep contract and audit in sync |
| examples | canonical minimal project | documented-only | no | define after contract freeze |

## PCC-9A Evidence

PCC-9A inspects the current repository state before any project-model fixture
or implementation work.

Covered evidence:

- `crates/smc-cli/src/package_manifest.rs` contains package-manifest parsing,
  validation, entry admission, and import-resolution helpers;
- `crates/smc-cli/src/package_manifest.rs` includes tests for first-wave
  manifest parsing, local dependency inventory, entry admission, and import
  resolution;
- docs describe `semantic.toml` / `src/main.sm` as the intended project
  layout;
- single-file CLI remains the stable baseline;
- no project-root `check` / `run` fixture suite exists yet.

Validation:

- `git diff --check`

PCC-9A does not add project implementation or project fixtures.
PCC-9B remains the contract freeze boundary.
PCC-9C remains positive project fixtures.
PCC-9D remains diagnostics fixtures.
PCC-9E remains closeout.

## PCC-9C Evidence

PCC-9C adds dedicated positive Project Model v0 acceptance fixtures for the
currently admitted manifest / project-adjacent baseline.

Covered positive cases:

- single-file `.sm` baseline remains stable;
- current admitted `Semantic.package` manifest baseline;
- `manifest_dir` / `module_root` baseline;
- local dependency inventory where admitted;
- entry / module admission where admitted;
- import-resolution helper baseline where admitted.

Validation:

- `cargo test --test pcc9_project_model_acceptance`
- `git diff --check`

PCC-9C follows the PCC-9B contract freeze boundary and stays on the currently
admitted baseline.
PCC-9C does not implement project-root check/run.
PCC-9C does not implement `semantic.toml`.
PCC-9C does not implement `smc new`.
PCC-9C does not add package-manager, registry, workspace, lockfile, or
dependency resolution semantics.
PCC-9D remains project diagnostics fixtures.
PCC-9E remains closeout.

## PCC-9D Evidence

PCC-9D adds dedicated diagnostics fixtures for the currently admitted project
model / manifest baseline.

Covered diagnostics cases:

- malformed current `Semantic.package` manifest input;
- missing required manifest / package field where admitted;
- invalid format / version where admitted;
- invalid dependency shape where admitted;
- entry / module admission rejection where admitted;
- import-resolution rejection where admitted.

Validation:

- `cargo test --test pcc9_project_model_diagnostics`
- `cargo test --test pcc9_project_model_acceptance`
- `git diff --check`

PCC-9D does not implement project-root check/run.
PCC-9D does not implement `semantic.toml`.
PCC-9D does not implement `smc new`.
PCC-9D does not add package-manager, registry, workspace, lockfile, or
dependency-resolution semantics.
PCC-9E remains bounded closeout.

## PCC-9E Closeout

PCC-9E closes the current admitted manifest / project-adjacent baseline for
Project Model v0.

Evidence chain:

- PCC-9A — docs audit / scope correction
- PCC-9B — minimal project layout and manifest contract freeze
- PCC-9C — positive minimal project / manifest-baseline fixtures
- PCC-9D — project-model diagnostics fixtures
- PCC-9E — bounded closeout / roadmap sync

Final verdict:

PCC-9 Project Model v0 is closed for the current admitted manifest /
project-adjacent baseline.

Evidence-backed surface:

- single-file `.sm` baseline is evidence-backed;
- current admitted `Semantic.package` manifest baseline is evidence-backed;
- `manifest_dir` / `module_root` baseline is evidence-backed;
- local dependency inventory is evidence-backed;
- entry / module admission is evidence-backed;
- import-resolution helper baseline is evidence-backed;
- manifest diagnostics are evidence-backed for the PCC-9D fixture set;
- entry path escape rejection is evidence-backed;
- unresolved dependency alias rejection is evidence-backed;
- no project-root CLI behavior was implemented by PCC-9E;
- no `semantic.toml` parser was implemented by PCC-9E;
- no `smc new` behavior was implemented by PCC-9E;
- no package manager / registry / workspace semantics were introduced.

The following are not claimed complete by PCC-9E:

- project-root `smc check <project-root>`;
- project-root `smc run <project-root>`;
- `semantic.toml` parser / loader;
- `src/main.sm` project-root discovery;
- `smc new`;
- project-level 7hell;
- package registry;
- dependency resolver;
- lockfile;
- workspace / multi-package model;
- remote packages;
- build cache redesign;
- UI / Workbench / Studio.

Acceptance checklist:

- [x] PCC-9A audit landed
- [x] PCC-9B contract freeze landed
- [x] PCC-9C positive fixtures landed
- [x] PCC-9D diagnostics fixtures landed
- [x] PCC-9E closeout evidence synced
- [x] open project-root boundaries documented
- [x] package-manager / registry / workspace exclusions preserved
- [x] no implementation changes by closeout

## Risk List

- Project Model v0 can silently become a package manager.
- `semantic.toml` can grow into dependency management too early.
- `smc new` can become template / product UX instead of a minimal project
  skeleton.
- Project-root discovery can become nondeterministic if filesystem walking is
  not bounded.
- Module roots must be deterministic and explicit.
- Import resolution must not depend on host cwd accidents.
- Project diagnostics must be stable and not path-order-dependent.
- Project-level run must not bypass verifier-first execution.
- Project-level 7hell must not become a broad release qualification gate too
  early.
- PCC-9 must not reopen stdlib, collections, or UI scope.

## Recommended PCC-9 Split

```text
PCC-9A — docs(project-model): audit Project Model v0 readiness before implementation
PCC-9B — docs(project-model): freeze minimal project layout and manifest contract
PCC-9C — test(project-model): lock positive minimal project fixtures
PCC-9D — test(project-model): lock project diagnostics fixtures
PCC-9E — docs(project-model): close PCC-9 with evidence sync and roadmap status update
```

If the audit finds missing implementation seams, propose narrow implementation
PRs between B/C/D, for example:

- `PCC-9I1 — cli(project-model): add project-root check entrypoint`
- `PCC-9I2 — cli(project-model): add project-root run entrypoint`
- `PCC-9I3 — cli(project-model): add minimal semantic.toml loader`
- `PCC-9I4 — cli(project-model): add smc new minimal skeleton`
- `PCC-9I5 — project-model: deterministic module root policy`
- `PCC-9I6 — diagnostics(project-model): stabilize project layout errors`

## Out of Scope

- package registry
- dependency resolver
- lockfile
- workspace / multi-package model
- remote packages
- version solving
- build cache redesign
- UI / Workbench / Studio
- release packaging
- stdlib expansion
- collection policy reopening
- Option / Result changes
- host ABI widening

## Acceptance Checklist

- [x] single-file CLI baseline inspected
- [x] project-root convention inspected
- [x] semantic.toml status inspected
- [x] src/main.sm convention inspected
- [x] smc new status inspected
- [x] smc check project status inspected
- [x] smc run project status inspected
- [x] deterministic module root policy inspected
- [x] import/module interaction inspected
- [x] project diagnostics inspected
- [x] project fixtures inspected
- [x] 7hell project-level readiness inspected
- [x] docs inspected
- [x] canonical examples inspected
- [x] risks documented
- [x] PCC-9 split proposed
- [x] no code changed

CTF touched: none

Reason:

`docs-only audit; no runtime value, trap, determinism, verifier, SymbolId, capability, or trace change`
