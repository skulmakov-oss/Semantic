# 7hell Skeleton-to-Runner Transition Rules

Status: active transition policy
Owner: language maturity / qualification harness
Scope: transition from skeleton report command to staged runner
Non-goal: implementation, fixtures, CI gate, release readiness, or CTF closure

## 1. Purpose

This document defines how `smc 7hell` may evolve from skeleton report into an executable stage runner.

It does not implement any runner behavior.

It prevents unsafe widening of `7hell`.

It keeps stage execution incremental, observable, and verifier-first.

It defines which stages may become executable first and which must remain blocked or not_implemented.

## 2. Current State

| Item               | Current state   | Source                     |
| ------------------ | --------------- | -------------------------- |
| skeleton command   | landed          | 7HELL-S1                   |
| human report shape | landed skeleton | 7HELL-S1 / report contract |
| JSON report shape  | landed skeleton | 7HELL-S1 / report contract |
| stage mapping      | docs-only       | 7HELL-WP2                  |
| stage execution    | not implemented | future                     |
| project-root 7hell | not implemented | future after PCC-9I        |
| release gate       | not admitted    | future policy              |
| CI gate            | not admitted    | future policy              |

## 3. Transition Principles

```text
Rule 1:
Skeleton report shape may exist before stage execution.

Rule 2:
A stage may only move from not_implemented to executable when its input/output contract is documented.

Rule 3:
7hell must not execute unchecked source as VM input.

Rule 4:
Verifier Hell and VM Hell must remain blocked until earlier source/check/lowering stages provide valid inputs.

Rule 5:
Project-root 7hell must remain not_implemented until PCC-9I defines and implements project-root behavior.

Rule 6:
A 7hell PASS is not release readiness.

Rule 7:
CI/release gate behavior requires separate policy and PR.

Rule 8:
Stage execution must preserve deterministic report output.
```

## 4. Stage Activation Ladder

| Step | Stage behavior                 | Allowed transition                                                              | Required evidence                  |
| ---- | ------------------------------ | ------------------------------------------------------------------------------- | ---------------------------------- |
| L0   | skeleton only                  | all stages `not_implemented`, result `incomplete`                               | S1                                 |
| L1   | syntax/check probe             | Syntax and Type may run existing single-file check path                         | docs + tests                       |
| L2   | diagnostics wiring             | Syntax/Type diagnostics mapped into report objects                              | report snapshot tests              |
| L3   | lowering/verifier placeholders | Lowering/Verifier can be `blocked` or `not_implemented` based on earlier result | report snapshot tests              |
| L4   | verifier execution             | Verifier may run only on emitted SemCode from checked/lowered source            | verifier-first evidence            |
| L5   | VM execution                   | VM may run only verified SemCode                                                | verifier-first + VM evidence       |
| L6   | practical stage                | practical selected fixtures / source path                                       | PCC/7hell evidence                 |
| L7   | diagnostics hell               | diagnostic quality checks                                                       | diagnostics evidence               |
| L8   | project-root mode              | after PCC-9I only                                                               | project-root trust policy + traces |
| L9   | CI/release gate                | future policy only                                                              | explicit release-gate PR           |

Important:

- `7HELL-S2` should target L1 only.
- `7HELL-S3` should target L2 only.
- `7HELL-S4` should target L3 placeholders only.
- L4/L5 must not be smuggled into S2/S3.

## 5. Allowed Next Implementation Split

```text
7HELL-S2 — cli(7hell): route skeleton stages to existing single-file check where safe
Allowed:
  - invoke existing single-file check path if available without new semantics
  - set Syntax/Type to pass/fail based on check result if stable
  - keep Lowering/Verifier/VM/Practical as blocked or not_implemented
  - keep project-root rejected / unsupported
  - keep result incomplete unless policy proves otherwise

Forbidden:
  - compile/emit SemCode
  - verifier execution
  - VM execution
  - project-root behavior
  - semantic.toml
  - CI gate
  - release gate
```

```text
7HELL-S3 — cli(7hell): add stage result wiring for syntax/type diagnostics
Allowed:
  - map existing diagnostic categories to report diagnostics[]
  - keep stable code/needle only
  - report blocked downstream stages
  - add report snapshot tests

Forbidden:
  - VM traps
  - verifier rejection unless actual verifier runs
  - project diagnostics pretending project-root exists
```

```text
7HELL-S4 — cli(7hell): add verifier / VM stage placeholders with explicit blocked states
Allowed:
  - make downstream blocked/not_implemented states clearer
  - no verifier/VM execution unless separately approved

Forbidden:
  - actual verifier/VM execution
  - release gate
```

Then future, not immediate:

```text
7HELL-S5 — cli(7hell): add verifier stage execution for selected single-file fixtures
7HELL-S6 — cli(7hell): add VM stage execution for selected verified fixtures
7HELL-E1 — test(7hell): add first report snapshot tests
```

## 6. Status Transition Rules

