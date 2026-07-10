# PCC Stack First Safe Slice Proposal

## Status

Result: `PROPOSED-WITH-WARNINGS`

This is `PCC-PORT-2`.

This document is proposal-only.

No files were ported.
No cherry-pick was performed.
No merge was performed.
No patch was applied.
No implementation code was changed.
No tests/examples/7hell files were changed.

## Purpose

Explain that this document proposes the first small safe slice from external
PR `#1301` after captured external diff sampling.

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
  - `docs/roadmap/pcc/pcc_stack_external_diff_sampling_captured.md`
  - `docs/roadmap/pcc/pcc_stack_external_diff_sampling_retry_plan.md`
  - `docs/roadmap/pcc/pcc_stack_external_inventory.md`
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
| `docs/roadmap/issues/issue_linguist_semantic_readiness.md` | `yes` | `absent` | `candidate` | Must keep "not submit-ready" wording and explicit blockers. |
| `docs/roadmap/issues/issue_linguist_semantic_grammar_repo.md` | `yes` | `absent` | `candidate` | Readiness infrastructure only; not acceptance. |
| `docs/roadmap/issues/issue_linguist_semantic_usage_evidence.md` | `yes` | `absent` | `candidate` | Usage evidence blocker remains explicit. |
| `docs/roadmap/issues/issue_linguist_semantic_samples.md` | `yes` | `absent` | `candidate` | Samples are readiness candidates, not proof of acceptance. |
| `docs/roadmap/issues/issue_linguist_semantic_local_validation.md` | `yes` | `absent` | `candidate` | Local validation remains pending / environment-dependent. |

## Why this slice is first

This is the first safe slice because it is:

- docs-only;
- independent from parser / VM / verifier behavior;
- independent from SemCode behavior;
- independent from tests / examples / 7hell wiring;
- not dependent on the absent local PCC Practical Core stack;
- easier to gate against overclaiming than code-bearing layers.

Captured patch sampling supports that this slice remains conservative:

- the readiness issue says the project is `not submit-ready yet`;
- the readiness issue says the real Linguist PR is still future work;
- the template set explicitly tracks usage evidence, grammar repo, samples,
  and local validation as readiness work;
- the captured wording does not claim upstream acceptance.

## Wording gates

| Gate | Result | Evidence / Required condition |
|---|---:|---|
| L1 not submit-ready wording | `PASS` | Captured readiness wording says `not submit-ready yet` and frames the work as a readiness track. |
| L2 usage evidence blocker explicit | `PASS` | Captured wording keeps usage evidence as a blocker / pending readiness item. |
| L3 grammar repo wording conservative | `PASS` | Captured grammar-repo wording stays at readiness infrastructure / candidate repo level, not acceptance. |
| L4 samples do not prove readiness | `PASS` | Captured samples wording treats samples as readiness artifacts, not proof of Linguist acceptance. |
| L5 local validation not overclaimed | `PASS` | Captured local validation wording remains pending / planned / environment-dependent. |
| L6 no dependency on missing PCC stack | `PASS` | Captured Linguist templates do not depend on the absent PCC Practical Core stack. |

## Non-goals

This proposal does not:

- port the files;
- claim Linguist submit-readiness;
- open an upstream Linguist PR;
- claim the `.sm` usage threshold is met;
- claim PCC Practical Core exists locally;
- claim CTF sync exists locally;
- modify code, tests, examples, or `tools/7hell`.

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

1. inspect exact external content of all five files;
2. verify wording gates L1-L6;
3. ensure local path does not conflict with existing docs;
4. create a dedicated branch / commit only for Linguist templates;
5. run docs / pre-commit checks if available;
6. do not include PCC / CTF or examples changes.

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
| Slice accidentally mixed with PCC stack | high | Only five issue template files are allowed. |
| External file content stale | medium | Recheck against PR `#1301` before port. |
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

- `PROPOSED-FIRST-SLICE`
- `PROPOSED-WITH-WARNINGS`
- `BLOCKED`
- `REJECTED`

Expected:

```text
PROPOSED-WITH-WARNINGS
```

Reason:

- the slice is likely the safest first candidate;
- it is docs-only and mostly independent;
- actual port still requires content wording audit.
