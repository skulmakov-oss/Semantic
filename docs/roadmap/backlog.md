# Semantic Readiness Backlog

Status: release-maintenance backlog for the current repository truth

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

This backlog is not a promise to open a new language track by default.
It exists to keep the stable-line reading, the qualified limited-release
reading, and current-`main` reality aligned without silent scope widening.

## Current Release-Control Wave

- keep the stable-line reading honest: there is no currently evidenced
  published stable line, and the `v1.1.1` tag remains an unresolved
  stable-tag checkpoint (`docs/roadmap/v1_readiness.md`)
- keep the current practical-programming verdict honest:
  `qualified limited release`, not `public release`
- keep current-`main` landed widenings described as landed, not silently
  promoted

## Current Required Maintenance Wave

- keep `README.md`, `milestones.md`, `v1_readiness.md`,
  `compatibility_statement.md`, and `stable_release_policy.md` aligned with:
  - `docs/roadmap/public_status_model.md`
  - `reports/g1_release_scope_statement.md`
  - the actual stable-line evidence (tags, GitHub Releases, validated assets)
- keep release bundle guidance and smoke validation aligned with the current
  stable asset story
- keep qualification reports and release-facing docs in sync after each
  completed Gate amendment or rerun

## Current Practical-Programming Reading

Qualified limited-release contour currently includes:

- single-file executable programs on the admitted source surface
- narrow helper-module executable programs through direct local-path bare
  imports
- narrow helper-module executable programs through direct local-path selected
  imports over function-only helper modules
- rule/state-oriented programs over records, `quad`, and explicit
  `Option` / `Result`
- built-in `Sequence(T)` iteration
- direct-record user-defined `Iterable` dispatch
- verified execution through the admitted
  `source -> sema -> IR -> SemCode -> verifier -> VM` path

Still explicitly outside the current qualified contour:

- broader executable-module authoring beyond the admitted bare/selected slice
- full CLI application authoring with admitted argv/stdout/file IO
- broader generalized iterable dispatch
- native UI, Workbench, and Semantic Studio, which are retired
  (`docs/roadmap/ui_workbench_studio_retirement.md`)

## Landed On `main`, Not Yet Promised

Current `main` contains widened surfaces beyond the qualified contour.
These remain landed and unpromoted unless a later explicit decision qualifies
or publishes them.

High-signal landed post-stable families include:

- schema/boundary-core work
- package baseline work
- ordered sequence surface
- iterable surface beyond the qualified built-in `Sequence(T)` iteration and
  direct-record `Iterable` dispatch
- first-wave closures
- first-wave generics
- runtime ownership for tuple + direct record-field paths
- module and import work beyond the qualified direct local-path bare/selected
  helper-module slice

The first-wave UI application boundary is landed on `main` but retired with
the native UI / Workbench / Semantic Studio contour; it is not awaiting
promotion.

## Default Rule For New Work

- do not open a new feature track by inertia
- do not treat landed-on-`main` behavior as automatically release-promised
- if broader practical-programming widening is desired, require:
  - a new explicit scope decision
  - and a Gate amendment or new qualification cycle

The application-completeness benchmark track
(`docs/roadmap/application_completeness_pr_ledger.md`) is closed by PR-F4.

The active strategic direction is Semantic self-hosting
([#1910](https://github.com/skulmakov-oss/Semantic/issues/1910)); it is an
architecture track, not a release promise, and it does not depend on any UI.

## Execution Rule

- one PR = one logical step
- docs/spec/tests/report truth must move together
- no silent scope movement
- merge only on green local validation where applicable and green CI
