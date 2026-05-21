# Semantic 7hell Qualification Contract

Status: seed contract
Track: PCC-0.6 7hell Skeleton Seed
Layer: language maturity / qualification harness
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `practical_core_truth_reset.md`
- `practical_core_feature_matrix_live_audit.md`
- `practical_core_completion_v0_3.md`
- `core_trust_freeze/index.md`
- `7hell_report_contract.md`

## 1. Purpose

This document seeds the `7hell` qualification contract for Semantic practical
core readiness.

`7hell` is not a decorative command name. It is the progressive qualification
gauntlet that proves a Semantic program can survive the full practical execution
path with stable diagnostics.

Core formula:

```text
smc 7hell program.sm
  = syntax + type + lowering + verifier + VM + practical + diagnostics gauntlet
```

This document defines the target command shape, stage taxonomy, output contract,
and growth rules before implementation begins.

## 2. Position in PCC

Current ladder:

```text
PCC-0 Truth Reset
  ↓
PCC-0.5 Feature Matrix Live Audit
  ↓
PCC-0.6 7hell Skeleton Seed
  ↓
PCC language phases
  ↕
CTF Core Trust Freeze Lane
```

PCC-0.6 is still a readiness gate. It does not implement the command.

The command implementation may only begin after this contract is accepted and
after PCC-0.5 has identified which fixtures must be covered first.

## 3. Command shape

Canonical command:

```bash
smc 7hell program.sm
```

Long alias:

```bash
smc seven-hell program.sm
```

Future project-level shape:

```bash
smc 7hell
smc 7hell --project .
smc 7hell --all
```

Only the single-file shape is required for the seed contract.

## 4. Non-goals

This document does not implement:

- CLI command parsing;
- test fixtures;
- stage runner code;
- JSON serializer;
- diagnostics renderer;
- CI integration;
- Workbench integration;
- UI panels;
- package builder;
- new language features.

Rule:

```text
7hell contract first, command implementation later.
```

## 5. Stage taxonomy

`7hell` consists of seven stages.

| Stage | Name | Purpose |
|---:|---|---|
| 1 | Syntax Hell | Prove lexer/parser behavior and syntax diagnostics. |
| 2 | Type Hell | Prove type system, scopes, binding, and semantic diagnostics. |
| 3 | Lowering Hell | Prove AST/typed model lowers to deterministic IR. |
| 4 | Verifier Hell | Prove emitted SemCode passes or fails admission correctly. |
| 5 | VM Hell | Prove verified SemCode executes with stable traps and quotas. |
| 6 | Practical Hell | Prove real small programs can complete the full path. |
| 7 | User Pain / Diagnostics Hell | Prove failures are understandable and actionable. |

Stage names are stable. Implementations may be shallow at first, but the stage
contract should not be renamed casually.

## 6. Stage 1 — Syntax Hell

Purpose:

- validate lexical handling;
- validate parser acceptance/rejection;
- validate indentation or block-shape behavior where relevant;
- validate syntax-level diagnostics.

Typical fixture classes:

- valid minimal function;
- comments and blank lines;
- nested blocks;
- invalid token;
- malformed declaration;
- unterminated string;
- bad delimiters.

Expected outcome:

```text
valid syntax -> pass
invalid syntax -> stable diagnostic
```

## 7. Stage 2 — Type Hell

Purpose:

- validate type checking;
- validate quad vs bool boundaries;
- validate numeric compatibility;
- validate binding and assignment rules;
- validate records / ADT / collections as PCC adds them.

Typical fixture classes:

- bool condition;
- rejected `if quad_expr` if applicable;
- incompatible assignment;
- invalid return type;
- missing symbol;
- shadowing or scope cases;
- unsupported type shape.

Expected outcome:

```text
valid typed source -> pass
invalid typed source -> stable semantic diagnostic
```

## 8. Stage 3 — Lowering Hell

Purpose:

- validate deterministic lowering;
- validate labels and jumps;
- validate temporary/register shape;
- validate terminal CFG behavior;
- validate that unsupported constructs fail before unstable lowering.

