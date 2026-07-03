# PCC Stack External Inventory

## Status

Result: `PASS-WITH-WARNINGS`

This is `PCC-PORT-0`.

This document is audit-only.

No files were ported.
No cherry-pick was performed.
No merge was performed.
No implementation code was changed.
No tests/examples/7hell files were changed.

## Purpose

Explain that PR `#1301` is treated as an external reference and must be
decomposed before any possible transfer into `Semantic_phase1_prom_ui`.

## Source-of-truth local repo

- path: `C:\Users\said3\Desktop\EXOcode\Semantic_phase1_prom_ui`
- branch: `main`
- HEAD: `cbb54af2518943950d3be5d0ed66520a762d1a34`
- main == origin/main: `yes`
- baseline 7hell: `PASS`
- bridge audit: `FAIL`
- selective port plan: `PASS-WITH-WARNINGS`
- dirty tree: untracked local files are present
- unexpected untracked files:
  - `docs/language/semantic_sugar_track_rfc.md`
  - `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
  - `docs/roadmap/pcc/local_repo_mismatch_audit.md`
  - `docs/roadmap/pcc/pcc_stack_bridge_audit.md`
  - `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`

## External PR reference

- PR: `#1301`
- repo: `skulmakov-oss/Semantic`
- title: `docs/test(pcc): close practical core phase and sync CTF follow-ups`
- state: merged reference
- merge SHA: `736b8bb066ea68e7e6d2e79ff300f77117c51561`
- head SHA: `987857f961a2e17141490925f135849a0f6f7ef8`
- changed files: `119`
- commits: `7`
- local availability of merge SHA: `absent`
- inventory source: `partial / unavailable locally`

External claimed layers:

- PCC Practical Core closeouts
- canonical examples
- PCC candidate probes
- negative diagnostics fixtures
- negative harnesses
- 7hell wiring
- PCC / CTF sync pack
- CTF follow-up issue bodies
- Linguist readiness templates
- post-UI docs
- PR 1185 7hell platform contour audit

## Inventory summary

| Layer | File count | Proposed default decision | Notes |
|---|---:|---|---|
| PCC docs / closeouts | `5 known target files` | `NEEDS-COMPATIBILITY-AUDIT` | Exact PR file list could not be retrieved locally; closeouts are absent in this repo. |
| Canonical examples | `7 known target files` | `REBUILD-NATIVELY` | Current repo contains an older 5-example canonical pack instead of the claimed PCC stack. |
| Candidate probes | `partial` | `PORT-CANDIDATE / NEEDS-COMPATIBILITY-AUDIT` | External probe trail may be useful, but it must not be treated as canonical. |
| Negative fixtures | `21 known target files` | `REBUILD-NATIVELY` | Diagnostics must be re-probed locally; external markers are not reusable blindly. |
| Negative harnesses | `4 known target files` | `REBUILD-NATIVELY` | Harnesses must match the current local test style. |
| 7hell wiring | `4 known target files + runner docs` | `PORT LAST / NEEDS-COMPATIBILITY-AUDIT` | Only after local examples and harnesses exist and pass. |
| PCC / CTF sync docs | `6 known target files` | `REBUILD AFTER LOCAL PCC EXISTS` | Trust wording must follow local qualification, not the external claim. |
| CTF issue bodies | `6 known target files` | `PORT-CANDIDATE LATER` | Useful as execution handles only after local sync validity exists. |
| Linguist templates | `5 known target files` | `PORT-CANDIDATE` | Independent from PCC, but must not claim submit-readiness. |
| Post-UI docs | `7 known target files` | `SEPARATE TRACK` | Must not be mixed into the PCC port. |
| PR 1185 audit | `1 known target file` | `NEEDS-COMPATIBILITY-AUDIT` | Relevance needs to be checked before any transfer. |

## Per-file inventory

Exact file enumeration from GitHub CLI was unavailable in this environment:

- `gh pr view 1301 --repo skulmakov-oss/Semantic --json ...` failed due
  connectivity
