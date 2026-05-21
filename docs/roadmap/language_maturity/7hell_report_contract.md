# 7hell Qualification Report Contract

Status: active report contract
Owner: language maturity / qualification harness
Scope: report shape for future 7hell implementation
Non-goal: implementation, CI gate, release readiness, or CTF closure

## 1. Purpose

This document defines the report shape for future `smc 7hell`.

It does not implement the command.

It does not add a serializer.

It does not add fixtures.

It prevents future 7hell implementation from inventing unstable output ad hoc.

It separates human output, machine-readable JSON, stage records, diagnostics, evidence, and final verdict.

## 2. Report Model Overview

```text
7hell report =
  run metadata
  target metadata
  stage results
  diagnostics
  evidence references
  CTF status references
  final verdict
```

Rules:

- Report must be deterministic for same input, same toolchain, same config.
- Report must not include wall-clock timing by default.
- Report must not include absolute local paths by default.
- Report must not include machine-specific temp directories.
- Report must not include telemetry.
- Local audit is not telemetry.

## 3. Human Report Contract

Canonical human shape:

```text
Semantic 7hell qualification
target: <normalized-target>
mode: single-file | project
profile: <profile-name-or-default>

[1/7] Syntax Hell                  PASS
[2/7] Type Hell                    PASS
[3/7] Lowering Hell                PASS
[4/7] Verifier Hell                PASS
[5/7] VM Hell                      PASS
[6/7] Practical Hell               PASS
[7/7] User Pain / Diagnostics Hell PASS

result: PASS
```

Failure shape:

```text
Semantic 7hell qualification
target: <normalized-target>
mode: single-file | project
profile: <profile-name-or-default>

[1/7] Syntax Hell                  PASS
[2/7] Type Hell                    FAIL
  code: E0201
  category: type-diagnostic
  reason: <stable diagnostic summary>
  next: inspect diagnostic output

[3/7] Lowering Hell                BLOCKED
[4/7] Verifier Hell                BLOCKED
[5/7] VM Hell                      BLOCKED
[6/7] Practical Hell               BLOCKED
[7/7] User Pain / Diagnostics Hell PASS

result: FAIL
```

Rules:

- Human output is stage-oriented.
- Exact spacing may change before implementation, but stage names and terminal result vocabulary must remain stable.
- Failing stage should include stable diagnostic category.
- Later stages blocked by earlier failure must be `BLOCKED`, not `FAIL`.
- Diagnostics Hell may still run against the failure output if policy allows.

## 4. JSON Report Contract

Top-level JSON:

```json
{
  "schema": "semantic.7hell.report.v0",
  "tool": "smc 7hell",
  "target": {
    "kind": "single-file",
    "display": "program.sm",
    "normalized": "program.sm"
  },
  "profile": "default",
  "result": "pass",
  "stages": [],
  "diagnostics": [],
  "evidence": [],
  "ctf": [],
  "boundaries": []
}
```

Required top-level fields:

- `schema`
- `tool`
- `target`
- `profile`
- `result`
- `stages`
- `diagnostics`
- `evidence`
- `ctf`
- `boundaries`

Rules:

- Field order should be stable if serialized.
- Paths must be normalized.
- No absolute paths unless explicit debug mode is added later.
- No wall-clock durations in v0.
- No environment dump.
- No telemetry fields.

## 5. Target Object

```json
{
  "kind": "single-file",
  "display": "program.sm",
  "normalized": "program.sm"
}
```

Allowed `kind` values:

- `single-file`
- `project-root`
- `artifact`

For WP1:

- only `single-file` is contract-ready.
- `project-root` is future and depends on PCC-9I.
- `artifact` is future if 7hell validates emitted SemCode directly.

Rules:

- `project-root` must not be claimed implemented.
- `semantic.toml` must not be claimed implemented.
- `src/main.sm` discovery must not be claimed implemented.

## 6. Stage Record Contract

Stage object:

```json
{
  "index": 1,
  "name": "Syntax Hell",
  "key": "syntax",
  "status": "pass",
  "summary": "syntax accepted",
  "diagnostic_ids": [],
  "evidence_ids": [],
  "blocked_by": null
}
```

Required fields:

- `index`
- `name`
- `key`
- `status`
- `summary`
- `diagnostic_ids`
- `evidence_ids`
- `blocked_by`

Stage keys:

- `syntax`
- `type`
- `lowering`
- `verifier`
- `vm`
- `practical`
- `diagnostics`

Allowed stage statuses:

- `pass`
- `fail`
- `skip`
- `not_implemented`
- `blocked`

Rules:

- `blocked_by` must reference earlier stage key or null.
- `fail` means the stage ran and found a failure.
- `blocked` means an earlier stage prevented execution.
- `not_implemented` is allowed only before skeleton maturity.
- `skip` requires explicit policy reason.

## 7. Overall Result Contract

Allowed result values:

- `pass`
- `fail`
- `blocked`
- `pass-with-skips`
- `incomplete`

Rules:

| Stage states | Overall result |
| --- | --- |
| all pass | `pass` |
| any fail | `fail` |
| any blocked and no fail | `blocked` |
| pass + skip only | `pass-with-skips` |
| any not_implemented | `incomplete` |

Additional rules:

- `fail` takes priority over `blocked`.
- `incomplete` must not be used for release readiness.
- `pass-with-skips` must not be treated as full pass unless policy says skips are allowed.

## 8. Diagnostic Object Contract

