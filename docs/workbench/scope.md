# Workbench Scope

> [!IMPORTANT]
> **Retired contour — historical record.** Native Semantic UI, Workbench, and
> Semantic Studio are retired from the active roadmap (#1968 and #1862 closed
> as not planned). This document is preserved as written for its era; any
> readiness, planning, or "proposed" status below is not an active commitment,
> and its open items are not being remediated. See
> [UI, Workbench, and Studio retirement](../roadmap/ui_workbench_studio_retirement.md).

Status: proposed v1

## Goal

Define the user-facing scope for Workbench v1 without reopening Semantic core
scope or encouraging IDE-overreach.

## Workbench v1 Includes

### Operations

- project open
- command runner
- jobs history
- command output panels

### Readiness And Release

- overview cockpit
- release status panel
- readiness and compatibility links
- bundle verification entrypoint
- asset smoke visibility

### Spec Navigation

- spec tree
- roadmap tree
- section navigation
- search over canonical document titles and paths

### Authoring Shell

- file tree
- multi-tab editor shell
- open/save/reload
- dirty markers
- current-file compile and check actions

### Diagnostics

- grouped diagnostics
- filters by family
- error-code lookup
- jump-to-file location
- links to related spec sections

### Inspection

- disasm view
- verify-result view
- trace and runtime summary view
- quota and capability summaries when present in outputs

### Tooling Integration

- formatter integration through the canonical formatter surface
- basic project bootstrap through canonical project layout commands

## Workbench v1 Excludes

- a second parser or type checker inside the UI
- direct VM or runtime embedding as an alternate execution authority
- deep source-level debugger with time-travel semantics
- private PROMETHEUS state editing
- alternate release scoring independent from repository gates
- widening narrow Semantic v1 scope for the sake of UI features

## Public Surface Rule

Workbench must use only:

- `smc`
- `svm`
- `cargo`
- public release scripts
- later, explicit public Rust facades

Workbench must not couple to private crate internals.

## Screen Inventory

The expected v1 screens are:

- overview
- project explorer
- editor shell
- diagnostics hub
- spec navigator
- inspect
- release console
- settings

## Scope Sequencing

Critical path:

1. foundation
2. cockpit
3. spec navigation
4. editor shell
5. diagnostics
6. formatter
7. inspectors
8. release console

Deferred path:

- scaffolding depth beyond baseline bootstrap
- `smlsp` bridge
- richer editor protocol features

## UX Honesty Rule

Workbench must show current repository reality, including:

- stable vs draft vs experimental status
- known limits
- release blockers
- command failures

Workbench must not mask repository limitations behind optimistic UI wording.

## Acceptance Criteria

This scope is valid when:

- included flows map to current public Semantic surfaces
- excluded flows prevent Workbench from becoming a second core
- critical-path ordering is explicit
- users can infer what Workbench does and does not promise from one document
