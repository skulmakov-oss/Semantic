# Workbench View Models

> [!IMPORTANT]
> **Retired contour — historical record.** Native Semantic UI, Workbench, and
> Semantic Studio are retired from the active roadmap (#1968 and #1862 closed
> as not planned). This document is preserved as written for its era; any
> readiness, planning, or "proposed" status below is not an active commitment,
> and its open items are not being remediated. See
> [UI, Workbench, and Studio retirement](../roadmap/ui_workbench_studio_retirement.md).

Status: proposed v1

## Purpose

Define the presentation models Workbench may cache and render without becoming a
second semantic authority.

View models are derived from repository files, command outputs, and public tool
surfaces. They are never the canonical source of truth.

## `OverviewViewModel`

Derived from:

- git branch and commit
- baseline tags
- latest test/build/release command results
- readiness and compatibility docs

Fields:

- current branch
- current commit
- baseline tag
- latest workspace test status
- latest release build status
- latest bundle verification status
- latest asset smoke summary
- known-limits summary

Must not contain:

- invented readiness percentages
- manual override flags that replace command truth

## `ProjectViewModel`

Derived from:

- selected workspace root
- recent-project cache
- local settings
- repository file tree

Fields:

- workspace root
- recent projects
- open files
- dirty files
- workspace settings

Must not contain:

- alternate package metadata semantics

## `JobViewModel`

Derived from:

- requested command
- process execution metadata
- stdout/stderr
- exit code
- parsed diagnostics references when available

Fields:

- job id
- command kind
- arguments
- start time
- finish time
- duration
- status
- exit code
- output
- related file

Must not contain:

- hidden semantic rewrites over output

## `DiagnosticsViewModel`

Derived from:

- parser diagnostics
- type diagnostics
- module/import/export diagnostics
- verifier diagnostics
- runtime failures

Fields:

- family
- severity
- code
- message
- file path
- start/end location
- related spec link

Must preserve:

- error code
- location
- severity
- original message text or faithful rendering

## `SpecDocumentViewModel`

Derived from:

- `docs/spec/*`
- `docs/roadmap/*`
- selected path and heading anchors

Fields:

- path
- title
- section headings
- last refreshed time
- stability label when declared by the document

Must not contain:

- silently edited mirror content

## `InspectorViewModel`

Derived from:

- `svm disasm`
- verify outputs
- trace outputs
- quota and capability summaries

Fields:

- artifact path
- disasm text
- verify summary
- trace summary
- runtime summary
- quota summary
- capability summary

Must not contain:

- a second VM interpretation layer

## `ReleaseViewModel`

Derived from:

- readiness docs
- compatibility docs
- release checklist
- smoke matrix
- bundle verification output
- latest validation jobs

Fields:

- gate statuses
- artifact list
- smoke status
- docs alignment notes
- known limits
- release-valid summary

Rule:

`release-valid` may be computed only from real gates and document states already
defined by the repository.

## `SettingsViewModel`

Derived from:

- local user settings
- selected workspace settings

Fields:

- default workspace
- shell preferences
- formatter preferences
- display preferences

Must not contain:

- hidden feature switches that widen Semantic scope

## View-Model Discipline

All Workbench view models must follow three rules:

1. derived, not canonical
2. explainable from public inputs
3. replaceable by refresh from repository state