| From            | To             | Condition                                             |
| --------------- | -------------- | ----------------------------------------------------- |
| not_implemented | blocked        | earlier stage produced a stable blocker               |
| not_implemented | pass           | stage executable and stable                           |
| not_implemented | fail           | stage executable and stable diagnostic/failure exists |
| blocked         | pass           | blocker removed and stage executable                  |
| blocked         | fail           | blocker removed and stage fails stably                |
| pass/fail       | changed status | requires test/report snapshot update                  |

Rules:

- `blocked` must reference `blocked_by`.
- `fail` means the stage actually ran.
- `not_implemented` means no runner exists.
- `skip` requires explicit policy reason.
- `incomplete` remains valid while any required stage is not_implemented.
- `pass-with-skips` must not be treated as release pass.

## 7. Report Field Transition Rules

- `diagnostics[]` may be empty in skeleton.
- `diagnostics[]` may be populated only from stable diagnostic code/needle/category.
- `evidence[]` may reference existing evidence only if stable ID/path exists.
- `ctf[]` may reference policy, but does not close CTF.
- `boundaries[]` must include unsupported project-root, semantic.toml, CI gate, and release gate where applicable.
- `target.kind` must remain `single-file` until project-root is implemented.

## 8. Failure Layering Rules

- Syntax/check diagnostics are not VM traps.
- Lowering diagnostics are not verifier rejections unless SemCode verifier runs.
- Verifier rejection means SemCode admission failed.
- VM trap means verified SemCode ran and trapped.
- Project diagnostics require project-root mode; until then, project-root requests are unsupported CLI boundary, not project diagnostics.
- Capability/boundary denial must not be hidden as diagnostics if it is policy denial.

## 9. Project-Root Boundary

`smc 7hell --project .` remains unsupported until PCC-9I and project-root trust policy are implemented.

`semantic.toml` remains future.

`src/main.sm` discovery remains future.

`smc new` remains future.

No package registry.

No remote dependencies.

No workspace model.

No project-root report PASS until project-root trace/replay evidence exists.

## 10. Determinism Requirements for Runner

Stage runner must not add:

- wall-clock time;
- absolute paths;
- temp dirs;
- environment dumps;
- nondeterministic ordering;
- telemetry IDs.

If stage execution adds timing or metrics later:

- must be behind explicit debug/metrics mode;
- must not affect pass/fail;
- must not enter stable JSON by default.

## 11. Required Evidence Before Each Transition

| Transition                    | Required docs                      | Required tests                  |
| ----------------------------- | ---------------------------------- | ------------------------------- |
| skeleton -> syntax/type check | transition rules + report contract | bin/unit test + snapshot/needle |
| syntax/type -> diagnostics[]  | diagnostic object contract         | report snapshot                 |
| lowering placeholder          | stage mapping                      | snapshot                        |
| verifier execution            | verifier-first policy              | verifier/trace test             |
| VM execution                  | verifier-first + trap taxonomy     | VM/trap snapshot                |
| practical stage               | PCC evidence mapping               | practical fixture snapshot      |
| project-root mode             | CTF-WP6 + PCC-9I                   | project-root traces/replay      |
| CI gate                       | release policy                     | release gate test               |

## 12. Stop Conditions

Future implementation must stop if:

1. S2 would invoke compile/verify/VM.
2. S2 would produce `pass` for Lowering/Verifier/VM.
3. S2 would accept `--project`.
4. S2 would parse `semantic.toml`.
5. S2 would create release-gate semantics.
6. report output includes absolute paths.
7. report output includes timing by default.
8. diagnostic mapping requires unstable full text.
9. VM trap is inferred without VM execution.
10. verifier rejection is inferred without verifier execution.
11. CTF reference is treated as CTF closure.
12. result `pass` is produced while stages remain not_implemented.

## 13. Branch Plan After WP3

Recommended next PRs:

```text
7HELL-S2 — cli(7hell): route skeleton stages to existing single-file check where safe
7HELL-S3 — cli(7hell): add stage result wiring for syntax/type diagnostics
7HELL-E1 — test(7hell): add first report snapshot tests
7HELL-S4 — cli(7hell): add verifier / VM stage placeholders with explicit blocked states
```

If S2 cannot safely call existing check without widening behavior, stop and report a seam.

S2 should prefer shallow syntax/type check only.

S2 should not add verifier or VM.

## 14. Final Verdict

```text
7HELL-WP3 defines skeleton-to-runner transition rules.
It does not implement the runner.
It does not change command behavior.
It does not create a CI or release gate.
It does not claim readiness.
```

## 15. Acceptance Checklist

```markdown
- [ ] current skeleton state documented
- [ ] transition principles defined
- [ ] stage activation ladder defined
- [ ] S2/S3/S4 boundaries defined
- [ ] status transition rules defined
- [ ] report field transition rules defined
- [ ] failure layering rules defined
- [ ] project-root boundary defined
- [ ] determinism requirements defined
- [ ] evidence requirements listed
- [ ] stop conditions listed
- [ ] no command behavior changed
- [ ] no tests or fixtures added
- [ ] no CI gate added
- [ ] no readiness claimed
- [ ] no CTF closure claimed
```
