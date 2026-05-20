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
| CTF-BL-003 | Collection determinism replay | repeated-run evidence for Sequence/Map admitted baseline | collection determinism is bounded but not deeply replay-backed | CTF-E2 | done | determinism freeze |
| CTF-BL-004 | Map open-edge policy | decide missing-key / iteration / quota evidence boundary | Map remains bounded-open | CTF-WP / future PCC-7 follow-up | planned | collections freeze |
| CTF-BL-005 | Trap taxonomy regression | map fixture-backed failure surfaces to stable trap/diagnostic categories | avoid compile diagnostics vs VM trap confusion | CTF-E3 | done | trap freeze |
| CTF-BL-006 | Project-root trust policy | define trust impact before project-root check/run implementation | project-root remains open | CTF-WP6 | done | project model freeze |
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

## CTF-E2 Evidence

CTF-E2 adds collection determinism replay evidence for selected admitted PCC-7 collection fixtures.

Covered:

- Sequence indexing replay;
- Sequence iteration replay;
- Sequence mutation replay;
- Map insert/lookup replay;
- Map persistent update replay.

Replay artifacts are checked in under:

- `tests/fixtures/core_trust_freeze/replay/ctf_e2/`;
- `tests/ctf_e2_collection_replay.rs`.

Boundaries:

- no Map missing-key policy;
- no Map iteration policy;
- no collection memory/quota policy;
- no project-root determinism;
- no semantic.toml determinism;
- no smc new determinism;
- no 7hell report determinism;
- no CTF closure;
- no release readiness.

## CTF-E3 Evidence

CTF-E3 adds trap taxonomy regression evidence for selected PCC failure surfaces.

Covered:

- Sequence out-of-bounds runtime trap candidate;
- empty Sequence pop runtime trap candidate;
- assert(false) runtime trap;
- unsupported to_text diagnostic;
- project manifest diagnostic.

Boundaries:

- compile/check-time diagnostics are not VM traps;
- project manifest diagnostics are not project-root execution traps;
- Map missing-key policy remains open;
- Map iteration policy remains open;
- collection quota/memory policy remains open;
- no new trap class is promoted to frozen without existing evidence;
- no CTF closure;
- no release readiness.

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

## CTF-WP6 Evidence

CTF-WP6 defines project-root trust policy before PCC-9I implementation.

Covered:

- verifier-first route for future project-root commands;
- deterministic manifest / entry / path policy;
- project diagnostics policy;
- capability / effect boundary;
- future golden trace requirements;
- future PCC-9I split.

Boundaries:

- no project-root implementation;
- no semantic.toml parser;
- no smc new;
- no package registry;
- no dependency resolver;
- no workspace;
- no remote packages;
- no CTF closure;
- no release readiness.

## Prioritized Next PRs

```text
7HELL-WP1 — docs(7hell): define qualification report contract
PCC-9I1 — cli(project-model): add project-root check entrypoint
```

Make clear:

- CTF-E3 distinguishes compile-time diagnostics from VM traps.

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