- `gh pr diff 1301 --repo skulmakov-oss/Semantic --name-only` failed due
  connectivity

This inventory is therefore partial and records the known target files from the
external reference set plus local comparison anchors.

### PCC docs / closeouts

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `docs/roadmap/pcc/control_flow_core_closeout.md` | PCC docs / closeouts | absent | `NEEDS-COMPATIBILITY-AUDIT` | External closeout file is not present locally. |
| `docs/roadmap/pcc/text_core_closeout.md` | PCC docs / closeouts | absent | `NEEDS-COMPATIBILITY-AUDIT` | External closeout file is not present locally. |
| `docs/roadmap/pcc/collections_core_closeout.md` | PCC docs / closeouts | absent | `NEEDS-COMPATIBILITY-AUDIT` | External closeout file is not present locally. |
| `docs/roadmap/pcc/stdlib_v0_closeout.md` | PCC docs / closeouts | absent | `NEEDS-COMPATIBILITY-AUDIT` | External closeout file is not present locally. |
| `docs/roadmap/pcc/practical_core_phase_checkpoint.md` | PCC docs / closeouts | absent | `NEEDS-COMPATIBILITY-AUDIT` | External phase checkpoint file is not present locally. |

### Canonical examples

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `examples/canonical/match_control_flow/src/main.sm` | Canonical examples | absent | `REBUILD-NATIVELY` | Not present in this repo; would need local rebuild and local validation. |
| `examples/canonical/option_result_control_flow/src/main.sm` | Canonical examples | absent | `REBUILD-NATIVELY` | Not present in this repo; would need local rebuild and local validation. |
| `examples/canonical/loop_control_flow/src/main.sm` | Canonical examples | absent | `REBUILD-NATIVELY` | Not present in this repo; would need local rebuild and local validation. |
| `examples/canonical/text_core/src/main.sm` | Canonical examples | absent | `REBUILD-NATIVELY` | Not present in this repo; would need local rebuild and local validation. |
| `examples/canonical/collections_core/src/main.sm` | Canonical examples | absent | `REBUILD-NATIVELY` | Not present in this repo; would need local rebuild and local validation. |
| `examples/canonical/stdlib_v0_helpers/src/main.sm` | Canonical examples | absent | `REBUILD-NATIVELY` | Not present in this repo; would need local rebuild and local validation. |
| `examples/canonical/text_collections_toolbox/src/main.sm` | Canonical examples | absent | `REBUILD-NATIVELY` | Not present in this repo; would need local rebuild and local validation. |

### PCC candidate probes

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `examples/pcc_candidates/README.md` | Candidate probes | absent | `PORT-CANDIDATE` | Could be useful as an audit trail, but must stay non-canonical. |
| `examples/pcc_candidates/loop_control_flow_probe/` | Candidate probes | absent | `PORT-CANDIDATE` | Could be ported only after path/content audit. |
| `examples/pcc_candidates/option_result_control_flow/` | Candidate probes | absent | `PORT-CANDIDATE` | Could be ported only after path/content audit. |

