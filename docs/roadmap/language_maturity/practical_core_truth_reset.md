# Semantic Practical Core Truth Reset

Status: active readiness gate
Track: PCC-0 Truth Reset + Live Audit
Layer: language maturity / readiness discipline
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `practical_core_completion_v0_3.md`
- `core_trust_freeze/index.md`
- `core_trust_freeze/runtime_value_registry.md`
- `core_trust_freeze/trap_taxonomy.md`
- `core_trust_freeze/determinism_matrix.md`
- `../../spec/ui/README.md`
- `../../spec/ui/local_runtime_skeleton_api_map.md`
- `../../spec/ui/local_runtime_command_result_envelope.md`
- `../../spec/ui/local_runtime_event_stream_contract.md`

## 1. Purpose

This document resets the current readiness posture after the UI documentation
phase v0 closure.

Its purpose is to prevent the project from moving from a documented UI boundary
into premature implementation work while the practical Semantic core still needs
a live truth audit and controlled completion path.

Core formula:

```text
UI boundary is frozen at docs phase v0.
Next work returns to Semantic practical core readiness.
No new UI/runtime implementation before PCC-0 / PCC-0.5 / 7hell seed.
```

## 2. Current transition

The UI docs phase v0 is now considered complete.

Phase composition:

```text
I67  local runtime API map
I68  command/result envelope
I69  event stream contract
```

The phase closed with the local runtime event stream contract. That document is
part of the UI/local runtime boundary documentation set, not an implementation
start signal.

This means the UI surface has enough written boundary material for now. The next
work must return to the language and execution core.

## 3. Frozen UI boundary

The UI boundary is frozen at documentation phase v0.

Hard stop:

```text
no I70
no Workbench implementation
no Tauri/runtime code
no package builder
no UI runtime widening
no native backend implementation
no renderer implementation
no event bus implementation
```

Allowed UI work during this freeze:

- typo fixes in existing UI docs;
- link corrections;
- contradiction fixes;
- explicit clarification that preserves the current boundary.

Forbidden UI work during this freeze:

- new Workbench behavior;
- new local runtime structs;
- command handler code;
- runtime event bus code;
- Tauri integration;
- packaging/distribution implementation;
- renderer/backend implementation;
- UI capability expansion.

Rule:

```text
UI docs phase v0 complete does not mean UI implementation may begin.
```

## 4. Current project truth posture

Semantic must now use a stricter truth vocabulary.

The project may describe components only as one of the following:

| Status | Meaning |
|---|---|
| `working` | Confirmed by current code, tests, fixtures, or full pipeline evidence. |
| `partial` | Exists, but missing edges or limitations are explicitly known. |
| `documented-only` | Documented contract or plan exists, but no implementation claim is made. |
| `experimental` | Useful research/donor/substrate exists, but it is not canonical readiness. |
| `out-of-scope` | Explicitly excluded from the current phase. |

Avoid mixed optimistic status labels such as:

```text
implemented / maybe partial
closed but needs audit
ready but not checked
landed but unverified
```

Those must be resolved by PCC-0.5 live audit.

## 5. Why PCC-0 comes next

The correct ladder is:

```text
Current Semantic
  ↓
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

Reason:

New feature work before a truth reset would widen the project on top of an
uncertain readiness map. That creates false closure: docs look mature while the
practical language core still has unresolved live-state gaps.

PCC-0 therefore exists to freeze the wording, sequence, and boundaries before
PCC-1 begins.

## 6. Practical core priority

The active priority is now:

```text
Semantic practical core readiness
```

This means work should target:

- control-flow completeness;
- numeric completeness;
- text core;
- records end-to-end;
- ADT + basic match;
- Option / Result;
- collections v0;
- stdlib v0;
- project model v0;
- verifier-first execution discipline;
- deterministic runtime behavior;
- 7hell qualification.

It does not mean:

- Workbench implementation;
- UI application capability;
- graphics/rendering;
- package builder;
- LLM / TinyLM research;
- hardware/backend acceleration;
- PROMETHEUS runtime widening without a separate scope gate.

## 7. Live audit requirement

Before new widening features begin, the project must perform a live feature
matrix audit against current `main`.

The audit must answer:

```text
what really works
what partially works
what is only documented
what is experimental
what is out of scope
```

For each audited item, the output must identify:

- current status;
- evidence source;
- missing edge if partial;
- owning PCC phase if work remains;
- whether CTF material is affected;
- whether 7hell coverage is needed.

No feature may advance from assumed readiness to stable readiness without live
evidence.

## 8. CTF runs in parallel

Core Trust Freeze is not a later cleanup phase. It runs in parallel with PCC.

Every practical core change must state whether it touches:

- runtime value registry;
- trap taxonomy;
- determinism matrix;
- SymbolId migration assumptions;
- verifier-first policy;
- golden trace policy;
- capability/effect denial behavior.

Required PR footer shape:

```text
CTF touched:
  - <file>
```

or:

```text
CTF touched: none
Reason: docs-only / parser-only / no runtime impact
```

This prevents the execution core from silently drifting while the language
surface expands.

## 9. 7hell is a qualification gate

`7hell` must be treated as a progressive qualification gate, not as a decorative
command.

Target role:

```text
smc 7hell program.sm
  = syntax + type + lowering + verifier + VM + practical + diagnostics gauntlet
```

Initial stages:

1. Syntax Hell
2. Type Hell
3. Lowering Hell
4. Verifier Hell
5. VM Hell
6. Practical Hell
7. User Pain / Diagnostics Hell

PCC-0.6 should seed the command contract and stage taxonomy early. Each PCC
phase then adds fixtures to the relevant stages.

## 10. Readiness claim discipline

Do not claim:

```text
Semantic is ready
Semantic core is fully trusted
Semantic practical core is complete
UI runtime may begin
Workbench may begin
```

until the required gates have evidence.

Allowed claim after this document:

```text
UI docs phase v0 is complete.
Semantic has returned to PCC-0 truth reset and practical core readiness work.
```

## 11. Next PR sequence

Recommended immediate sequence:

```text
PR-PCC-0A  docs(readiness): add Semantic practical core truth reset
PR-PCC-0B  docs(readiness): add live feature matrix audit scaffold
PR-PCC-0C  docs(7hell): seed 7hell qualification contract
```

PCC-1 must not start until:

```text
[ ] PCC-0 Truth Reset exists
[ ] PCC-0.5 Feature Matrix Live Audit exists
[ ] PCC-0.6 7hell Skeleton Seed exists
[ ] CTF directory exists and is referenced by PCC work
```

## 12. Acceptance checklist

This PR is complete when:

- UI docs phase v0 is marked complete;
- I67 / I68 / I69 are named as the closed UI docs set;
- UI/runtime implementation is explicitly frozen;
- Workbench implementation is explicitly out of scope;
- PCC-0 is named as the next active track;
- PCC-0.5 live audit is required before widening;
- CTF is described as parallel to PCC;
- `7hell` is described as a qualification gate;
- readiness wording is corrected away from optimistic overclaim;
- no code is changed.

## 13. Final state

After this document:

```text
UI docs phase v0 = complete
UI implementation = frozen
Semantic practical core = active priority
PCC-0 = current gate
PCC-0.5 = next audit gate
PCC-0.6 = qualification harness seed
CTF = parallel trust lane
```
