# PCC Stack External Diff Sampling Captured

## Status

Result: `PASS-WITH-WARNINGS`

This is `PCC-PORT-1C`.

This document is audit-only.

No files were ported.
No cherry-pick was performed.
No merge was performed.
No patch was applied.
No implementation code was changed.
No tests/examples/7hell files were changed.

## Purpose

Explain that this document captures real external file list / diff sampling
evidence for PR `#1301` before any possible transfer into
`Semantic_phase1_prom_ui`.

## Source-of-truth local repo

- path: `C:\Users\said3\Desktop\EXOcode\Semantic_phase1_prom_ui`
- branch: `main`
- HEAD: `cbb54af2518943950d3be5d0ed66520a762d1a34`
- main == origin/main: `yes`
- baseline 7hell: `PASS`
- bridge audit: `FAIL`
- selective port plan: `PASS-WITH-WARNINGS`
- external inventory: `PASS-WITH-WARNINGS`
- external diff sampling: `BLOCKED`
- retry plan: `PASS-WITH-WARNINGS`
- dirty tree: untracked local files are present
- unexpected untracked files:
  - `docs/language/semantic_sugar_track_rfc.md`
  - `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
  - `docs/roadmap/pcc/local_repo_mismatch_audit.md`
  - `docs/roadmap/pcc/pcc_stack_bridge_audit.md`
  - `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
  - `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
  - `docs/roadmap/pcc/pcc_stack_external_inventory.md`
  - `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`
  - `docs/roadmap/pcc/external/pr_1301_changed_files.txt`

## External PR reference

- PR: `#1301`
- repo: `skulmakov-oss/Semantic`
- title: `docs/test(pcc): close practical core phase and sync CTF follow-ups`
- state: `MERGED`
- merge SHA: `736b8bb066ea68e7e6d2e79ff300f77117c51561`
- head SHA: `987857f961a2e17141490925f135849a0f6f7ef8`
- changed files: `119`
- commits: `7`
- local availability of merge SHA: `absent`
- captured file list: `docs/roadmap/pcc/external/pr_1301_changed_files.txt`
- sampling source: `gh pr diff` / `gh pr view` / representative `gh pr diff --patch`

## Evidence capture

| Evidence | Result | Path / command | Notes |
|---|---:|---|---|
| PR file list | `PASS` | `docs/roadmap/pcc/external/pr_1301_changed_files.txt` | Captured from `gh pr diff 1301 --repo skulmakov-oss/Semantic --name-only`; file contains 119 lines. |
| PR metadata | `PASS` | `gh pr view 1301 --repo skulmakov-oss/Semantic --json title,state,mergedAt,url,files,commits` | Returned merged PR metadata and full file list object. |
| Representative patch access | `PASS` | `gh pr diff 1301 --repo skulmakov-oss/Semantic --patch` | Patch access succeeded and was sampled via representative file headers/snippets. |
| Local merge SHA available | `FAIL` | `git cat-file -t 736b8bb066ea68e7e6d2e79ff300f77117c51561` | Merge SHA remains absent from this repo history. |

## Sampling summary

| Layer | Sample count | Result | Default decision | Notes |
|---|---:|---:|---|---|
| Linguist templates | `3` | `PASS` | `PORT-CANDIDATE` | Sampled readiness / usage-evidence / grammar-repo issue bodies. No submit-readiness claim is present. |
| Candidate probes | `4` | `PASS` | `PORT-CANDIDATE / NEEDS-COMPATIBILITY-AUDIT` | Sampled `AUDIT.md` and `src/main.sm` probe pairs. They are clearly probes, not canonical. |
| Canonical examples | `4` | `PASS` | `REBUILD-NATIVELY` | Sampled canonical `.sm` examples; these are new-file additions and must be rebuilt/probed locally if ever ported. |
| Negative fixtures | `4` | `PASS` | `REBUILD-NATIVELY` | Sampled control-flow, text, collections, and stdlib negative fixtures; external markers must not be reused blindly. |
| Negative harnesses | `4` | `PASS` | `REBUILD-NATIVELY` | Sampled `tests/pcc_*_negative.rs` harnesses; they are harness code, not port-ready canonical content. |
| 7hell wiring | `4` | `PASS` | `PORT LAST` | Sampled `run.ps1`, `run.sh`, `README.md`, and `7hell_mini_runner.md`; wiring is later-stage only. |
| PCC / CTF sync docs | `3` | `PASS` | `REBUILD AFTER LOCAL PCC EXISTS` | Sampled checkpoint / closeout / trust-lane closeout docs; wording depends on local PCC qualification. |
| CTF issue bodies | `3` | `PASS` | `PORT-CANDIDATE LATER` | Sampled execution-handle issue bodies; useful only after local sync validity exists. |
| Post-UI docs | `3` | `PASS` | `SEPARATE TRACK` | Sampled post-UI renderer/windowing reconciliation docs; they must remain separate from PCC. |
| PR 1185 audit | `1` | `PASS` | `NEEDS-COMPATIBILITY-AUDIT` | Sampled the PR 1185 7hell contour audit; it is a compatibility topic, not a PCC transfer target. |