```json
{
  "id": "D001",
  "stage": "type",
  "kind": "check-diagnostic",
  "code": "E0201",
  "category": "type-mismatch",
  "message_needle": "expected bool",
  "severity": "error",
  "source": {
    "file": "program.sm",
    "line": null,
    "column": null
  }
}
```

Allowed `kind` values:

- `syntax-diagnostic`
- `check-diagnostic`
- `lowering-diagnostic`
- `verifier-rejection`
- `vm-trap`
- `project-diagnostic`
- `boundary-denial`

Rules:

- compile/check diagnostics are not VM traps.
- project diagnostics are not project-root execution traps unless project-root command exists and the stage contract says so.
- verifier rejection must mean SemCode admission failed.
- VM trap must mean verified execution reached VM and trapped.
- line/column may be null if not stable.
- exact full message should not be required if only code/needle is stable.

## 9. Evidence Object Contract

```json
{
  "id": "E001",
  "class": "E2-test",
  "source": "tests/...",
  "description": "canonical fixture passed check/compile/verify/run"
}
```

Allowed evidence classes:

- `E0-doc`
- `E1-code`
- `E2-test`
- `E3-trace`
- `E4-replay`
- `E5-release-gate`

Rules:

- 7hell may reference CTF evidence classes.
- Report must distinguish ordinary test evidence from golden trace evidence.
- 7hell result must not imply CTF freeze unless CTF docs say so.
- CTF-E1 / E2 / E3 artifacts may be referenced later, but WP1 does not add references to live artifacts unless existing stable IDs are already intended for report contract.

## 10. CTF Reference Contract

```json
{
  "area": "verifier-first",
  "status": "checked",
  "source": "docs/roadmap/language_maturity/core_trust_freeze/verifier_first_policy.md"
}
```

Allowed CTF areas:

- `runtime-value`
- `trap-taxonomy`
- `determinism`
- `verifier-first`
- `golden-trace`
- `capability-effect`
- `project-root-trust`

Rules:

- CTF reference is not CTF closure.
- 7hell consumes CTF policy; it does not replace it.
- A passing 7hell report must not imply release readiness unless release gate policy says so.

## 11. Boundary Object Contract

```json
{
  "id": "B001",
  "scope": "project-root",
  "status": "out-of-scope",
  "reason": "project-root command not implemented"
}
```

Use boundaries to record:

- project-root out of scope;
- semantic.toml out of scope;
- smc new out of scope;
- package registry out of scope;
- remote dependencies out of scope;
- UI / Workbench out of scope.

## 12. Project-Root Future Report Shape

Future target shape:

```json
{
  "kind": "project-root",
  "display": ".",
  "normalized": ".",
  "manifest": "semantic.toml",
  "entry": "src/main.sm"
}
```

Rules:

- not implemented by WP1.
- depends on PCC-9I.
- must follow `project_root_trust_policy.md`.
- must preserve verifier-first route.
- must add traces / replay before readiness.

## 13. Determinism Rules

Report must be deterministic:

- same input;
- same Semantic version;
- same runtime config;
- same profile;
- same capability context.

Do not include by default:

- wall-clock time;
- hostnames;
- absolute paths;
- temp dirs;
- nondeterministic map ordering;
- environment variables;
- telemetry IDs.

If timing is later added:

- it must be under explicit `metrics` or `debug` mode;
- it must not affect pass / fail.

## 14. Report Versioning

Schema version:

```text
semantic.7hell.report.v0
```

Rules:

- backward-incompatible changes require version bump.
- additive fields allowed only if deterministic.
- removal / rename requires migration note.
- report consumers must treat unknown fields conservatively.

## 15. Minimal v0 Report Acceptance

A future implementation of v0 report is acceptable only when:

- human output has stable stage names;
- JSON output has required top-level fields;
- stage statuses are stable;
- overall result rules are implemented;
- diagnostics distinguish failure layers;
- verifier-first relationship is visible;
- project-root unsupported state is explicit;
- no release readiness claim is made.

## 16. Stop Conditions for Future Implementation

Implementation must stop if:

1. report requires project-root support before PCC-9I;
2. report needs semantic.toml parser before it exists;
3. report needs exact unstable diagnostic text;
4. report includes absolute paths by default;
5. report includes timing by default;
6. report treats CTF references as CTF closure;
7. report treats pass-with-skips as full pass without policy;
8. report mixes compile diagnostics and VM traps;
9. report emits telemetry;
10. report creates release gate behavior before policy exists.

## 17. Final Verdict

```text
7HELL-WP1 defines the qualification report contract.
It does not implement 7hell.
It does not create a release gate.
It does not claim readiness.
```

## 18. Acceptance Checklist

```markdown
- [ ] report model defined
- [ ] human report shape defined
- [ ] JSON report shape defined
- [ ] target object defined
- [ ] stage record contract defined
- [ ] result rules defined
- [ ] diagnostic object contract defined
- [ ] evidence object contract defined
- [ ] CTF reference contract defined
- [ ] boundary object contract defined
- [ ] project-root future shape marked non-implemented
- [ ] determinism rules defined
- [ ] report versioning defined
- [ ] no implementation added
- [ ] no release gate claimed
- [ ] no readiness claimed
```

## PCC Evidence Mapping

Future report evidence objects may reference the PCC-4..PCC-9 stage mapping:

- `7hell_pcc4_pcc9_stage_mapping.md`

The mapping defines which PCC evidence belongs to each 7hell stage.
This mapping is docs-only and does not imply the skeleton command executes mapped stages.

When stages become executable, report status transitions must follow `7hell_skeleton_to_runner_transition.md`.

WP2 does not implement report generation.
