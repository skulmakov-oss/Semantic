# PCC Stack Linguist Small Slice Port Plan

## Status

Result: PLAN-ONLY

This is PCC-PORT-4.

This document is audit-only and plan-only.

No files were ported.
No cherry-pick was performed.
No merge was performed.
No patch was applied.
No implementation code was changed.
No tests/examples/7hell files were changed.

## Purpose

Define the actual small-slice port plan for the five Linguist readiness template files after the wording audit passed with warnings.

This is still not a port.
It is the pre-port plan that must be satisfied before a future isolated docs-only transfer can happen.

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
- Linguist wording audit: `PASS-WITH-WARNINGS`
- dirty tree: untracked local files are present

## External PR reference

- PR: `#1301`
- repo: `skulmakov-oss/Semantic`
- title: `docs/test(pcc): close practical core phase and sync CTF follow-ups`
- merge SHA: `736b8bb066ea68e7e6d2e79ff300f77117c51561`
- head SHA: `987857f961a2e17141490925f135849a0f6f7ef8`
- changed files: `119`
- commits: `7`
- local availability of merge SHA: `absent`

## Proposed slice

Slice:

```text
Linguist readiness templates
```

Files in scope:

| File | Layer | Current local state | Port status | Notes |
|---|---|---|---|---|
| `docs/roadmap/issues/issue_linguist_semantic_readiness.md` | Linguist templates | absent | candidate | Keep not-submit-ready wording. |
| `docs/roadmap/issues/issue_linguist_semantic_grammar_repo.md` | Linguist templates | absent | candidate | Keep readiness-infrastructure wording. |
| `docs/roadmap/issues/issue_linguist_semantic_usage_evidence.md` | Linguist templates | absent | candidate | Keep public `.sm` usage-evidence blocker explicit. |
| `docs/roadmap/issues/issue_linguist_semantic_samples.md` | Linguist templates | absent | candidate | Keep samples as readiness/canonical candidates only. |
| `docs/roadmap/issues/issue_linguist_semantic_local_validation.md` | Linguist templates | absent | candidate | Keep local validation pending / environment-dependent. |

## Why this slice is the first actual port candidate

This slice remains the smallest credible unit because:

- it is docs-only;
- it is independent from the missing PCC Practical Core stack;
- it does not require parser, verifier, VM, SemCode, or 7hell changes;
- it is already wording-audited against the captured external content;
- it can be transferred without dragging in examples, fixtures, or harnesses.

## Required port gates

Before any future actual port, all of these must hold:

### Gate P1 - Exact content copy

The five files must be copied from the captured PR #1301 content exactly, with no wording drift that weakens the blocker language.

### Gate P2 - Scope isolation

The diff must contain only these five files.

No PCC/CTF docs, no examples, no fixtures, no harnesses, no 7hell changes, and no post-UI files.

### Gate P3 - No submit-readiness claim

The final wording must not claim upstream Linguist submit-readiness.

### Gate P4 - Blocker preservation

The `.sm` usage evidence blocker must remain explicit.

### Gate P5 - Local path sanity

The destination path must not conflict with existing docs or other roadmap slices.

### Gate P6 - Commit granularity

If the port is approved, it should be a single isolated docs commit containing only the five Linguist files.

## Required pre-port checks

Before actual porting:

1. re-read the five sampled external file bodies from PR #1301;
2. compare the exact wording to the intended local destination text;
3. confirm the local paths do not already exist;
4. confirm the commit will touch only the five files;
5. run docs/pre-commit checks if available;
6. verify the branch still reflects the current local baseline.

## Non-goals

This plan does not:

- port the files;
- change code, tests, examples, or 7hell;
- claim Linguist submit-readiness;
- claim the public `.sm` threshold is met;
- reopen PCC or CTF scope;
- mix this slice with canonical examples or any other PR #1301 layer.

## Explicit exclusions

The following layers are not part of this small-slice port plan:

- canonical examples;
- PCC candidate probes;
- negative fixtures;
- negative harnesses;
- 7hell wiring;
- PCC / CTF sync docs;
- CTF issue bodies;
- post-UI docs;
- PR 1185 platform audit.

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Wording drifts during the actual copy | high | Copy only after a line-by-line check. |
| Slice expands beyond five files | high | Enforce diff scope before commit. |
| Submit-readiness gets implied | high | Preserve blocker wording verbatim in spirit. |
| PCC/CTF content leaks into slice | high | Reject any extra files. |
| Local path conflict appears | medium | Recheck destination paths before transfer. |

## Recommended next step

If this plan is approved:

```text
PCC-PORT-5: isolated port of the five Linguist readiness templates
```

If wording or scope review finds a mismatch:

```text
PCC-PORT-4B: revise the isolated slice plan
```

## Final verdict

One of:

- PLAN-ONLY
- PASS-WITH-WARNINGS
- BLOCKED
- REJECTED

Final verdict:

```text
PLAN-ONLY
```

Reason:

- the slice is identified and bounded;
- the wording audit already passed with warnings;
- actual porting is still intentionally withheld pending a separate approval step.
