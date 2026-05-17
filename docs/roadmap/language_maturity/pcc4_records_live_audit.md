# PCC-4 Records Live Audit

Status: live audit
Owner: language maturity stream
Scope: records readiness before PCC-4 implementation
Non-goal: code changes

## 1. Purpose

This document is a live readiness audit for records before PCC-4 implementation work begins.

It is docs-only. It does not change parser, typecheck, lowering, SemCode, verifier, VM, diagnostics, tests, or examples.

## 2. Current Known Status

Records are already present as a nominal aggregate family across the current executable surface.

Observed surface:

- syntax for `record` declarations exists in the frontend parser;
- record literals, field access, update/copy-with, destructuring bind, and `let-else` exist in the frontend AST and typecheck path;
- records have nominal type identity through `Type::Record(SymbolId)`;
- field validation, acyclic-declaration checks, and stable-equality gating exist in typecheck;
- lowering emits canonical `MakeRecord` and `RecordGet` IR;
- `sm-ir` encodes the record operations as fixed opcodes;
- `sm-verify` recognizes and validates record opcodes and their payload shape;
- `sm-vm` executes records as `Value::Record(RecordCarrier<Value>)`;
- record values are intentionally not part of the PROMETHEUS host ABI surface;
- diagnostics exist for unknown record types, duplicate fields, missing fields, out-of-bounds access, and unsupported conversions such as record-to-text;
- record-oriented unit coverage already exists in `sm-verify`, `sm-vm`, and lowering tests;
- the current repository does not appear to have a dedicated `tests/pcc4_*` or `tests/fixtures/pcc4_*` suite yet.

Conclusion:

```text
The core record seams already exist.
PCC-4 can start with fixture/closeout work rather than new parser/lowering/VM architecture.
```

## 3. Readiness Matrix

| Layer | Required for PCC-4 | Current state | Ready? | Next action |
| --- | --- | --- | --- | --- |
| parser | record declarations / literals / field access | Nominal record syntax and expression forms already exist | yes | lock acceptance fixtures |
| typecheck | record types and field validation | Nominal record typing, uniqueness checks, acyclic checks, and equality gating already exist | yes | keep diagnostics stable |
| lowering | record construction/access to IR | `MakeRecord` and `RecordGet` lowering already exist | yes | verify fixture coverage |
| SemCode | stable representation or lowering strategy | `MakeRecord` / `RecordGet` opcodes exist in the local format | yes | keep opcode contract frozen |
| verifier | accepts record-related bytecode safely | verifier recognizes record opcodes and validates payload shape | yes | keep verifier diagnostics aligned |
| VM | runtime record value behavior | `Value::Record(RecordCarrier<Value>)` already executes deterministically | yes | keep host ABI boundary closed |
| diagnostics | clear source errors | record-specific parse/typecheck/runtime diagnostics already exist | yes | keep error text stable |
| tests | positive/negative coverage | unit coverage exists, but no dedicated PCC-4 fixture suite is present yet | partial | add PCC-4 fixture lock / closeout coverage |

## 4. Risk List

Observed risks:

- record support is distributed across core layers and current tests, but there is no dedicated PCC-4 fixture suite yet;
- a PCC-4 implementation PR could accidentally drift into adjacent aggregate work such as ADT or collections if the scope is not frozen;
- record values are still not part of the PROMETHEUS host ABI surface, so host-effect widening would be out of scope for PCC-4;
- record diagnostics are already present, but a closeout PR should keep their wording stable so the current acceptance evidence remains readable;
- the record surface is currently nominal and slot-based, so any attempt to introduce a general object model would be a scope break.

Not observed:

- no evidence of missing parser support for nominal records;
- no evidence of missing lowering for record construction or field access;
- no evidence of missing SemCode opcodes for record construction or access;
- no evidence that the VM lacks a runtime carrier for records.

## 5. Recommended PCC-4 Split

Because the core seams are already present, the next work is better split as fixture and closeout coverage rather than a new architecture build.

Recommended sequence:

```text
PCC-4B — lock canonical record acceptance fixtures across parser / typecheck / lowering / verify / VM
PCC-4C — expand record diagnostics and negative fixtures
PCC-4D — close PCC-4 with evidence sync and roadmap status update
```

If a future implementation delta is discovered during PCC-4B, split it narrowly from the fixture lock before landing.

## 6. Out of Scope

This audit does not expand into:

- ADT;
- collections;
- methods;
- traits;
- inheritance;
- reflection;
- serialization;
- general object systems;
- heap GC;
- UI / Workbench / Studio;
- Linguist;
- README promotion;
- host ABI widening.

## 7. Acceptance Checklist

- [x] parser surface inspected
- [x] typecheck inspected
- [x] lowering inspected
- [x] SemCode / verifier inspected
- [x] VM runtime inspected
- [x] diagnostics inspected
- [x] tests inspected
- [x] risks documented
- [x] next PCC-4 split proposed
- [x] no code changed

