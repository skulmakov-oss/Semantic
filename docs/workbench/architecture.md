# Workbench Architecture

> [!IMPORTANT]
> **Retired contour — historical record.** Native Semantic UI, Workbench, and
> Semantic Studio are retired from the active roadmap (#1968 and #1862 closed
> as not planned). This document is preserved as written for its era; any
> readiness, planning, or "proposed" status below is not an active commitment,
> and its open items are not being remediated. See
> [UI, Workbench, and Studio retirement](../roadmap/ui_workbench_studio_retirement.md).

Status: proposed v1

## Purpose

Semantic Workbench is a desktop orchestration and presentation layer over the
existing Semantic repository contracts.

Workbench exists to make the current project, release, and authoring workflows
usable without requiring users to manually navigate the repository and run every
command from a terminal.

## Architectural Rule

Workbench is not a second compiler, verifier, VM, runtime, or release model.

Workbench owns:

- desktop shell and routes
- UI state
- command orchestration
- cached presentation models
- job history
- workspace settings

Workbench does not own:

- parser or type-checking semantics
- verifier logic
- VM logic
- ABI, capability, or gate semantics
- PROMETHEUS runtime, state, rules, or audit semantics
- release-truth calculations independent from repository commands and docs

## Source Of Truth Policy

Workbench may read and present:

- `docs/spec/*`
- `docs/roadmap/*`
- release artifacts and manifests
- test, golden, and release command outputs
- public CLI and script surfaces

Workbench must not introduce:

- a second readiness score
- a second compatibility matrix
- hidden semantic rewrites over command output
- alternate ownership maps

## Integration Rule

The first integration path is process-based:

- `smc`
- `svm`
- `cargo`
- release verification scripts

Later integration may use public Rust facades when a facade is already part of
the supported public surface. Private crate internals remain off-limits.

## Runtime Boundary Rule

Workbench must respect the existing repository ownership boundaries:

- construction contracts stay in construction crates
- execution contracts stay in execution crates
- integration contracts stay behind ABI, capability, and gate surfaces

Workbench may inspect command outputs from those layers, but it may not absorb
ownership of those layers.

## Application Modules

### `workbench-shell`

Desktop shell, routes, layout, navigation, window state.

### `workbench-core`

Job queue, event model, command dispatch, shared UI state primitives.

### `workbench-adapter-cli`

Process adapter over:

- `smc`
- `svm`
- `cargo`
- release scripts

### `workbench-spec-index`

Read-only index for spec, roadmap, readiness, compatibility, and release docs.

### `workbench-project`

Workspace open/close flow, recent projects, project-level settings, local cache
keys.

### `workbench-editor`

Editor shell only: tabs, save/reload, dirty markers, current-file actions.

### `workbench-diagnostics`

Structured diagnostics list, filters, grouping, jump targets, spec links.

### `workbench-inspector`

Disasm, verify, trace, quota, and runtime inspection views.

### `workbench-release`

Release gates, readiness, asset validation, bundle verification, known limits,
and release reports.

## Event Model

The minimal event model for v1:

- workspace opened
- file opened
- file saved
- command requested
- job started
- job finished
- diagnostics published
- release snapshot refreshed
- spec index refreshed

Events are orchestration events only. They are not semantic execution events.

## Route Map

The expected v1 route families are:

- overview
- project
- editor
- diagnostics
- spec
- inspect
- release
- settings

## Stability Labels

Workbench must surface repository maturity labels instead of flattening them.

Every workflow shown to users should be marked as one of:

- stable now
- draft target
- experimental idea

## Definition Of Done For Foundation

The foundation layer is considered valid when:

- Workbench is documented as an orchestration/UI layer, not a second core
- source-of-truth rules are explicit
- private internal coupling is forbidden
- module ownership is separated cleanly
- route families and event model are defined before implementation grows
