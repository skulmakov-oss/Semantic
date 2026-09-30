# Issue 1969 Reconciliation

Baseline: revalidated against `origin/main` at `19c1fb8bcd3c03230d076f41ce830da0b673af7d`
(after #1966 and the #1967 macro PR #1973).
Scope: 2 remediation packages (REM-022, REM-027) and 34 findings from GitHub
issue 1969, plus the retirement reconciliation of the 64 findings from issue
1968.
Historical audit CSV/JSON remain immutable; this is derived current-state
evidence.

## Governance precondition

Issue #1969 was written as blocked until the #1968 macro PR merged. That
condition is superseded by an explicit owner decision: #1968 was closed **not
planned** (2026-09-30) because native Semantic UI, Workbench, and Semantic
Studio development was retired, and #1862 (Workbench native launch smoke) was
closed **not planned** for the same reason. #1969 is therefore unblocked by
architectural retirement, not by implementation completion. The decision record
is `docs/roadmap/ui_workbench_studio_retirement.md`.

Round 0 source of each historical claim: the original review thread on the
origin PR (all 34 re-read on 2026-09-30), compared with the current file on
`main`.

## REM-022 — scripts and local preflight (8)

| Finding | Historical claim (origin PR) | Round 0 classification | Current truth / action | Final disposition |
|---|---|---|---|---|
| FND-059 | `verify_release_assets.ps1` deleted the per-tag output tree before resolving `-AssetsDirectory`, destroying reusable downloaded assets (#320) | ACTIVE_CODE_GAP | Still present: `Remove-IfExists $tagOutputDirectory` ran before `Resolve-Path`. New `scripts/release_asset_output.ps1` resolves caller assets first and stages/restores them when they live inside the output tree. Tests: `release_asset_output_*` in `tests/admission_guard_scripts.rs` (mutation "delete before resolve" kills 2). | FIXED_BY_THIS_PR |
| FND-168 | local CI ended with `git diff --check`, which passes ordinary tracked modifications (#754) | ACTIVE_CODE_GAP | Still present in the default guard mode. Added `Assert-TrackedWorktreeUnchanged` (`git diff --exit-code` for worktree and index) as the `TrackedClean` gate for default, merge, and full modes. Test: `tracked_worktree_guard_rejects_unstaged_and_staged_changes` (precondition asserts `git diff --check` passes). | FIXED_BY_THIS_PR |
| FND-174 | add the nested Workbench workspace (then `apps/workbench/src-tauri`) to the fmt gate (#766) | ARCHITECTURALLY_SUPERSEDED | `apps/**` no longer exists; the nested Workbench workspace is now `examples/workbench_semantic`, excluded from the root workspace. Evidence chain: historical requirement → Workbench track retired → #1968 not planned → #1862 not planned → Workbench is not an active qualification surface. Not added back to any gate; guarded by `retired_contour_is_not_a_required_ci_or_admission_gate`. | ARCHITECTURALLY_SUPERSEDED_WITH_EVIDENCE |
| FND-184 | project-root smoke only saw the last `smc` exit code, masking `check`/`run` failures (#784) | ACTIVE_CODE_GAP | Still present. Smoke now runs through `Invoke-SmcProjectRootSmoke`, where each `smc` command is checked by `Invoke-CheckedNative`. Tests: `project_root_smoke_fails_on_check_even_when_later_commands_succeed`, `..._fails_on_verify`, `..._runs_all_five_commands_...` (mutation "no per-command check" kills 3). | FIXED_BY_THIS_PR |
| FND-187 | same masking in the `Semantic.package` baseline smoke (#792) | ACTIVE_CODE_GAP | Same root cause and fix; both smokes share the checked helper. Test: `project_root_smoke_fails_on_run_even_when_later_commands_succeed`. | FIXED_BY_THIS_PR |
| FND-190 | merge preflight always fetched `origin main` regardless of `-BaseRef` (#805) | ACTIVE_CODE_GAP | Still present. `Update-MergePreflightBaseRef` fetches `<remote>/<branch>` with an explicit refspec, warns for local refs, and fails deterministically when the base does not resolve; the worktree is created from the resolved SHA. Tests: `merge_preflight_fetches_the_selected_remote_base_ref`, `merge_preflight_rejects_an_unresolvable_base_ref` (mutation "always fetch main" kills 1). | FIXED_BY_THIS_PR |
| FND-192 | `-FullPreflight` exited before the existing guard sequence (#809) | ALREADY_FIXED_NEEDS_EVIDENCE | Fixed later: FullPreflight already ran PRReady + Readiness + LegacyAdditional + merge preflight + diff check. Anchored by moving mode composition into `Get-AdmissionGuardPlan` (the guard dispatches from it) and testing `full_and_merge_preflight_plans_are_supersets_of_the_default_guard` (mutation "Full drops a default gate" kills 1). | FIXED_LATER_CONFIRMED |
| FND-221 | whitespace-only EOF line in `.agents/skills/semantic/SKILL.md` broke `git diff --check` (#959) | ALREADY_FIXED_NEEDS_EVIDENCE | Fixed later: `git diff --check <empty-tree> HEAD -- .agents/skills/semantic/SKILL.md` reports nothing and the file ends with a single newline. Every admission mode that runs `DiffCheck` keeps guarding it. | FIXED_LATER_CONFIRMED |

## REM-027 — roadmap, maturity, and release-statement truth (26)

| Finding | Historical claim (origin PR) | Round 0 classification | Current truth / action | Final disposition |
|---|---|---|---|---|
| FND-016 | backlog/milestones pointed to a missing `ui_application_boundary_scope.md` (#216) | ALREADY_FIXED_NEEDS_EVIDENCE | The file now exists at `docs/roadmap/language_maturity/ui_application_boundary_scope.md`; it is additionally marked retired/historical by this PR. | FIXED_LATER_CONFIRMED |
| FND-057 | `limited release` declared while Q1/Q2 evidence was not fully green (#316) | ALREADY_FIXED_NEEDS_EVIDENCE | Re-evaluated against `gate1_protocol.md` Decision Rule: Q1 (`g1_real_program_trial.md`), Q2 (`g1_frontend_trust.md`), Q3, Q4, and G1-A now all record green verdicts for the admitted narrow contour, which is exactly the `limited release` condition. The verdict stays **Qualified Limited Release**; nothing is promoted to Published Stable. | FIXED_LATER_CONFIRMED |
| FND-060 | truth audit cited a machine-local checkpoint path (#325) | ACTIVE_CODE_GAP (doc) | Still present. Path redacted; the audit now states the checkpoint was never committed and names the `origin/main` SHA as the reproducible input; the document is marked historical. Removed from the hygiene allowlist so `check_repository_hygiene.py` enforces it; also guarded by `current_facing_status_docs_have_no_machine_local_paths`. | FIXED_BY_THIS_PR |
| FND-061 | authority order depended on a not-yet-existing `public_status_model.md` (#333) | ALREADY_FIXED_NEEDS_EVIDENCE | `docs/roadmap/public_status_model.md` exists and is the active vocabulary authority, so the authority order is enforceable. | FIXED_LATER_CONFIRMED |
| FND-064 | `v1_readiness.md` listed qualified selected imports as landed-not-promised (#343) | DOC_ALIGNMENT_ONLY | Still present. The landed list now names only work beyond the qualified bare/selected slice; the retired UI boundary moved out of the promotion queue. Guard: `landed_lists_do_not_relist_qualified_surfaces`. | FIXED_BY_THIS_PR |
| FND-065 | backlog listed qualified direct-record iterable dispatch as landed-only (#343) | DOC_ALIGNMENT_ONLY | Still present. Reworded to the iterable surface beyond the qualified slice; stale "published stable `v1.1.1`" wording in the same backlog aligned with `v1_readiness.md`. Same guard. | FIXED_BY_THIS_PR |
| FND-066 | required `snake_trace.sm` benchmark never scheduled (#350) | DOC_ALIGNMENT_ONLY | Still present: no `snake_trace.sm` exists. Ledger now records that the trace requirement was closed by PR-F3 (adapter contract + sample trace) and PR-F4, and that the `.sm` line is the original plan, not an outstanding deliverable. | FIXED_BY_THIS_PR |
| FND-071 | landed sequence helpers still listed as benchmark blockers (#399) | ALREADY_FIXED_NEEDS_EVIDENCE | Blocker list now reads "(none — all base-path surfaces admitted …)". | FIXED_LATER_CONFIRMED |
| FND-076 | wrong PR-D2 merge SHA `6bcb1f62` (#406) | ALREADY_FIXED_NEEDS_EVIDENCE | Ledger now records merge SHA `0eca4d60`, the commit the review identified for PR #404. | FIXED_LATER_CONFIRMED |
| FND-077 | text observation contract header targeted PR-E1 instead of PR-E1a (#410) | DOC_ALIGNMENT_ONLY | Still present. Header now names PR-E1a (landed) and explains the historical PR-E1/PR-E1a split without rewriting the frozen body. | FIXED_BY_THIS_PR |
| FND-109 | PCC truth reset introduced a competing five-label status taxonomy (#558) | DOC_ALIGNMENT_ONLY | Still present. Labels are now declared an internal PCC evidence vocabulary with an explicit mapping to the canonical status families; they never promote. | FIXED_BY_THIS_PR |
| FND-133 | PCC-4 declared closed while its DoD checklist was unchecked (#679) | ACTIVE_CODE_GAP (evidence) | Still present, and one DoD item (`run-smc`) was genuinely unevidenced. Added `run_smc_artifact` to `tests/pcc4_records_acceptance.rs`, then ticked each DoD item with its evidence. | FIXED_BY_THIS_PR |
| FND-142 | PCC-7 validation cited a nonexistent `pcc7_collections_diagnostics` test (#696) | ALREADY_FIXED_NEEDS_EVIDENCE | `tests/pcc7_collections_diagnostics.rs` now exists and runs as a test target. | FIXED_LATER_CONFIRMED |
| FND-144 | stale "no PCC-8 helper contract frozen yet" claim (#698) | ALREADY_FIXED_NEEDS_EVIDENCE | The audit now states PCC-8B froze the public helper boundary; the stale sentence is gone. | FIXED_LATER_CONFIRMED |
| FND-145 | PCC-8 CTF note still said docs-only after test fixtures were added (#700) | DOC_ALIGNMENT_ONLY | Still present. Purpose and CTF note now state PCC-8C/8D added test fixtures while no runtime/CTF surface changed. | FIXED_BY_THIS_PR |
| FND-146 | PCC-8 "closed" vs remaining `Ready? no` rows (#701) | DOC_ALIGNMENT_ONLY | Still present. Added an explicit reading: closure covers only the admitted helper surface (`yes` rows); `no` rows are the named bounded-open items and are not claimed complete. No completion was fabricated. | FIXED_BY_THIS_PR |
| FND-147 | PCC-9 contract both required an explicit entry and defaulted to `src/main.sm` (#703) | DOC_ALIGNMENT_ONLY | Still present. Aligned to the implemented rule (explicit entry or default `src/main.sm`, verified in `crates/smc-cli/src/package_manifest.rs` and its unit test). | FIXED_BY_THIS_PR |
| FND-148 | FM-041 still listed completed PCC-9 diagnostics fixtures as open (#705) | ALREADY_FIXED_NEEDS_EVIDENCE | FM-041 no longer lists diagnostics/fixtures as open; its evidence cites `tests/pcc9_project_model_diagnostics.rs`. | FIXED_LATER_CONFIRMED |
| FND-169 | maturity matrix promoted unpromoted closures/runtime ownership (#756) | ALREADY_FIXED_NEEDS_EVIDENCE | `docs/status/feature_maturity_matrix.md` no longer carries per-feature status rows; it routes to the SSF-00 matrix, where first-wave closures are "Landed and qualified on `main`" (not release-promised). | FIXED_LATER_CONFIRMED |
| FND-191 | non-canonical "Qualified application-completeness contour" status (#808) | ALREADY_FIXED_NEEDS_EVIDENCE | That status value no longer appears; benchmark evidence is described as landed and benchmark-qualified, not a release tier. | FIXED_LATER_CONFIRMED |
| FND-196 | FM-041 `confirmed-partial` row lost its exact missing edge (#862) | DOC_ALIGNMENT_ONLY | Still present. `missing_edge` now names the concrete remaining gaps (`smc new` scaffolding; workspace/remote resolution/lockfile non-goals) while keeping the row partial. | FIXED_BY_THIS_PR |
| FND-198 | matrix marked draft quota taxonomy as frozen (#864) | ALREADY_FIXED_NEEDS_EVIDENCE | The row is gone from the maturity index; the SSF-00 matrix reads quotas/fuel as "Landed but unqualified" and `docs/spec/quotas.md` remains `draft v0` — no frozen claim remains. | FIXED_LATER_CONFIRMED |
| FND-199 | Workbench readiness doc left formatter integration "deferred" (#871) | ARCHITECTURALLY_SUPERSEDED | `docs/workbench/remaining_readiness_surface.md` is a Workbench readiness plan; nothing current links it as authority. Not repaired as though Workbench were returning; marked retired/historical (banner) with the rest of `docs/workbench/**`. Evidence: #1968 and #1862 not planned; retirement decision record. | ARCHITECTURALLY_SUPERSEDED_WITH_EVIDENCE |
| FND-200 | Workbench spec-index contract grouped nested docs by parent dir (#873) | ARCHITECTURALLY_SUPERSEDED | No code implements a spec index reader (`crates`, `examples`, `src`, `tools`, `tests` searched); only a historical post-UI note references the contract. No generic non-UI indexing mechanism depends on it. Marked retired; not reorganized. | ARCHITECTURALLY_SUPERSEDED_WITH_EVIDENCE |
| FND-339 | PCC audits linked to a contributor-local Windows checkout (#1301) | ACTIVE_CODE_GAP (doc) | Still present in `collections_core_audit.md` and the sibling text/control-flow/CLI-sample audits named by the same review; replaced with repo-relative links. Other PCC audits quoting a separate local clone were redacted. All removed from the hygiene allowlist; guarded by the hygiene script and `current_facing_status_docs_have_no_machine_local_paths`. | FIXED_BY_THIS_PR |
| FND-351 | wiki project-root commands used `.` from the repo root, which has no manifest (#1505) | DOC_ALIGNMENT_ONLY | Still present. Commands now name `examples/qualification/pcc9_project_root_minimal` (and mention the `Semantic.package` baseline). | FIXED_BY_THIS_PR |

## Totals (34 / 34)

- `FIXED_BY_THIS_PR`: 18
- `FIXED_LATER_CONFIRMED`: 13
- `ARCHITECTURALLY_SUPERSEDED_WITH_EVIDENCE`: 3 (FND-174, FND-199, FND-200)
- `FALSE_POSITIVE_CONFIRMED`: 0
- unresolved or unverified: 0

Invariant: 34 expected, 34 reconciled, 0 missing, 0 duplicate (REM-022: 8,
REM-027: 26).

## UI / Workbench / Studio Retirement Reconciliation

- #1968 ([Ledger 3/4] Native UI & Workbench Closure) is closed **not planned**
  by owner decision; no #1968 branch or macro PR exists.
- #1862 (Workbench native launch smoke) is closed **not planned**.
- Native UI code was **not** repaired: this PR changes no file under
  `crates/prom-ui*/**` or `examples/workbench_semantic/**`.
- Native UI is not part of the supported future contour; Workbench is not an
  active qualification target (no CI job, admission-guard mode, or SSF phase
  depends on it); Semantic Studio is not on the active roadmap.
- Historical UI code, docs, and evidence are preserved; current-facing
  documents mark them retired instead of rewriting them.
- Self-hosting (#1910) remains the active architectural direction; this PR
  starts no self-hosting implementation.

CI/script classification of retired-contour references:

| Reference | Classification |
|---|---|
| `.github/workflows/*.yml` | no Workbench/UI job or step |
| `scripts/admission_guard.ps1`, `scripts/local_ci.ps1` | no Workbench/UI step |
| `scripts/workbench_native_launch_smoke.ps1` | historical manual script; marked retired; not invoked anywhere |
| `tools/7hell/run_ci.ps1`, `run.ps1`, `run.sh` (`sm-vm` must not depend on `prom-ui`) | generic core boundary guard — kept |
| `tests/trust_boundary_guards.rs` (`prom-cap`/`sm-vm` must not depend on `prom-ui`) | generic core boundary guard — kept |
| `prom-ui*` crates under workspace `fmt`/`clippy`/`test` | repository integrity of retained code — kept; not a product qualification |
| `scripts/check_repository_hygiene.py` allowlist for `artifacts/workbench/**`, `docs/roadmap/post_ui/**` | historical evidence exemptions — kept |

### The 64 retired #1968 findings

These findings are retired from active remediation by the architectural
decision. They are **not** fixed and must never be reported as `FIXED`.

| REM | Findings |
|---|---|
| REM-014 (15) | FND-083, FND-084, FND-085, FND-087, FND-088, FND-089, FND-090, FND-091, FND-092, FND-093, FND-306, FND-307, FND-313, FND-315, FND-332 |
| REM-015 (8) | FND-024, FND-025, FND-081, FND-094, FND-095, FND-107, FND-354, FND-355 |
| REM-016 (24) | FND-096, FND-097, FND-098, FND-099, FND-208, FND-213, FND-217, FND-219, FND-220, FND-223, FND-232, FND-271, FND-286, FND-287, FND-288, FND-289, FND-290, FND-291, FND-292, FND-293, FND-300, FND-301, FND-352, FND-353 |
| REM-017 (5) | FND-027, FND-082, FND-316, FND-317, FND-318 |
| REM-025 (12) | FND-079, FND-080, FND-100, FND-101, FND-102, FND-103, FND-104, FND-105, FND-106, FND-108, FND-356, FND-357 |

Derived-Ledger mapping: the Ledger's `Current_Status` vocabulary
(`STILL_PRESENT`, `PARTIALLY_FIXED`, `FIXED_LATER`, `OBSOLETE`,
`FALSE_POSITIVE`, `UNVERIFIED`) has no retirement state, and `OBSOLETE`
would blur "the code went away" with "the owner stopped the contour". The
derived Ledger therefore adds one explicit, non-fixed status,
`ARCHITECTURALLY_RETIRED`, used only for these 64 findings, with
`Verification_Status` left at its existing value (the defects are still
present in code), `Fix_Required = NO (retired contour)`, and evidence pointing
at #1968, #1862, the owner decision, and this reconciliation. It is excluded
from active remediation debt and is never counted as fixed.

## Qualification Contract

The PR must pass the new script regressions (`tests/admission_guard_scripts.rs`),
the governance drift guards (`tests/governance_status_drift.rs`), existing
status-drift and boundary tests, workspace tests, formatting, clippy, public API
contracts, repository hygiene, Harness scope enforcement, and `git diff --check`.
Review and merge remain separate owner-controlled phases.
