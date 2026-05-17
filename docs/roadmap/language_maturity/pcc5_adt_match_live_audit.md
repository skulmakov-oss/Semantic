# PCC-5 ADT + Basic Match Live Audit

Status: live audit
Owner: language maturity stream
Scope: ADT + basic match readiness before PCC-5 implementation
Non-goal: code changes

## 1. Purpose

This document audits current ADT and basic match readiness on `main` before any PCC-5 implementation work.

It is docs-only. It does not add ADT or match behavior.

## 2. Current Known Status

Current `main` already contains the ADT and basic match seam across the full practical stack:

- Parser support exists for `enum` declarations, constructor expressions, and enum-style `match` patterns.
- The frontend AST already models ADTs, ADT constructors, ADT match patterns, and match arms explicitly.
- Type checking validates ADT declarations, constructor payloads, match arms, and non-exhaustive match policy.
- Lowering already emits ADT constructor and match IR.
- SemCode already has ADT opcodes and the verifier recognizes them.
- The VM already carries ADT runtime values and executes ADT constructor/tag/access instructions.
- Diagnostics for invalid ADT and match shapes already exist.
- Existing tests already exercise the surface end-to-end through the snake benchmark path.

Audit verdict:

```text
The core ADT + basic match seams are already present in main.
PCC-5 does not need a new architecture seam before implementation.
The remaining gap is evidence packaging: dedicated PCC-5 fixture / closeout documentation.
```

CTF touched: none
Reason: docs-only audit; no runtime value, trap, determinism, verifier, SymbolId, capability, or trace change.

## 3. Readiness Matrix

| Layer | Required for PCC-5 | Current state | Ready? | Next action |
| --- | --- | --- | --- | --- |
| parser | enum / ADT declarations | confirmed-working | yes | keep as fixture-backed evidence |
| parser | constructor expressions | confirmed-working | yes | keep as fixture-backed evidence |
| parser | match syntax | confirmed-working | yes | keep as fixture-backed evidence |
| typecheck | nominal ADT typing | confirmed-working | yes | keep as fixture-backed evidence |
| typecheck | constructor payload validation | confirmed-working | yes | keep as fixture-backed evidence |
| typecheck | match arm validation | confirmed-working | yes | keep as fixture-backed evidence |
| exhaustiveness | policy or explicit limitation | confirmed-working | yes | keep explicit limitation documented |
| lowering | constructor / match lowering | confirmed-working | yes | keep as fixture-backed evidence |
| SemCode | stable representation | confirmed-working | yes | keep opcode mapping and verifier coverage stable |
| verifier | validates ADT/match bytecode | confirmed-working | yes | keep bytecode checks stable |
| VM | executes ADT/match runtime values | confirmed-working | yes | keep runtime carrier behavior stable |
| diagnostics | clear ADT/match errors | confirmed-working | yes | keep diagnostic needles stable |
| tests | positive/negative coverage | confirmed-partial | partial | add PCC-5-named fixture suite if desired |

## 4. Risk List

Observed risks are narrow, not architectural:

- ADT can be confused with records if roadmap wording drifts.
- Match exhaustiveness can become too broad if the phase is widened beyond basic enum shapes.
- Option / Result should stay as separate phase work even though they reuse the ADT substrate.
- Constructor payload access must remain explicit and not turn into a generic object or reflection model.
- ADT representation must not leak into the PROMETHEUS host ABI.
- The current evidence is spread across existing benchmarks and unit tests, not a dedicated `pcc5` fixture suite yet.

## 5. Recommended PCC-5 Split

The audit does not point to a missing core seam that requires a new parser/typecheck/lowering redesign.

Recommended next split:

```text
PCC-5B — test(adt): lock canonical ADT declaration / constructor fixtures
PCC-5C — test(match): lock basic ADT match fixtures
PCC-5D — test(adt): lock negative ADT/match diagnostics fixtures
PCC-5E — docs(adt): close PCC-5 with evidence sync and roadmap status update
```

If a dedicated fixture suite becomes necessary for roadmap hygiene, add it as evidence work, not as a new language design PR.

## 6. Out of Scope

- records
- collections
- Option / Result standardization
- advanced pattern matching
- exhaustive pattern checker beyond the current basic policy
- methods / traits
- generics
- object system
- host ABI
- serialization
- reflection
- UI / Workbench

## 7. Acceptance Checklist

- [x] parser surface inspected
- [x] AST/frontend model inspected
- [x] typecheck inspected
- [x] constructor support inspected
- [x] match/pattern support inspected
- [x] exhaustiveness policy inspected
- [x] lowering inspected
- [x] SemCode/verifier inspected
- [x] VM runtime inspected
- [x] diagnostics inspected
- [x] tests inspected
- [x] risks documented
- [x] PCC-5 split proposed
- [x] no code changed