Typical fixture classes:

- if/else lowering;
- match lowering;
- loop lowering once PCC-1 lands;
- return path;
- unreachable shape if applicable;
- invalid lowering precondition.

Expected outcome:

```text
typed source -> deterministic IR or stable rejection
```

## 9. Stage 4 — Verifier Hell

Purpose:

- validate verifier-first admission;
- validate SemCode header/version expectations;
- validate opcode layout;
- validate branch targets;
- validate capability requirements;
- validate malformed SemCode rejection where fixture support exists.

Typical fixture classes:

- valid emitted SemCode;
- invalid opcode fixture;
- invalid jump target fixture;
- missing capability fixture;
- bad header fixture;
- resource limit fixture.

Expected outcome:

```text
valid SemCode -> admitted
invalid SemCode -> rejected before VM execution
```

Hard rule:

```text
VM Hell must not run unverified SemCode.
```

## 10. Stage 5 — VM Hell

Purpose:

- validate execution of verified SemCode;
- validate runtime values;
- validate stack/frame behavior;
- validate traps;
- validate quotas;
- validate deterministic result behavior.

Typical fixture classes:

- return value;
- function call;
- arithmetic path;
- branch path;
- trap path;
- quota path;
- host/capability denial path only when explicitly in scope.

Expected outcome:

```text
verified SemCode -> deterministic result or stable trap
```

## 11. Stage 6 — Practical Hell

Purpose:

- validate that ordinary small programs work end-to-end;
- validate canonical examples;
- validate practical language combinations;
- validate no hidden reliance on implementation accidents.

Typical fixture classes:

- calculator-like program;
- decision program;
- small data transform;
- record example once PCC-4 lands;
- ADT/Result example once PCC-5/PCC-6 land;
- collection example once PCC-7 lands.

Expected outcome:

```text
source -> check -> compile -> verify -> run-smc -> expected result
```

## 12. Stage 7 — User Pain / Diagnostics Hell

Purpose:

- validate that failures are understandable;
- validate diagnostic code stability;
- validate caret/source context where available;
- validate help messages;
- validate that unsupported constructs explain the boundary.

Typical fixture classes:

- syntax error with caret;
- type mismatch with expected/found;
- unsupported feature with explicit message;
- verifier rejection with reason;
- runtime trap with stable category;
- project-level error once PCC-9 lands.

Expected outcome:

```text
bad input -> stable, actionable diagnostic
```

## 13. Output contract

Human output should be concise and stage-oriented.

Representative shape:

```text
Semantic 7hell qualification
target: examples/foo.sm

[1/7] Syntax Hell              PASS
[2/7] Type Hell                PASS
[3/7] Lowering Hell            PASS
[4/7] Verifier Hell            PASS
[5/7] VM Hell                  PASS
[6/7] Practical Hell           PASS
[7/7] User Pain / Diagnostics  PASS

result: PASS
```

Failure shape:

```text
[2/7] Type Hell                FAIL
code: E0201
reason: expected bool condition, found quad
next: inspect diagnostic output

result: FAIL
```

The exact formatting can change during implementation, but the stage model and
terminal result must remain stable.

## 14. JSON output contract

Future JSON form:

```json
{
  "tool": "smc 7hell",
  "target": "program.sm",
  "result": "pass",
  "stages": [
    {
      "index": 1,
      "name": "Syntax Hell",
      "status": "pass",
      "evidence": [],
      "diagnostics": []
    }
  ]
}
```

Allowed stage statuses:

```text
pass
fail
skip
not_implemented
blocked
```

Rules:

- `fail` means the stage ran and found a failure;
- `skip` means the stage was intentionally skipped by policy or command flags;
- `not_implemented` is allowed only during early skeleton phase;
- `blocked` means an earlier stage prevents this stage from running.

Overall result:

| Stage states | Overall result |
|---|---|
| all pass | `pass` |
| any fail | `fail` |
| any blocked and no fail | `blocked` |
| only pass/skip | `pass-with-skips` |
| any not_implemented | `incomplete` |

