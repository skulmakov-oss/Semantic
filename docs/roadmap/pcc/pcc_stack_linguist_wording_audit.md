# PCC Stack Linguist Wording Audit

## Status

Result: PASS-WITH-WARNINGS

This is PCC-PORT-3.

This document is audit-only.

No files were ported.
No cherry-pick was performed.
No merge was performed.
No patch was applied.
No implementation code was changed.
No tests/examples/7hell files were changed.

## Purpose

Explain that the first safe slice proposal from external PR #1301 is limited to the Linguist readiness templates layer and must still pass wording gates before any future port.

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
- captured sampling: `PASS-WITH-WARNINGS`
- dirty tree: untracked local files are present
- unexpected untracked files:
  - `docs/language/semantic_sugar_track_rfc.md`
  - `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
  - `docs/roadmap/pcc/local_repo_mismatch_audit.md`
  - `docs/roadmap/pcc/pcc_stack_bridge_audit.md`
  - `docs/roadmap/pcc/pcc_stack_external_diff_sampling.md`
  - `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
  - `docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md`
  - `docs/roadmap/pcc/pcc_stack_external_inventory.md`
  - `docs/roadmap/pcc/pcc_stack_first_safe_slice_proposal.md`
  - `docs/roadmap/pcc/pcc_stack_selective_port_plan.md`
  - `docs/roadmap/pcc/external/pr_1301_changed_files.txt`

## External PR reference

- PR: `#1301`
- repo: `skulmakov-oss/Semantic`
- title: `docs/test(pcc): close practical core phase and sync CTF follow-ups`
- merge SHA: `736b8bb066ea68e7e6d2e79ff300f77117c51561`
- head SHA: `987857f961a2e17141490925f135849a0f6f7ef8`
- changed files: `119`
- commits: `7`
- local availability of merge SHA: `absent`

## Proposed first slice

Slice:

```text
Linguist readiness templates
```

Candidate files:

| File | Present in captured PR file list | Current local state | Proposed status | Notes |
|---|---:|---|---|---|
| `docs/roadmap/issues/issue_linguist_semantic_readiness.md` | yes | absent | candidate | Docs-only readiness wording; must keep submit-readiness blocker explicit. |
| `docs/roadmap/issues/issue_linguist_semantic_grammar_repo.md` | yes | absent | candidate | Readiness infrastructure, not acceptance. |
| `docs/roadmap/issues/issue_linguist_semantic_usage_evidence.md` | yes | absent | candidate | Must retain explicit `.sm` usage evidence blocker. |
| `docs/roadmap/issues/issue_linguist_semantic_samples.md` | yes | absent | candidate | Samples are readiness/canonical candidates only. |
| `docs/roadmap/issues/issue_linguist_semantic_local_validation.md` | yes | absent | candidate | Local validation must remain pending/planned/environment-dependent. |

## Why this slice is first

This slice is the first plausible safe candidate because:

- it is docs-only;
- it does not require parser, VM, verifier, or SemCode changes;
- it does not require tests, examples, or 7hell changes;
- it does not depend on the absent local PCC Practical Core stack;
- its safety can be gated by wording rather than behavior.

## Wording gates

| Gate | Result | Evidence / Required condition |
|---|---:|---|
| L1 not submit-ready wording | PASS | `Status: not submit-ready` and equivalent blocker wording are present in the sampled readiness doc. |
| L2 usage evidence blocker explicit | PASS | The sampled usage-evidence doc keeps the upstream PR blocked until public `.sm` usage evidence is strong enough. |
| L3 grammar repo wording conservative | PASS | The sampled grammar repo doc frames the grammar repository as readiness infrastructure, not acceptance. |
| L4 samples do not prove readiness | PASS | The sampled samples doc treats samples as readiness/canonical candidates, not proof of Linguist acceptance. |
| L5 local validation not overclaimed | PASS | The sampled local-validation doc keeps validation pending / planned / environment-dependent. |
| L6 no dependency on missing PCC stack | PASS | The slice stays in the language-recognition / grammar-readiness lane and does not rely on the absent PCC stack. |

## Non-goals

This proposal does not:

- port the files;
- claim Linguist submit-readiness;
- open an upstream Linguist PR;
- claim `.sm` usage threshold is met;
- claim PCC Practical Core exists locally;
- claim CTF sync exists locally;
- modify code, tests, examples, or 7hell.

## Explicitly excluded from this slice

The following layers are not part of this first slice:

- canonical examples;
- PCC candidate probes;
- negative fixtures;
- negative harnesses;
- 7hell wiring;
- PCC / CTF sync docs;
- CTF issue bodies;
- post-UI docs;
- PR 1185 platform audit.

## Required checks before actual port

Before any future port of this slice:

1. inspect the exact external content of all five files;
2. verify wording gates L1-L6 against that content;
3. ensure the local path does not conflict with existing docs;
4. create a dedicated branch or commit only for Linguist templates;
5. run docs/pre-commit checks if available;
6. do not include PCC/CTF or examples changes.

## Proposed future commit, if approved later

Suggested future commit message:

```text
docs(linguist): add Semantic recognition readiness issue templates
```

This is only a future proposal, not an action.

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Linguist readiness overclaimed | high | Enforce wording gates. |
| Usage evidence blocker omitted | high | Require explicit blocker. |
| Slice accidentally mixed with PCC stack | high | Keep the slice limited to the five issue template files. |
| External file content stale | medium | Recheck against PR #1301 before port. |
| Local docs path conflict | medium | Check current local tree before port. |

## Recommended next step

If this proposal is accepted:

```text
PCC-PORT-3: Linguist readiness templates wording audit
```

Still audit-only unless explicitly approved for actual port.

If wording evidence is insufficient:

```text
PCC-PORT-2B: deeper Linguist template content sampling
```

## Final verdict

One of:

- PASS
- PASS-WITH-WARNINGS
- BLOCKED
- REJECTED

Final verdict:

```text
PASS-WITH-WARNINGS
```

Reason:

- the slice is docs-only and the sampled wording is conservative;
- the five Linguist template files are present in the captured external PR file list;
- actual port still requires a separate content-and-approval step.
