# 7hell Initial Fixture Selection

Status: active fixture-selection contract
Owner: language maturity / qualification harness
Scope: initial fixture selection for future 7hell skeleton command
Non-goal: implementation, fixture addition, CI gate, release readiness, or CTF closure

## 1. Purpose

This document defines the initial fixture selection policy for the future `smc 7hell` skeleton command.

It does not implement `smc 7hell`.

It does not add fixtures.

It does not add tests.

It does not create a CI gate.

It does not claim readiness.

The goal is to prevent the first skeleton command from trying to cover every PCC surface at once. The first skeleton should prove the stage/report shape using a deliberately small, representative fixture set.

## 2. Inputs

This document depends on:

- `7hell_qualification_contract.md`
- `7hell_report_contract.md`
- `7hell_pcc4_pcc9_stage_mapping.md`
- `docs/roadmap/language_maturity/core_trust_freeze/ctf_evidence_backlog.md`
- `docs/roadmap/language_maturity/core_trust_freeze/project_root_trust_policy.md`

The selected fixture set must respect the existing CTF boundaries.

## 3. Selection principles

The initial fixture set must be:

- small enough for a skeleton command;
- representative across all seven 7hell stages;
- already backed by existing PCC evidence where possible;
- deterministic;
- path-normalized;
- free of project-root assumptions;
- free of release-readiness claims.

Hard rules:

```text
Rule 1:
Initial 7hell skeleton fixtures must come from already admitted single-file surfaces unless explicitly marked as project-adjacent diagnostics.

Rule 2:
Project-root behavior, semantic.toml, src/main.sm discovery, and smc new remain future.

Rule 3:
Compile/check diagnostics must not be treated as VM traps.

Rule 4:
VM Hell must not run unverified SemCode.

Rule 5:
Practical Hell is an end-to-end sanity stage, not a release gate.
```

## 4. Initial skeleton fixture budget

Recommended first skeleton budget:

| Group | Count | Purpose |
| --- | ---: | --- |
| Positive end-to-end fixtures | 3 | prove happy path through stages |
| Negative diagnostics fixtures | 3 | prove failure/stage reporting |
| Runtime trap fixtures | 1 | prove VM trap reporting |
| Project-adjacent diagnostic fixture | 1 | prove unsupported/project boundary reporting |

Total target: 8 fixtures.

Maximum for the first skeleton: 10 fixtures.

Do not exceed this without a separate docs decision.

## 5. Candidate fixture set

### Positive fixtures

| Candidate ID | Source fixture | Surface | 7hell stages | Reason |
| --- | --- | --- | --- | --- |
| `7H-S1-POS-001` | `tests/fixtures/pcc5_match/` representative positive match fixture | ADT + Match | syntax, type, lowering, verifier, vm, practical | exercises ADT + match without project-root scope |
| `7H-S1-POS-002` | `tests/fixtures/pcc6_option/` representative Option fixture | Option | syntax, type, lowering, verifier, vm, practical | exercises standard-form Option flow |
| `7H-S1-POS-003` | `tests/fixtures/pcc7_sequence/positive_sequence_indexing.sm` | Sequence | syntax, type, lowering, verifier, vm, practical | exercises admitted collection baseline |

Selection note:
If a named directory has multiple valid candidates, the implementation PR must choose one stable fixture and record the exact path in the skeleton mapping.

### Negative diagnostics fixtures

| Candidate ID | Source fixture | Surface | Expected failure layer | Reason |
| --- | --- | --- | --- | --- |
| `7H-S1-DIAG-001` | `tests/fixtures/pcc5_adt_diagnostics/` representative constructor/payload diagnostic | ADT diagnostic | check-diagnostic | validates type/constructor failure routing |
| `7H-S1-DIAG-002` | `tests/fixtures/pcc6_option_result_diagnostics/` representative payload/exhaustiveness diagnostic | Option/Result diagnostic | check-diagnostic | validates standard-form diagnostics |
| `7H-S1-DIAG-003` | `tests/fixtures/pcc8_stdlib_diagnostics/negative_to_text_record.sm` | Stdlib helper diagnostic | check-diagnostic | protects to_text admitted-types-only boundary |

### Runtime trap fixture

