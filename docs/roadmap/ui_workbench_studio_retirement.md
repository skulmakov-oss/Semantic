# Native UI, Workbench, and Semantic Studio Retirement

Status: active governance decision (current-facing)
Decision recorded: 2026-09-30
Decision owner: repository owner
Evidence: [#1968](https://github.com/skulmakov-oss/Semantic/issues/1968)
(closed **not planned**, with the owner's closure decision comment),
[#1862](https://github.com/skulmakov-oss/Semantic/issues/1862) (closed
**not planned**), reconciled by
[#1969](https://github.com/skulmakov-oss/Semantic/issues/1969)
Active strategic direction:
[#1910 Semantic Self-Hosting Foundation](https://github.com/skulmakov-oss/Semantic/issues/1910)

## Decision

Semantic no longer owns a native UI product roadmap.

Native Semantic UI, Workbench, and Semantic Studio are **retired** from the
active roadmap and from the supported future contour. Historical code and
evidence are preserved, but the contour is not being remediated or promoted.
Its unresolved historical findings are not represented as fixed; they are
retired from active remediation through this explicit architectural decision.

Retired is not the same as completed:

| Reading | True for the retired contour? |
|---|---|
| defects fixed | **No.** Known historical defects remain as they were. |
| stable / published | **No.** It never reached a stable or qualified release. |
| completed | **No.** Development stopped; it did not finish. |
| promoted | **No.** Nothing in it is release-promised. |
| on the active roadmap | **No.** No phase, gate, or milestone depends on it. |
| erased from history | **No.** Code, docs, and evidence stay in the repository. |

This decision describes the current roadmap. It does not claim that a UI can
never return: any future UI work would need a new explicit owner decision and
would start as a new track, not as a resumption of this contour.

## Retired contour

| Family | Repository locations | Current reading |
|---|---|---|
| Native Semantic UI | `crates/prom-ui`, `crates/prom-ui-runtime`, `crates/prom-ui-backend-native`, `crates/prom-ui-iced-adapter`, `crates/prom-ui-demo`, `experiments/ui-shell-kit` | Landed on `main`, unqualified, retired. Crates remain workspace members and keep compiling and testing as repository integrity; they are not repaired. |
| Workbench | `examples/workbench_semantic`, `docs/workbench/**`, `docs/roadmap/workbench_spec_index_reader_v0.md`, `scripts/workbench_native_launch_smoke.ps1`, `artifacts/workbench/**` | Historical evidence only. Excluded from the root workspace; not an active qualification surface. |
| Semantic Studio | no code; roadmap references only | Retired product direction; was already a Stable Foundation non-goal. |
| UI-DNA / post-UI roadmap | `docs/roadmap/post_ui/**`, `tools/post_ui/**`, `docs/spec/ui/**`, `docs/architecture/ui_*.md`, `docs/roadmap/language_maturity/ui_application_boundary_scope.md` | Historical design and audit record. Not an active roadmap or contract commitment. |

The first-wave UI application boundary described in older roadmap documents
(milestone M6/M7, WBS `1.8`) is part of this retired contour.

## What stays in force

Retirement does not weaken any generic repository rule:

- the dependency and boundary guards that keep `prom-ui*` out of compiler,
  verifier, and VM crates (`tests/trust_boundary_guards.rs`,
  `tools/7hell/run_ci.ps1`) remain active; they protect the core, not the UI;
- UI crates stay under the workspace-wide `fmt`, `clippy -D warnings`, and
  `cargo test --workspace` gates for as long as they remain workspace members;
- the verifier-first, capability, and determinism rules in `CONSTRAINTS.md`
  still apply to any code that remains in the repository;
- the rule that UI is a projection and never language, verifier, runtime, or
  Foundation authority remains true of the historical code.

## Qualification boundary

- No CI job, admission-guard mode, release gate, or Stable Foundation phase
  requires a Workbench or native UI launch.
- `scripts/workbench_native_launch_smoke.ps1` is a historical manual script;
  its failure against the current root workspace (#1862) is recorded and is
  not repaired.
- The nested Workbench workspace is not added to the formatting gate
  (historical request FND-174); it is outside active qualification.
- Retired code must not block core or self-hosting qualification by being
  treated as an active release promise.

## Historical findings

The 64 historical review findings grouped in #1968 (REM-014, REM-015,
REM-016, REM-017, REM-025) are retired from active remediation by this
decision. They are **not** fixed. The immutable historical audit keeps them as
historical facts; the derived Health Ledger records them with an explicit
architectural-retirement disposition, never as `FIXED`. See
`reports/security/ISSUE_1969_RECONCILIATION.md`.

## Strategic direction

The active architectural direction is Semantic self-hosting (#1910):

```text
Semantic Foundation
        ↓
Bootstrap capability
        ↓
Semantic compiler in Semantic
        ↓
C0 → C1 → C2 reproducible fixed point
```

Initial self-hosting keeps the existing verifier, VM, and host boundary in
Rust; it does not require any UI. Presentation for future Semantic
applications may be provided externally (for example by consuming
deterministic traces, as the snake benchmark's trace adapter contract already
does) and is not part of the current self-hosting requirements.

Current-facing documents must not describe a mandatory
`Foundation → Workbench → Studio` sequence.
