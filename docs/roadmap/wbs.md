# Semantic v1 WBS Summary

Program:

- `1.0 Semantic v1`

Milestones:

- `1.1` repository discipline
- `1.2` core contract
- `1.3` language completion
- `1.4` platform formalization
- `1.5` PROMETHEUS boundary
- `1.6` semantic runtime
- `1.7` v1 lockdown
- `1.8` UI application boundary (post-stable; retired, see
  `docs/roadmap/ui_workbench_studio_retirement.md`)

Current post-stable focus:

- keep the stable-line reading honest while current `main` moves forward
  (`v1.1.1` is an unresolved stable-tag checkpoint, not an evidenced published
  stable line; see `docs/roadmap/v1_readiness.md`)
- treat post-stable widening as explicit tracked streams rather than silent drift
- keep roadmap/spec/release-facing docs aligned with actual owner layers on `main`

Current non-blocking follow-up work:

- the former post-stable UI application boundary track
  (`docs/roadmap/language_maturity/ui_application_boundary_scope.md`) is
  retired together with native UI, Workbench, and Semantic Studio; it is kept
  as historical evidence and is not an active follow-up
  (`docs/roadmap/ui_workbench_studio_retirement.md`)
- the active strategic direction is Semantic self-hosting
  ([#1910](https://github.com/skulmakov-oss/Semantic/issues/1910))
- the retained non-owning TON618 compatibility perimeter is frozen as completed
  post-stable baseline history in
  `docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md`
- the first-wave PROMETHEUS host-call expansion is frozen as completed
  post-stable baseline history in
  `docs/roadmap/language_maturity/prometheus_host_call_expansion_scope.md`
- deepen runtime semantics only after `v1` scope is frozen