### Negative fixtures

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `tests/fixtures/pcc/control_flow/fail/break_outside_loop.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-authored or rebuilt against the current local surface. |
| `tests/fixtures/pcc/control_flow/fail/continue_outside_loop.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-authored or rebuilt against the current local surface. |
| `tests/fixtures/pcc/control_flow/fail/if_quad_condition.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-authored or rebuilt against the current local surface. |
| `tests/fixtures/pcc/control_flow/fail/match_missing_fallback.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-authored or rebuilt against the current local surface. |
| `tests/fixtures/pcc/control_flow/fail/missing_return_path.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-authored or rebuilt against the current local surface. |
| `tests/fixtures/pcc/control_flow/fail/while_quad_condition.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-authored or rebuilt against the current local surface. |
| `tests/fixtures/pcc/text/fail/multiline_text.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/text/fail/text_ordering.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/text/fail/text_plus_bool.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/text/fail/text_plus_i32.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/text/fail/text_plus_quad.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/text/fail/to_text_record.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/collections/fail/map_iteration_unsupported.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-authored or rebuilt against the current local surface. |
| `tests/fixtures/pcc/collections/fail/map_remove_unsupported.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-authored or rebuilt against the current local surface. |
| `tests/fixtures/pcc/collections/fail/map_set_wrong_key_type.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/collections/fail/map_set_wrong_value_type.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/collections/fail/sequence_contains_wrong_type.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/collections/fail/sequence_index_wrong_type.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/collections/fail/to_text_map.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/collections/fail/to_text_sequence.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/stdlib/fail/print_bool.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/stdlib/fail/print_i32.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/stdlib/fail/print_map.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/stdlib/fail/print_quad.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/stdlib/fail/print_sequence.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/stdlib/fail/to_text_record.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/stdlib/fail/to_text_sequence.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |
| `tests/fixtures/pcc/stdlib/fail/unknown_std_namespace.sm` | Negative fixtures | absent | `REBUILD-NATIVELY` | Must be re-probed locally; external markers are not reusable blindly. |

### Negative harnesses

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `tests/pcc_control_flow_negative.rs` | Negative harnesses | absent | `REBUILD-NATIVELY` | Harness must be built against the current local fixtures and diagnostics. |
| `tests/pcc_text_negative.rs` | Negative harnesses | absent | `REBUILD-NATIVELY` | Harness must be built against the current local fixtures and diagnostics. |
| `tests/pcc_collections_negative.rs` | Negative harnesses | absent | `REBUILD-NATIVELY` | Harness must be built against the current local fixtures and diagnostics. |
| `tests/pcc_stdlib_negative.rs` | Negative harnesses | absent | `REBUILD-NATIVELY` | Harness must be built against the current local fixtures and diagnostics. |

### 7hell wiring

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `tools/7hell/run.ps1` | 7hell wiring | exists locally, but older contour | `PORT LAST / NEEDS-COMPATIBILITY-AUDIT` | Existing runner is valid baseline, but not the requested PCC negative wiring. |
| `tools/7hell/run.sh` | 7hell wiring | exists locally, but older contour | `PORT LAST / NEEDS-COMPATIBILITY-AUDIT` | Existing runner is valid baseline, but not the requested PCC negative wiring. |
| `tools/7hell/README.md` | 7hell wiring | exists locally, but older contour | `PORT LAST / NEEDS-COMPATIBILITY-AUDIT` | Needs local alignment before any port claim. |
| `docs/roadmap/pcc/7hell_mini_runner.md` | 7hell wiring | exists locally, but older contour | `PORT LAST / NEEDS-COMPATIBILITY-AUDIT` | Needs local alignment before any port claim. |

### PCC / CTF sync docs

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md` | PCC / CTF sync docs | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Sync docs cannot be claimed before a local PCC stack exists. |
| `docs/roadmap/pcc/pcc_ctf_sync_closeout.md` | PCC / CTF sync docs | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Sync docs cannot be claimed before a local PCC stack exists. |
| `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_closeout.md` | PCC / CTF sync docs | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust-lane sync wording must follow local evidence. |
| `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_runtime_value_registry.md` | PCC / CTF sync docs | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust-lane sync wording must follow local evidence. |
| `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_trap_taxonomy.md` | PCC / CTF sync docs | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust-lane sync wording must follow local evidence. |
| `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_determinism_matrix.md` | PCC / CTF sync docs | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust-lane sync wording must follow local evidence. |
| `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_capability_print_text.md` | PCC / CTF sync docs | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust-lane sync wording must follow local evidence. |
| `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_golden_trace_policy.md` | PCC / CTF sync docs | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust-lane sync wording must follow local evidence. |