## Sampled files

### Linguist templates

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `docs/roadmap/issues/issue_linguist_semantic_readiness.md` | `gh pr diff --patch` shows a new-file diff header and title `linguist: track Semantic language recognition readiness` | absent | `PORT-CANDIDATE` | Docs-only readiness wording; must keep submit-readiness blocker explicit. |
| `docs/roadmap/issues/issue_linguist_semantic_usage_evidence.md` | `gh pr diff --patch` and file list show this issue body as a new file | absent | `PORT-CANDIDATE` | Usage evidence blocker remains part of the claim surface. |
| `docs/roadmap/issues/issue_linguist_semantic_grammar_repo.md` | `gh pr diff --patch` and file list show this issue body as a new file | absent | `PORT-CANDIDATE` | Independent docs-only layer; still not a submit-readiness claim. |

### Candidate probes

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `examples/pcc_candidates/loop_control_flow_probe/AUDIT.md` | `gh pr diff --patch` shows a new-file audit doc for the probe | absent | `PORT-CANDIDATE` | Clearly probe/audit trail, not canonical. |
| `examples/pcc_candidates/loop_control_flow_probe/src/main.sm` | `gh pr diff --patch` shows a new-file `.sm` probe source | absent | `PORT-CANDIDATE` | Candidate code path only; local `smc check` would still be required later. |
| `examples/pcc_candidates/option_result_control_flow/AUDIT.md` | `gh pr diff --patch` shows a new-file audit doc for the option/result probe | absent | `PORT-CANDIDATE` | Clearly probe/audit trail, not canonical. |
| `examples/pcc_candidates/option_result_control_flow/src/main.sm` | `gh pr diff --patch` shows a new-file `.sm` probe source | absent | `PORT-CANDIDATE` | Candidate code path only; local `smc check` would still be required later. |

### Canonical examples

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `examples/canonical/match_control_flow/src/main.sm` | `gh pr diff --patch` shows a new canonical source file with `quad`/`match` control-flow content | absent | `REBUILD-NATIVELY` | Canonical examples must be rebuilt against the current local admitted surface, not imported directly. |
| `examples/canonical/text_core/src/main.sm` | `gh pr diff --patch` shows a new canonical source file for text handling | absent | `REBUILD-NATIVELY` | Text helper surface must be validated locally, not direct-ported. |
| `examples/canonical/collections_core/src/main.sm` | `gh pr diff --patch` shows a new canonical source file for collections helpers | absent | `REBUILD-NATIVELY` | Collections surface must be rebuilt/probed locally. |
| `examples/canonical/stdlib_v0_helpers/src/main.sm` | `gh pr diff --patch` shows a new canonical helper-surface example | absent | `REBUILD-NATIVELY` | Helper surface must be reconstructed after local compatibility checks. |

### Negative fixtures

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `tests/fixtures/pcc/control_flow/fail/if_quad_condition.sm` | `gh pr diff --patch` shows a new-file negative fixture for `quad` control flow | absent | `REBUILD-NATIVELY` | External markers are not reusable blindly. |
| `tests/fixtures/pcc/text/fail/text_plus_i32.sm` | `gh pr diff --patch` shows a new-file negative fixture for text concat misuse | absent | `REBUILD-NATIVELY` | Local marker re-probe is required. |
| `tests/fixtures/pcc/collections/fail/map_iteration_unsupported.sm` | `gh pr diff --patch` shows a new-file negative fixture for map iteration | absent | `REBUILD-NATIVELY` | Local marker re-probe is required. |
| `tests/fixtures/pcc/stdlib/fail/print_i32.sm` | `gh pr diff --patch` shows a new-file negative fixture for non-text `print` | absent | `REBUILD-NATIVELY` | Local marker re-probe is required. |

### Negative harnesses

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `tests/pcc_control_flow_negative.rs` | `gh pr diff --patch` shows a new harness file with `smc_cli`-style test plumbing | absent | `REBUILD-NATIVELY` | Harness must be rebuilt after fixtures/markers are confirmed locally. |
| `tests/pcc_text_negative.rs` | `gh pr diff --patch` shows a new harness file with `smc_cli`-style test plumbing | absent | `REBUILD-NATIVELY` | Harness must be rebuilt after fixtures/markers are confirmed locally. |
| `tests/pcc_collections_negative.rs` | `gh pr diff --patch` shows a new harness file with `smc_cli`-style test plumbing | absent | `REBUILD-NATIVELY` | Harness must be rebuilt after fixtures/markers are confirmed locally. |
| `tests/pcc_stdlib_negative.rs` | `gh pr diff --patch` shows a new harness file with `smc_cli`-style test plumbing | absent | `REBUILD-NATIVELY` | Harness must be rebuilt after fixtures/markers are confirmed locally. |

