# PCC-9 Project Model v0 Live Audit

Status: live audit
Owner: language maturity stream
Scope: Project Model v0 readiness before PCC-9 implementation or fixture work
Non-goal: code changes

## 1. Purpose

This document audits current Project Model v0 readiness after PCC-8 closeout.

It is docs-only and does not add project behavior.

## 2. Current Known Status

Current `main` has evidence for a package-manifest baseline and import
resolution helpers in `smc-cli`, but it does not yet evidence a closed
project-root workflow.

What is currently evidenced:

- single-file `.sm` operation works and remains the baseline CLI path;
- package-manifest parsing and validation exist in `crates/smc-cli/src/package_manifest.rs`;
- `admit_package_entry_module` and `resolve_package_import_path` exist as
  project-adjacent helpers;
- package-manifest parsing and module-admission tests exist for the manifest
  baseline and local import resolution;
- docs already describe canonical project layout ideas such as
  `semantic.toml` / `src/main.sm`.

What is not yet evidenced as a closed Project Model v0 surface:

- project-root `check` as a first-class admitted CLI path;
- project-root `run` as a first-class admitted CLI path;
- `smc new` for project skeleton creation;
- deterministic module-root policy at the project level;
- project-level diagnostics fixtures;
- project-level 7hell readiness;
- a minimal project contract frozen as a public project-model spec.

This means PCC-9 is still an audit, not a closeout.

## 3. Readiness Matrix

| Layer       | Required for PCC-9                          | Current state | Ready? | Next action |
| ----------- | ------------------------------------------- | ------------- | ------ | ----------- |
| CLI         | single-file check/run baseline              | confirmed-working | yes | keep the single-file baseline stable |
| CLI         | project-root check                          | unknown | no | audit the first-class project entrypoint before implementation |
| CLI         | project-root run                            | unknown | no | audit the first-class project entrypoint before implementation |
| CLI         | `smc new`                                   | unknown | no | keep as explicit follow-up unless admitted by roadmap policy |
| manifest    | `semantic.toml` schema                      | confirmed-partial | no | freeze the minimal manifest contract before any project expansion |
| manifest    | manifest parser/loading                     | confirmed-partial | no | keep parser/validation evidence bounded to the baseline package manifest surface |
| layout      | `src/main.sm` convention                    | documented-only | no | keep as roadmap intent until project-root behavior is evidenced |
| layout      | deterministic source discovery              | unknown | no | audit whether discovery is explicit and stable before claiming support |
| modules     | project-root module resolution              | confirmed-partial | no | keep import resolution bounded to the package-manifest baseline |
| modules     | import path interaction                     | confirmed-partial | no | preserve current baseline and avoid claiming project-root completion |
| diagnostics | missing manifest / missing main diagnostics | unknown | no | name the missing-edge diagnostics before implementation widening |
| diagnostics | invalid project layout diagnostics          | unknown | no | keep diagnostics stable only once project layout is admitted |
| execution   | project check pipeline                      | unknown | no | do not mark complete without end-to-end project fixtures |
| execution   | project run pipeline                        | unknown | no | do not mark complete without end-to-end project fixtures |
| determinism | stable module root ordering                 | unknown | no | audit module-root ordering before implementation |
| fixtures    | positive project fixtures                   | unknown | no | add only after the minimal contract is frozen |
| fixtures    | negative project diagnostics fixtures       | unknown | no | add only after the minimal contract is frozen |
| 7hell       | project-level 7hell readiness               | unknown | no | keep as explicit follow-up, not implied readiness |
| docs        | public project model contract               | confirmed-partial | no | freeze the minimal project layout and manifest boundary before adding fixtures |
| examples    | canonical minimal project                   | documented-only | no | keep examples descriptive until project behavior is admitted |

## 4. PCC-9A Evidence

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
PCC-9B remains the minimal contract freeze.
PCC-9C remains positive project fixtures.
PCC-9D remains diagnostics fixtures.
PCC-9E remains closeout.

## 5. Risk List

Include at least:

- Project Model v0 can silently become a package manager.
- `semantic.toml` can grow into dependency management too early.
- `smc new` can become template / product UX instead of a minimal project skeleton.
- Project-root discovery can become nondeterministic if filesystem walking is not bounded.
- Module roots must be deterministic and explicit.
- Import resolution must not depend on host cwd accidents.
- Project diagnostics must be stable and not path-order-dependent.
- Project-level run must not bypass verifier-first execution.
- Project-level 7hell must not become a broad release qualification gate too early.
- PCC-9 must not reopen stdlib, collections, or UI scope.

## 6. Recommended PCC-9 Split

Default split:

```text
PCC-9A — docs(project-model): audit Project Model v0 readiness before implementation
PCC-9B — docs(project-model): freeze minimal project layout and manifest contract
PCC-9C — test(project-model): lock positive minimal project fixtures
PCC-9D — test(project-model): lock project diagnostics fixtures
PCC-9E — docs(project-model): close PCC-9 with evidence sync and roadmap status update
```

If the audit finds missing implementation seams, propose narrow implementation
PRs between B/C/D, for example:

```text
PCC-9I1 — cli(project-model): add project-root check entrypoint
PCC-9I2 — cli(project-model): add project-root run entrypoint
PCC-9I3 — cli(project-model): add minimal semantic.toml loader
PCC-9I4 — cli(project-model): add smc new minimal skeleton
PCC-9I5 — project-model: deterministic module root policy
PCC-9I6 — diagnostics(project-model): stabilize project layout errors
```

Do not add implementation work in PCC-9A itself.

## 7. Out of Scope

Explicitly list:

- package registry;
- dependency resolver;
- lockfile;
- workspace / multi-package model;
- remote packages;
- version solving;
- build cache redesign;
- UI / Workbench / Studio;
- release packaging;
- stdlib expansion;
- collection policy reopening;
- Option / Result changes;
- host ABI widening.

## 8. Acceptance Checklist

```markdown
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
```

## 9. CTF Note

Because this is docs-only:

`CTF touched: none`

Reason:

`docs-only audit; no runtime value, trap, determinism, verifier, SymbolId,
capability, or trace change`
