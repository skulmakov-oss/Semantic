# CTF Freeze-Candidate Promotion Rules

Status: active policy
Owner: language maturity / execution contract
Scope: promotion rules for CTF entries after PCC/CTF sync
Non-goal: release readiness or automatic freeze

## Purpose

This document defines how CTF entries move from `audit-needed` / `planned` / `freeze-candidate` toward stronger freeze states.

It prevents docs-only PRs from silently creating release-facing freeze.

It keeps CTF honest after PCC expansion.

## Status Ladder

| Status | Meaning | Allowed evidence |
| ------ | ------- | ---------------- |
| planned | belongs to a future phase | E0-doc |
| audit-needed | exists or likely exists but needs review | E0/E1 |
| documented-only | contract intent exists, behavior not proven | E0-doc |
| freeze-candidate | behavior is bounded and should not change silently | E0 + E1/E2 preferred |
| evidence-backed | tests/traces/replay support the behavior | E2/E3/E4 |
| frozen | release-facing stable contract | E2 + E3/E4 or explicit waiver |
| out-of-scope | not part of current plan | E0-doc |

Do not use `frozen` casually.

## Promotion Gates

### planned → audit-needed

Required:

- surface identified;
- owner known;
- scope described.

### audit-needed → freeze-candidate

Required:

- bounded scope;
- current behavior described;
- open edges listed;
- no known contradictory evidence.

### freeze-candidate → evidence-backed

Required:

- tests, traces, or replay evidence;
- failure mode categorized;
- determinism impact reviewed;
- verifier-first impact reviewed if execution path exists.

### evidence-backed → frozen

Required:

- evidence is linked;
- release-facing contract is intended;
- CTF owner approves;
- overclaim risks reviewed;
- compatibility / migration note if public behavior may matter.

## Demotion Rule

A CTF entry must be demoted if:

- evidence is stale;
- behavior changed;
- scope widened;
- tests were removed;
- trace artifacts changed without explanation;
- open edge invalidates the previous claim.

Demotion targets:

- frozen → evidence-backed
- evidence-backed → freeze-candidate
- freeze-candidate → audit-needed

## Required CTF Note for Future PRs

```text
CTF touched:
  - <file>
Reason:
  <why trust surface changed>

CTF status impact:
  - no status change
  - planned -> audit-needed
  - audit-needed -> freeze-candidate
  - freeze-candidate -> evidence-backed
  - evidence-backed -> frozen
  - demotion required
```

## Prohibited Promotions

Explicitly forbid:

- docs-only PR promoting to release-facing `frozen` without evidence;
- treating PCC closeout as CTF freeze;
- treating fixtures as golden traces automatically;
- treating compile-time diagnostics as VM traps;
- treating project helper tests as public execution evidence;
- treating `print(text)` as host IO without capability review;
- treating `to_text` as reflection;
- treating `debug_render` as public language output.

## Review Checklist

```markdown
- [ ] status before and after stated
- [ ] evidence class stated
- [ ] open edges listed
- [ ] owner known
- [ ] overclaim risk reviewed
- [ ] demotion considered
- [ ] release claim avoided unless explicitly approved
```