### 7hell wiring

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `tools/7hell/run.ps1` | `gh pr diff --patch` shows edits around Hell 6 wiring | exists | `PORT LAST` | Current local runner exists, but this layer must be last after local tests and harnesses exist. |
| `tools/7hell/run.sh` | `gh pr diff --patch` shows edits around Hell 6 wiring | exists | `PORT LAST` | Current local runner exists, but this layer must be last after local tests and harnesses exist. |
| `tools/7hell/README.md` | `gh pr diff --patch` shows corresponding runner documentation edits | exists | `PORT LAST` | Runner docs must stay aligned with local gating order. |
| `docs/roadmap/pcc/7hell_mini_runner.md` | `gh pr diff --patch` shows the mini-runner docs updated alongside wiring | exists | `PORT LAST` | Wiring is late-stage, not a first port slice. |

### PCC / CTF sync docs

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md` | `gh pr diff --patch` shows a large new checkpoint doc | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust wording depends on the local PCC stack, which is absent here. |
| `docs/roadmap/pcc/pcc_ctf_sync_closeout.md` | `gh pr diff --patch` shows a new closeout doc | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust wording depends on the local PCC stack, which is absent here. |
| `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_closeout.md` | `gh pr diff --patch` shows a corresponding trust-lane closeout | absent | `REBUILD AFTER LOCAL PCC EXISTS` | Trust wording depends on local qualification, not external claims. |

### CTF issue bodies

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `docs/roadmap/issues/issue_ctf_sync_runtime_value_registry.md` | `gh pr diff --patch` shows a compact execution-handle issue body | absent | `PORT-CANDIDATE LATER` | Useful later, but only after local sync validity exists. |
| `docs/roadmap/issues/issue_ctf_sync_trap_taxonomy.md` | `gh pr diff --patch` shows a compact execution-handle issue body | absent | `PORT-CANDIDATE LATER` | Useful later, but only after local sync validity exists. |
| `docs/roadmap/issues/issue_ctf_sync_closeout.md` | `gh pr diff --patch` shows the wording-pack closeout execution handle | absent | `PORT-CANDIDATE LATER` | Useful later, but only after local sync validity exists. |

### Post-UI docs

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `docs/roadmap/post_ui/r12_ui_renderer_boundary.md` | `gh pr diff --patch` shows status-note updates to the renderer boundary doc | exists | `SEPARATE TRACK` | Post-UI docs are real in this repo but not part of the PCC transfer path. |
| `docs/roadmap/post_ui/r12_ui_windowing_boundary.md` | `gh pr diff --patch` shows status-note updates to the windowing boundary doc | exists | `SEPARATE TRACK` | Post-UI docs are real in this repo but not part of the PCC transfer path. |
| `docs/roadmap/post_ui/r12_ui_native_wgpu_reality_reconciliation.md` | `gh pr diff --patch` shows the reconciliation doc being marked audited | exists | `SEPARATE TRACK` | Post-UI docs are real in this repo but not part of the PCC transfer path. |

### PR 1185 audit

| File | Sample evidence | Local state | Decision | Reason |
|---|---|---|---|---|
| `docs/roadmap/pcc/pr_1185_ci_7hell_platform_contour_audit.md` | `gh pr diff --patch` shows the audit doc as part of the PR stack | absent | `NEEDS-COMPATIBILITY-AUDIT` | This is a compatibility topic, not a port target. |

## Compatibility observations

Record only what was actually observed.

- external file list was captured into `docs/roadmap/pcc/external/pr_1301_changed_files.txt`: `VERIFIED`
- representative patch access via `gh pr diff --patch`: `VERIFIED`
- local merge SHA remains absent from this repo history: `VERIFIED`
- Linguist templates may be the first safe candidate if no submit-readiness claim is present: `SUPPORTED BY SAMPLE`
- candidate probes are a non-canonical audit trail: `SUPPORTED BY SAMPLE`
- canonical examples should be rebuilt/probed locally: `SUPPORTED BY SAMPLE`
- negative fixtures require local marker re-probe: `SUPPORTED BY SAMPLE`
- 7hell wiring must remain last: `SUPPORTED BY SAMPLE`
- PCC / CTF sync docs depend on a missing local PCC stack: `SUPPORTED BY SAMPLE`
- post-UI docs remain a separate track: `SUPPORTED BY SAMPLE`

## First safe slice recommendation

Based on sampled evidence, recommend:

```text
Linguist templates first, as docs-only independent readiness templates,
provided submit-readiness blockers remain explicit.
```

## Do-not-port-as-monolith rule

PR `#1301` must not be cherry-picked, merged, or patch-applied as a single
unit.

## Next step

If evidence is sufficient:

```text
PCC-PORT-2: first small safe slice proposal
```

If deeper per-file compatibility work is still needed:

```text
PCC-PORT-1D: targeted sampling for missing layer evidence
```

## Final verdict

One of:

- `PASS`
- `PASS-WITH-WARNINGS`
- `BLOCKED`

Use:

- `PASS` only if representative sampling succeeded across all major layers.
- `PASS-WITH-WARNINGS` if sampling succeeded enough to recommend a first safe
  slice, but deeper per-file compatibility remains required.
- `BLOCKED` if file list / patch evidence is still unavailable.

Final verdict:

`PASS-WITH-WARNINGS`

Reason:

- the external file list was captured;
- representative patch sampling succeeded across the requested major layers;
- the first safe slice can be recommended only as a hypothesis, not as a port;
- deeper per-file compatibility is still required before any transfer.
