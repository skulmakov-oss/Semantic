# PCC-4 Records Live Audit

Status: PCC-4B in progress
Owner: language maturity stream
Scope: records readiness and fixture lock
Non-goal: record architecture redesign

## 1. Purpose

This document is a live readiness audit for records before PCC-4 implementation work begins.

PCC-4A was docs-only. It did not change parser, typecheck, lowering, SemCode, verifier, VM, diagnostics, tests, or examples.

PCC-4B adds positive canonical acceptance fixtures only. It does not redesign record architecture or widen ADT, schema, PROMETHEUS host ABI, UI, Workbench, or runtime ownership scope.

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
- before PCC-4B, the repository did not have a dedicated `tests/pcc4_*` or `tests/fixtures/pcc4_*` suite.

Conclusion:

```text
The core record seams already exist.
PCC-4 starts with fixture/closeout work rather than new parser/lowering/VM architecture.
```

## 3. Readiness Matrix

| Layer | Required for PCC-4 | Current state | Ready? | Next action |
| --- | --- | --- | --- | --- |
| parser | record declarations / literals / field access | Nominal record syntax and expression forms already exist | yes | locked by PCC-4B positive fixtures |
| typecheck | record types and field validation | Nominal record typing, uniqueness checks, acyclic checks, and equality gating already exist | yes | keep diagnostics stable |
| lowering | record construction/access to IR | `MakeRecord` and `RecordGet` lowering already exist | yes | locked through compile path |
| SemCode | stable representation or lowering strategy | `MakeRecord` / `RecordGet` opcodes exist in the local format | yes | keep opcode contract frozen |
| verifier | accepts record-related bytecode safely | verifier recognizes record opcodes and validates payload shape | yes | locked through verify path |
| VM | runtime record value behavior | `Value::Record(RecordCarrier<Value>)` already executes deterministically | yes | locked through run path |
| diagnostics | clear source errors | record-specific parse/typecheck/runtime diagnostics already exist | yes | expand in PCC-4C negative fixtures |
| tests | positive/negative coverage | PCC-4B adds dedicated positive acceptance coverage | partial | add PCC-4C negative diagnostics fixtures |

## 4. PCC-4B Evidence

PCC-4B adds `tests/pcc4_records_acceptance.rs` as the dedicated positive acceptance fixture lock.

The fixture suite covers three canonical positive slices:

1. `pcc4_record_declaration.sm`
   - locks a nominal `record` declaration through the CLI path;
   - proves the parser/typecheck pipeline accepts a standalone record declaration.
2. `pcc4_record_construction_and_field_read.sm`
   - locks record literal construction;
   - locks field reads through `assert(pair.left == ...)` and `assert(pair.right == ...)`;
   - exercises lowering, SemCode emission, verifier admission, and VM execution for `MakeRecord` / `RecordGet`.
3. `pcc4_record_function_boundary.sm`
   - locks a record value crossing a function boundary;
   - reads a field inside the callee;
   - verifies deterministic execution through `check -> run -> compile -> verify`.

The test helper intentionally uses the same CLI-stage shape as the canonical example tests:

```text
smc check
smc run
smc compile -o out.smc
smc verify out.smc
```

This makes PCC-4B an end-to-end acceptance lock rather than a parser-only fixture.

## 5. Risk List

Observed risks:

- record support is distributed across core layers and current tests, but before PCC-4B there was no dedicated PCC-4 fixture suite;
- a PCC-4 implementation PR could accidentally drift into adjacent aggregate work such as ADT or collections if the scope is not frozen;
- record values are still not part of the PROMETHEUS host ABI surface, so host-effect widening remains out of scope for PCC-4;
- record diagnostics are already present, but PCC-4C should keep their wording stable so acceptance evidence remains readable;
- the record surface is currently nominal and slot-based, so any attempt to introduce a general object model would be a scope break.

Not observed:

- no evidence of missing parser support for nominal records;
- no evidence of missing lowering for record construction or field access;
- no evidence of missing SemCode opcodes for record construction or access;
- no evidence that the VM lacks a runtime carrier for records.

## 6. Recommended PCC-4 Split

Because the core seams are already present, the next work is better split as fixture and closeout coverage rather than a new architecture build.

Recommended sequence:

```text
PCC-4B — lock canonical positive record acceptance fixtures across parser / typecheck / lowering / verify / VM
PCC-4C — expand record diagnostics and negative fixtures
PCC-4D — close PCC-4 with evidence sync and roadmap status update
```

If a future implementation delta is discovered during PCC-4B or PCC-4C, split it narrowly from the fixture lock before landing.

## 7. Out of Scope

This audit and PCC-4B do not expand into:

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

## 8. Acceptance Checklist

- [x] parser surface inspected
- [x] typecheck inspected
- [x] lowering inspected
- [x] SemCode / verifier inspected
- [x] VM runtime inspected
- [x] diagnostics inspected
- [x] tests inspected
- [x] risks documented
- [x] next PCC-4 split proposed
- [x] PCC-4B positive fixture suite added
- [x] record declaration positive fixture added
- [x] record construction + field read positive fixture added
- [x] record function-boundary positive fixture added
- [x] no record architecture redesign

