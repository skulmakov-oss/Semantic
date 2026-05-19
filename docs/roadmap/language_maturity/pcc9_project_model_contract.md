# PCC-9 Project Model v0 Contract

Status: contract freeze
Owner: language maturity stream
Scope: minimal project layout and manifest boundary before PCC-9 implementation / fixture work
Non-goal: package manager, registry, workspace, dependency resolver

## Purpose

This document freezes the minimal Project Model v0 contract before PCC-9C and
PCC-9D fixtures and before any implementation seam is reopened.

It does not add behavior.

Current repo evidence note:

- the current admitted lower-level manifest baseline still uses
  `Semantic.package`
- that baseline exposes `format`, `package`, `manifest_dir`, `module_root`,
  and `dep`
- `semantic.toml` is the PCC-9 project-model naming target, not a landed
  implementation claim

## Minimal Project Layout

The intended minimal project layout is:

```text
project-root/
  semantic.toml
  src/
    main.sm
```

Contract rules:

- `project-root` is the directory containing the project manifest
- `src/main.sm` is the default entry source
- `semantic.toml` is the project manifest target name for PCC-9
- other source files may exist under `src/`, but PCC-9B does not define broad
  package or workspace discovery
- file ordering must be deterministic wherever multiple source files are
  considered
- host current working directory must not silently change semantics

## Minimal Manifest Shape

This is the intended minimal manifest shape as contract intent:

```toml
[package]
name = "example"
version = "0.1.0"

[project]
entry = "src/main.sm"
```

Current admitted baseline evidence uses the existing package-manifest parser
surface instead:

- `format <u32>`
- `package <name>`
- `manifest_dir <path>`
- `module_root <path>`
- `dep <alias> <package_name> <local_path>`

Contract rules:

- package name is required if the admitted parser requires it; current baseline
  does
- project entry is required for project-root check/run
- entry path must be relative to project root
- entry path must not escape project root
- path normalization must be deterministic
- no dependency resolver is introduced
- no lockfile is introduced
- no remote package source is introduced

## Project Command Boundary

Freeze the intended command semantics without implementing them:

```text
smc check <project-root>
smc run <project-root>
```

Rules:

- project-root commands must resolve the manifest
- project-root commands must load the manifest
- project-root commands must resolve the entry path
- project-root commands must run the existing verifier-first pipeline
- project-root commands must produce stable diagnostics for invalid layout
- project-root commands must not bypass single-file behavior
- project-root commands must not bypass verifier-first execution

Also state:

- single-file behavior remains valid:
  - `smc check file.sm`
  - `smc run file.sm`
- project-root behavior is an additional admitted path, not a replacement for
  the single-file baseline

## `smc new` Boundary

`smc new <name>` is a PCC-9 follow-up candidate, not required by this contract
freeze.

If it is later admitted, it must create only the minimal skeleton:

- `semantic.toml`
- `src/main.sm`

It must not add:

- templates beyond a minimal checkable project
- a package registry
- dependency fetch
- UI scaffolding
- Workbench scaffolding

## Deterministic Module Root Policy

Freeze the module-root boundary:

- project root is the manifest directory
- entry module is explicit from the manifest or defaults to `src/main.sm`
- import resolution must be relative to deterministic project and module roots
- import resolution must not depend on accidental process cwd
- directory traversal must be sorted deterministically if used
- symlink behavior is not widened unless already defined
- path escape outside project root must be rejected or explicitly out of scope

## Diagnostics Boundary

The intended stable diagnostics categories are:

- missing `semantic.toml`
- malformed `semantic.toml`
- missing `src/main.sm` or missing entry
- entry path escapes project root
- entry path points to a directory
- invalid project root
- unsupported workspace or dependency fields
- invalid import path relative to project root

Do not invent exact error codes unless they already exist.

Use diagnostic category names and leave exact codes to PCC-9D if they are not
frozen yet.

## Public / Internal / Deferred Classification

| Item | PCC-9 v0 status | Boundary |
| --- | --- | --- |
| single-file `.sm` check/run | existing baseline | remains supported |
| `semantic.toml` | public project manifest target | minimal fields only |
| `src/main.sm` | public default entry target | project-root default |
| `smc check <project-root>` | target public command | implementation and fixtures later |
| `smc run <project-root>` | target public command | implementation and fixtures later |
| `smc new` | optional / follow-up | minimal skeleton only if admitted |
| package registry | out of scope | no remote packages |
| dependency resolver | out of scope | no version solving |
| lockfile | out of scope | no lockfile v0 |
| workspace model | out of scope | no multi-package workspace |
| build cache redesign | out of scope | no cache changes |
| Workbench / UI | out of scope | no UI scaffolding |

## Follow-Up Split

```text
PCC-9C — test(project-model): lock positive minimal project fixtures
PCC-9D — test(project-model): lock project diagnostics fixtures
PCC-9E — docs(project-model): close PCC-9 with evidence sync and roadmap status update
```

Possible narrow implementation seams after the freeze:

- `PCC-9I1 — cli(project-model): add project-root check entrypoint`
- `PCC-9I2 — cli(project-model): add project-root run entrypoint`
- `PCC-9I3 — cli(project-model): add minimal semantic.toml loader`
- `PCC-9I4 — cli(project-model): add smc new minimal skeleton`
- `PCC-9I5 — project-model: deterministic module root policy`
- `PCC-9I6 — diagnostics(project-model): stabilize project layout errors`

Do not implement these in PCC-9B.