### CTF issue bodies

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `docs/roadmap/issues/issue_ctf_sync_runtime_value_registry.md` | CTF issue bodies | absent | `PORT-CANDIDATE LATER` | Useful as an execution handle only after local sync validity exists. |
| `docs/roadmap/issues/issue_ctf_sync_trap_taxonomy.md` | CTF issue bodies | absent | `PORT-CANDIDATE LATER` | Useful as an execution handle only after local sync validity exists. |
| `docs/roadmap/issues/issue_ctf_sync_determinism_matrix.md` | CTF issue bodies | absent | `PORT-CANDIDATE LATER` | Useful as an execution handle only after local sync validity exists. |
| `docs/roadmap/issues/issue_ctf_sync_capability_print_text.md` | CTF issue bodies | absent | `PORT-CANDIDATE LATER` | Useful as an execution handle only after local sync validity exists. |
| `docs/roadmap/issues/issue_ctf_sync_golden_trace_policy.md` | CTF issue bodies | absent | `PORT-CANDIDATE LATER` | Useful as an execution handle only after local sync validity exists. |
| `docs/roadmap/issues/issue_ctf_sync_closeout.md` | CTF issue bodies | absent | `PORT-CANDIDATE LATER` | Useful as an execution handle only after local sync validity exists. |

### Linguist templates

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `docs/roadmap/issues/issue_linguist_semantic_readiness.md` | Linguist templates | absent | `PORT-CANDIDATE` | Independent from PCC, but must not claim submit-readiness. |
| `docs/roadmap/issues/issue_linguist_semantic_grammar_repo.md` | Linguist templates | absent | `PORT-CANDIDATE` | Independent from PCC, but must not claim submit-readiness. |
| `docs/roadmap/issues/issue_linguist_semantic_usage_evidence.md` | Linguist templates | absent | `PORT-CANDIDATE` | Independent from PCC, but must not claim submit-readiness. |
| `docs/roadmap/issues/issue_linguist_semantic_samples.md` | Linguist templates | absent | `PORT-CANDIDATE` | Independent from PCC, but must not claim submit-readiness. |
| `docs/roadmap/issues/issue_linguist_semantic_local_validation.md` | Linguist templates | absent | `PORT-CANDIDATE` | Independent from PCC, but must not claim submit-readiness. |

### Post-UI docs

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `docs/roadmap/post_ui/r12_ui_draw_backend_selection_boundary.md` | Post-UI docs | absent locally | `SEPARATE TRACK` | Must not be mixed with PCC transfer. |
| `docs/roadmap/post_ui/r12_ui_intent_admission_and_dispatch_source_closeout.md` | Post-UI docs | absent locally | `SEPARATE TRACK` | Must not be mixed with PCC transfer. |
| `docs/roadmap/post_ui/r12_ui_interaction_pipeline_integration_source_closeout.md` | Post-UI docs | absent locally | `SEPARATE TRACK` | Must not be mixed with PCC transfer. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_reality_reconciliation.md` | Post-UI docs | absent locally | `SEPARATE TRACK` | Must not be mixed with PCC transfer. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_renderer_reality_audit.md` | Post-UI docs | absent locally | `SEPARATE TRACK` | Must not be mixed with PCC transfer. |
| `docs/roadmap/post_ui/r12_ui_renderer_boundary.md` | Post-UI docs | absent locally | `SEPARATE TRACK` | Must not be mixed with PCC transfer. |
| `docs/roadmap/post_ui/r12_ui_windowing_boundary.md` | Post-UI docs | absent locally | `SEPARATE TRACK` | Must not be mixed with PCC transfer. |

### PR 1185 audit

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `docs/roadmap/pcc/pr_1185_ci_7hell_platform_contour_audit.md` | PR 1185 audit | absent | `NEEDS-COMPATIBILITY-AUDIT` | Relevance needs to be checked before any transfer. |

### Current local comparison anchors

These files exist locally and are part of the current baseline. They are not the
claimed PR `#1301` stack, but they are useful comparison anchors.

