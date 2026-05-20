# CTF Evidence Backlog

Status: active backlog
Owner: language maturity / execution contract
Scope: evidence backlog after PCC-4..PCC-9 and CTF-WP1..WP4
Non-goal: implementation, trace artifact addition, release readiness, or CTF closure

## Purpose

This document converts the CTF-WR1 evidence backlog preview into an actionable backlog.

It does not add evidence artifacts.

It defines what evidence is needed next.

It prevents uncontrolled jump from docs-sync to release claims.

## Evidence Backlog Table

| Backlog ID | Area | Evidence need | Why needed | Candidate PR | Status | Blocking for |
| ---------- | ---- | ------------- | ---------- | ------------ | ------ | ------------ |
| CTF-BL-001 | Golden trace selection | choose representative PCC fixture surfaces for trace promotion | PCC fixtures are not golden traces | CTF-E1 | done | trace freeze |
| CTF-BL-002 | Golden trace artifacts | add selected source/type/IR/SemCode/verifier/VM trace samples | protect byte/result/diagnostic stability | CTF-E1 | done | trace freeze |
| CTF-BL-003 | Collection determinism replay | repeated-run evidence for Sequence/Map admitted baseline | collection determinism is bounded but not deeply replay-backed | CTF-E2 | planned | determinism freeze |
| CTF-BL-004 | Map open-edge policy | decide missing-key / iteration / quota evidence boundary | Map remains bounded-open | CTF-WP / future PCC-7 follow-up | planned | collections freeze |
| CTF-BL-005 | Trap taxonomy regression | map fixture-backed failure surfaces to stable trap/diagnostic categories | avoid compile diagnostics vs VM trap confusion | CTF-E3 | planned | trap freeze |
| CTF-BL-006 | Project-root trust policy | define trust impact before project-root check/run implementation | project-root remains open | PCC-9I / CTF follow-up | planned | project model freeze |
| CTF-BL-007 | 7hell report shape | define stable qualification report surface | readiness needs stable qualification output | 7HELL-WP / CTF follow-up | planned | readiness |
| CTF-BL-008 | Capability denial replay | replay denied-effect behavior once capability surfaces widen | capability denial must be deterministic | future CTF-E | deferred | capability freeze |
| CTF-BL-009 | SymbolId / hot-path audit | verify PCC value/name surfaces do not regress into string hot paths | freeze-candidate names need hot-path discipline | future CTF-WP/E | planned | runtime performance/trust |

Status values allowed:

- `planned`
- `ready-for-task`
- `in-progress`
- `blocked`
- `deferred`
- `done`

Do not mark any backlog item done unless the evidence already exists and is linked.

## CTF-E1 Evidence

CTF-E1 adds the first selected golden trace coverage for PCC fixture-backed surfaces.

Covered:

- Records trace candidate;
- ADT / match trace candidate;
- Option trace candidate;
- Sequence trace candidate;
- Stdlib helper boundary trace candidate.

Trace artifacts are checked in under:

- `tests/fixtures/core_trust_freeze/golden_traces/ctf_e1/`;
- `tests/ctf_e1_golden_traces.rs`.

Boundaries:

- not all PCC fixtures are golden traces;
- no release readiness claim;
- no CTF closure;
- no 7hell report trace;
- no Map missing-key / iteration / quota trace;
- no project-root execution trace;
- no semantic.toml trace;
- no package registry / remote dependency trace.

## Evidence Classes

| Evidence class | Meaning | Example |
| -------------- | ------- | ------- |
| E0-doc | documented claim only | policy docs |
| E1-code | implementation exists | code path exists |
| E2-test | ordinary test evidence | cargo test / integration test |
| E3-trace | golden trace / snapshot evidence | stable trace artifact |
| E4-replay | repeated-run determinism evidence | replay harness |
| E5-release-gate | release qualification gate | 7hell/readiness gate |

Rules:

- `freeze-candidate` can be supported by E0/E1/E2 depending on scope.
- `frozen` requires at least E2 and usually E3/E4 for execution-sensitive surfaces.
- release-facing freeze requires E5 or explicit waiver.

## Prioritized Next PRs

```text
CTF-E2 — test(core-trust-freeze): add collection determinism replay coverage
CTF-E3 — test(core-trust-freeze): add trap taxonomy regression coverage
CTF-WP6 — docs(core-trust-freeze): define project-root trust policy before PCC-9I
7HELL-WP1 — docs(7hell): define qualification report contract
```

Make clear:

- CTF-E2 should focus on admitted collection baseline only, not open Map policy edges.
- CTF-E3 should distinguish compile-time diagnostics from VM traps.

## Out of Scope

- release readiness;
- CTF closure;
- new language features;
- project-root implementation;
- semantic.toml parser;
- smc new;
- package registry;
- remote dependencies;
- Workbench / UI;
- host IO widening;
- runtime behavior changes.

## Acceptance Checklist

```markdown
- [ ] backlog IDs defined
- [ ] evidence classes defined
- [ ] next PR order proposed
- [ ] no item overclaimed as done without evidence
- [ ] no CTF closure claimed
- [ ] no release readiness claimed
- [ ] no unrelated code changed
- [ ] no unrelated tests or fixtures changed
- [ ] no unreviewed trace artifacts added
```