## 15. Report Contract Split

The stage taxonomy and command intent live in this document.

The stable report shape is defined in:

- `7hell_report_contract.md`

`7hell_report_contract.md` owns:

- human report structure;
- JSON report schema;
- diagnostic object shape;
- evidence references;
- CTF references;
- boundary records;
- versioning rules.

This split does not implement 7hell.

## 16. Fixture Growth Rule

Each PCC phase must add or assign fixtures to `7hell`.

Minimum mapping:

| PCC phase | Required 7hell growth |
|---|---|
| PCC-1 Control Flow | loop / break / continue fixtures in Syntax, Lowering, VM, Diagnostics. |
| PCC-2 Numeric Core | arithmetic, relation, division/trap fixtures in Type, VM, Diagnostics. |
| PCC-3 Text Core | literal/equality/concat/length fixtures in Type, VM, Practical. |
| PCC-4 Records | declaration/construction/field access fixtures in Type, Lowering, Practical. |
| PCC-5 ADT + Match | constructor/match/exhaustiveness or limitation fixtures. |
| PCC-6 Option / Result | success/failure flow fixtures. |
| PCC-7 Collections | index/iteration/bounds/missing-key fixtures. |
| PCC-8 Stdlib | assert/to_text/helper failure fixtures. |
| PCC-9 Project Model | project-level check/run/diagnostics fixtures. |

Rule:

```text
No PCC feature is complete until its 7hell coverage decision is recorded.
```

The decision may be:

- fixture added;
- fixture assigned to follow-up PR;
- not applicable with reason.

## 17. CTF relationship

`7hell` does not replace CTF.

CTF owns trust documents such as:

- runtime value registry;
- trap taxonomy;
- determinism matrix;
- verifier-first policy;
- golden trace policy;
- capability/effect denial matrix.

`7hell` consumes those decisions as executable or checkable qualification
pressure.

Required PR footer for future 7hell implementation PRs:

```text
CTF touched:
  - <file>
7hell coverage:
  - <stage>
```

or:

```text
CTF touched: none
7hell coverage: contract/docs only
```

## 18. Blocking rule

PCC-1 remains blocked until the following are true:

```text
[ ] PCC-0 Truth Reset exists
[ ] PCC-0.5 Feature Matrix Live Audit scaffold exists
[ ] PCC-0.6 7hell qualification contract exists
[ ] CTF directory exists
```

After this document lands, the remaining blocker is not the contract itself, but
whether the team chooses to implement the skeleton before or during the first
PCC-1 implementation PR.

## 19. Out of scope

Out of scope for this contract:

- deciding exact CLI parser internals;
- deciding exact test harness crate layout;
- deciding CI workflow names;
- adding all fixtures now;
- making `7hell` a release gate immediately;
- UI visualization of 7hell;
- Workbench integration.

## 20. Acceptance checklist

This PR is complete when:

- `7hell` purpose is defined;
- command shape is documented;
- seven stages are named and explained;
- output contract is sketched;
- JSON contract is sketched;
- stage status vocabulary is defined;
- fixture growth rule is defined;
- PCC phase mapping is defined;
- CTF relationship is explicit;
- implementation remains out of scope;
- no code is changed.

## 21. Final state

After this document:

```text
PCC-0 Truth Reset = landed
PCC-0.5 Live Audit = scaffolded
PCC-0.6 7hell Qualification Contract = seeded
PCC-1 Control Flow = still blocked until audit and skeleton decisions are handled
```

## PCC-4..PCC-9 Stage Mapping

The PCC-4..PCC-9 evidence-to-stage mapping is defined in:

- `7hell_pcc4_pcc9_stage_mapping.md`

The fixture growth rule is constrained by `7hell_pcc4_pcc9_stage_mapping.md`, which maps PCC-4..PCC-9 evidence into 7hell stages.

7HELL-WP2 is docs-only; implementation remains future work.