| File | Layer | Current local state | Classification | Reason |
|---|---|---|---|---|
| `examples/canonical/cli_batch_core/src/main.sm` | Canonical examples baseline | exists | `EXTERNAL-REFERENCE-ONLY` | Current baseline anchor, not the claimed PCC stack. |
| `examples/canonical/rule_state_decision/src/main.sm` | Canonical examples baseline | exists | `EXTERNAL-REFERENCE-ONLY` | Current baseline anchor, not the claimed PCC stack. |
| `examples/canonical/data_audit_record_iterable/src/main.sm` | Canonical examples baseline | exists | `EXTERNAL-REFERENCE-ONLY` | Current baseline anchor, not the claimed PCC stack. |
| `examples/canonical/wave2_local_helper_import/src/main.sm` | Canonical examples baseline | exists | `EXTERNAL-REFERENCE-ONLY` | Current baseline anchor, not the claimed PCC stack. |
| `examples/canonical/positive_selected_import/src/main.sm` | Canonical examples baseline | exists | `EXTERNAL-REFERENCE-ONLY` | Current baseline anchor, not the claimed PCC stack. |
| `examples/canonical/boundary_alias_import/src/main.sm` | Canonical examples baseline | exists | `EXTERNAL-REFERENCE-ONLY` | Current baseline anchor, not the claimed PCC stack. |

## Do-not-port-as-monolith rule

PR `#1301` must not be cherry-picked, merged, or patch-applied as a single
unit.

## Required compatibility before future transfer

For any future port candidate, require:

1. local path check;
2. current architecture check;
3. local syntax/admitted surface check;
4. local diagnostics marker check for negative fixtures;
5. local test harness compatibility;
6. local 7hell compatibility;
7. no readiness claim before local validation.

## Layer-specific next actions

### Linguist templates

Decision:

- likely `PORT-CANDIDATE`

Required next action:

- verify no submit-readiness claim;
- verify the usage-evidence blocker remains explicit.

### Candidate probes

Decision:

- `PORT-CANDIDATE` or `NEEDS-COMPATIBILITY-AUDIT`

Required next action:

- import only as candidate / probe, never canonical;
- run local `smc check` if later ported.

### Canonical examples

Decision:

- `REBUILD-NATIVELY`

Required next action:

- introduce first as probes;
- promote only after local checks.

### Negative fixtures

Decision:

- `REBUILD-NATIVELY`

Required next action:

- re-probe diagnostics locally;
- do not reuse external markers blindly.

### Negative harnesses

Decision:

- `REBUILD-NATIVELY`

Required next action:

- build only after local fixture markers are confirmed.

### 7hell wiring

Decision:

- `PORT LAST`

Required next action:

- wire only after local tests exist and pass.

### PCC / CTF sync docs

Decision:

- `REBUILD AFTER LOCAL PCC EXISTS`

Required next action:

- do not claim CTF sync until local PCC stack exists.

### Post-UI docs

Decision:

- `SEPARATE TRACK`

Required next action:

- do not mix with the PCC stack port.

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Blind cherry-pick of incompatible stack | high | Prohibit monolithic import. |
| Canonical examples not admitted locally | high | Probe before canonical promotion. |
| Negative diagnostics markers differ locally | high | Re-run local marker audit. |
| CTF sync claimed before PCC exists locally | high | Rebuild sync only after local evidence. |
| Linguist readiness overclaimed | medium | Preserve blocker wording. |
| Post-UI mixed into PCC port | medium | Separate track. |

## Recommended next step

Recommended next step:

```text
PCC-PORT-1: external file diff sampling
```

Goal:

- sample representative files from each layer;
- compare against current local architecture;
- decide the first small safe slice.

No transfer should happen before `PCC-PORT-1`.

## Final verdict

One of:

- `PASS`
- `PASS-WITH-WARNINGS`
- `BLOCKED`

Expected:

```text
PASS-WITH-WARNINGS
```

Reason:

- inventory can be created;
- PR `#1301` remains external;
- exact file-level compatibility still requires deeper audit.