| Candidate ID | Source fixture | Surface | Expected failure layer | Reason |
| --- | --- | --- | --- | --- |
| `7H-S1-TRAP-001` | `tests/fixtures/pcc8_stdlib_diagnostics/negative_assert_false_trap.sm` | assert(false) | vm-trap | validates runtime trap stage reporting |

### Project-adjacent diagnostic fixture

| Candidate ID | Source fixture | Surface | Expected failure layer | Reason |
| --- | --- | --- | --- | --- |
| `7H-S1-PROJ-001` | `tests/fixtures/pcc9_project_model_diagnostics/` representative missing-field or malformed manifest fixture | Project Model baseline | project-diagnostic | validates project-adjacent diagnostics without project-root execution |

Boundary:
This does not implement project-root `smc check <project-root>` or `smc run <project-root>`.

## 6. Stage coverage matrix

| Stage | Covered by initial selection | Notes |
| --- | --- | --- |
| Syntax Hell | positive fixtures parse | syntax-specific negative fixture may be added later |
| Type Hell | ADT, Option, Sequence, helper diagnostics | check diagnostics stay separate from VM traps |
| Lowering Hell | positive fixtures that lower | skeleton may report blocked if earlier stage fails |
| Verifier Hell | positive fixtures that emit SemCode | VM must only run verified SemCode |
| VM Hell | positive execution + assert(false) trap | runtime traps are not compile diagnostics |
| Practical Hell | small positive end-to-end fixtures | not release readiness |
| Diagnostics Hell | negative diagnostics + project-adjacent diagnostic | diagnostics must be stable and actionable |

## 7. Report object mapping

Future skeleton reports should map selected fixtures as follows:

| Fixture kind | `stages[].status` | `diagnostics[].kind` | `evidence[].class` | Boundary |
| --- | --- | --- | --- | --- |
| positive end-to-end | `pass` until final stage | none unless warning policy exists | `E2-test` | no readiness claim |
| check diagnostic | earlier stage `fail`, later stages `blocked` | `check-diagnostic` | `E2-test` | not VM trap |
| VM trap | VM stage `fail` or trap-specific failure status if later admitted | `vm-trap` | `E2-test` | verified execution only |
| project-adjacent diagnostic | diagnostics/project stage fail as policy defines | `project-diagnostic` | `E2-test` | not project-root execution |

No JSON report is generated by this PR.

## 8. Explicit exclusions

The initial skeleton selection excludes:

- project-root `smc check <project-root>`;
- project-root `smc run <project-root>`;
- `semantic.toml` parser / loader;
- `src/main.sm` discovery;
- `smc new`;
- package registry;
- remote dependencies;
- workspace / multi-package behavior;
- Map missing-key policy;
- Map iteration policy;
- collection memory/quota policy;
- capability denial replay;
- release gate behavior;
- CI integration.

## 9. Stop conditions for 7HELL-S1

Future skeleton implementation must stop if:

1. selected fixture output is nondeterministic;
2. selected fixture requires production semantic changes;
3. selected fixture requires project-root implementation;
4. selected fixture requires `semantic.toml`;
5. selected fixture requires Map missing-key or Map iteration policy;
6. selected fixture requires collection quota policy;
7. report generation needs absolute paths by default;
8. report generation mixes check diagnostics with VM traps;
9. skeleton command would be treated as release gate;
10. CI gate behavior would be introduced accidentally.

## 10. Recommended next split

```text
7HELL-S1 — cli(7hell): add docs-backed skeleton command without release gate
```

Alternative docs split if implementation discovers fixture ambiguity:

```text
7HELL-WP4 — docs(7hell): finalize exact skeleton fixture path list
```

## 11. Final verdict

```text
7HELL-WP3 defines the initial fixture selection for the future skeleton command.
It does not implement 7hell.
It does not add fixtures.
It does not create a release gate.
It does not claim readiness.
```

## 12. Acceptance checklist

- [ ] initial fixture budget defined
- [ ] positive fixture candidates defined
- [ ] negative diagnostic candidates defined
- [ ] runtime trap candidate defined
- [ ] project-adjacent diagnostic candidate defined
- [ ] stage coverage matrix defined
- [ ] report object mapping defined
- [ ] explicit exclusions listed
- [ ] stop conditions listed
- [ ] no implementation added
- [ ] no fixtures added
- [ ] no tests added
- [ ] no CI gate added
- [ ] no release readiness claimed
- [ ] no CTF closure claimed
