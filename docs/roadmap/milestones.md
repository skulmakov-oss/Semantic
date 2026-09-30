# Semantic Readiness Milestones

Status: historical checkpoint map, not release-verdict authority

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

For the current release-facing posture, use:

- `docs/roadmap/v1_readiness.md`
- `reports/g1_release_scope_statement.md`

## Purpose

This document records the major milestone families already landed in the
repository.

It should not be read as:

- automatic stable publication
- automatic qualification
- authority for the current release verdict

Milestones can be completed in code while still remaining only:

- `landed on main, not yet promised`
- or outside the current qualified contour

## Historical Milestone Families

- `M0 Repository Discipline`
  - process baseline
  - architecture map
  - ownership/governance conventions
  - current reading: landed baseline

- `M1 Core Contract`
  - verifier-first execution
  - runtime quotas
  - owner boundaries for core crates
  - current reading: landed baseline

- `M2 Language/Core Surface`
  - source frontend
  - type completeness work
  - SemCode family growth
  - current reading: mixed between qualified limited release and
    landed-on-`main` widenings (no line is currently evidenced as published
    stable; see `docs/roadmap/v1_readiness.md`)

- `M3 Toolchain Formalization`
  - spec bundle
  - CLI ownership
  - SemCode contract ownership
  - current reading: landed baseline

- `M4 Boundary And Runtime Layering`
  - `prom-*` owner split
  - runtime/state/rules/audit layering
  - current reading: landed baseline, with stable-vs-main commitments governed
    elsewhere

- `M5 Validation And Qualification`
  - release bundle checks
  - compatibility checks
  - Gate 1 protocol
  - Gate 1 evidence and synthesis reports
  - current reading: first Gate 1 cycle completed; current verdict is
    `qualified limited release`

- `M6 Post-Stable Language Waves`
  - schemas
  - package baseline
  - ordered sequence surface
  - iterable surface
  - closures
  - generics
  - runtime ownership
  - UI application boundary (retired with the native UI / Workbench /
    Semantic Studio contour; see
    `docs/roadmap/ui_workbench_studio_retirement.md`)
  - selected-import executable module entry
  - current reading: landed on `main`, not automatically published stable and
    not automatically qualified; the retired UI family is not awaiting
    promotion

## Operational Reading

If a milestone family is completed in code:

- check `docs/roadmap/v1_readiness.md` for the release-facing reading
- check `reports/g1_release_scope_statement.md` for the current practical
  qualification reading
- do not infer promotion from this milestone map alone
