# Tuple / Sequence Ownership Audit

## 1. Executive Verdict

Tuple ownership is now fully qualified for direct tuple element ownership
paths. This audit remains the original tuple/sequence split document, but the
tuple half should now be read alongside the tuple closeout matrix:
`docs/roadmap/pcc/tuple_ownership_matrix.md`.

Sequence ownership is still not a qualified ownership slice. The repository has
evidence for sequence literals, indexing, iteration, and runtime execution, but
not for a dedicated sequence ownership-path vocabulary or sequence ownership
E2E evidence.

Current verdict:

- Tuple ownership: **COMPLETE / PASS for direct tuple element ownership paths**
- Sequence ownership: **PARTIAL / NOT PROVEN**

Do not claim sequence ownership as equivalent to tuple ownership.

## 2. Current Support Matrix

| Subsystem | Tuple support | Sequence support | Evidence | Status | Gaps |
| --- | --- | --- | --- | --- | --- |
| Parser | READY | MOSTLY READY | `PatternPath::tuple_index`, `PatternPathElem::TupleIndex`; `Expr::SequenceLiteral`, `Expr::SequenceIndex`, `Stmt::ForEach` | Tuple READY; Sequence MOSTLY READY | Sequence syntax exists, but not as a dedicated ownership path family |
| Typecheck | READY | MOSTLY READY | `PatternPathElem::TupleIndex`; `expr_access_path_sequence_index_literal` maps literal sequence index to `tuple_index`; `infer_sequence_index_type` | Tuple READY; Sequence MOSTLY READY | Sequence index is treated as tuple-like access-path material, not sequence ownership vocabulary |
| Pattern conflict planning | READY | PARTIAL | Tuple path availability / capture tests in `crates/sm-front/src/typecheck.rs`; sequence index path extraction exists, but no sequence ownership conflict suite | Tuple READY; Sequence PARTIAL | No dedicated sequence ownership conflict model or tests |
| Lowering | READY | PARTIAL | `PathComponent::TupleIndex`; `OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX`; tuple ownership event serialization; sequence lowering opcodes exist separately | Tuple READY; Sequence PARTIAL | No sequence ownership emission component; sequence lowering is value/runtime lowering only |
| SemCode format vocabulary | READY | NOT SUPPORTED | `OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX`; `DecodedAccessPathComponent::TupleIndex`; sequence iteration has separate `CAP_SEQUENCE_ITERATION` and `SEQUENCE_LEN` | Tuple READY; Sequence NOT SUPPORTED | No `OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX` or equivalent |
| SemCode decode | READY | NOT SUPPORTED | `DecodedAccessPathComponent::TupleIndex` and `FieldSymbol`; no sequence ownership decode kind exists | Tuple READY; Sequence NOT SUPPORTED | Sequence decode exists for value/iteration transport, not ownership paths |
| Verifier | MOSTLY READY | NOT SUPPORTED | Ownership section presence checks and component-kind rejection in `crates/sm-verify/src/lib.rs`; docs/spec freeze tuple ownership transport | Tuple MOSTLY READY; Sequence NOT SUPPORTED | No sequence ownership payload kind to admit or reject |
| VM value execution | READY | READY | `BorrowWriteConflict`; `PathComponent::TupleIndex`; `Value::Sequence`; `SequenceGet`, `SequencePush`, `SequencePop`, `SequenceLen` | Tuple READY; Sequence READY for value execution | Sequence runtime execution does not imply ownership-path support |
| VM overlap semantics | READY | NOT SUPPORTED | Tuple/record overlap traps in `tests/runtime_ownership_e2e.rs` and `crates/sm-vm/src/semcode_vm.rs` | Tuple READY; Sequence NOT SUPPORTED | No sequence ownership overlap semantics because no sequence ownership paths exist |
| Positive E2E tests | READY | NOT SUPPORTED | Tuple ownership E2E coverage in `tests/runtime_ownership_e2e.rs`; sequence positive tests live in `tests/pcc7_sequence_acceptance.rs` as runtime tests, not ownership tests | Tuple READY; Sequence NOT SUPPORTED | No positive sequence ownership golden |
| Negative hardening | READY | NOT SUPPORTED | Tuple/record negative ownership cases in `tests/runtime_ownership_e2e.rs`; verifier malformed ownership tests in `crates/sm-verify/src/lib.rs` | Tuple READY; Sequence NOT SUPPORTED | No sequence ownership negative corpus |
| 7hell coverage | READY | UNKNOWN | `tools/7hell/run.ps1`, `tools/7hell/run.sh`, `tools/7hell/README.md`, and `docs/roadmap/pcc/7hell_mini_runner.md` now include tuple ownership smoke in Hell 6 | Tuple READY; Sequence UNKNOWN | Sequence-specific 7hell wiring not found |
| Docs / closeout | READY | PARTIAL | `docs/roadmap/pcc/tuple_ownership_matrix.md`, this audit, and existing ownership docs | Tuple READY; Sequence PARTIAL | Sequence ownership closeout still missing |

## 3. Tuple Ownership Readiness

Tuple ownership is the stronger half of this audit and is now complete for
direct tuple element ownership paths.

What is already shown:

- tuple path construction exists in source-facing path types;
- tuple path capture / availability logic exists in typecheck;
- lowering emits `TupleIndex` ownership components;
- SemCode carries tuple ownership components;
- decode and VM remap tuple ownership components back into runtime paths;
- runtime overlap semantics reject conflicting writes;
- tuple E2E ownership tests exist;
- tuple smoke is now part of 7hell Hell 6.

Tuple ownership therefore qualifies as a completed direct tuple element slice.
The remaining question is not whether tuple ownership exists, but whether the
project wants any broader tuple-family behavior beyond the direct path cases
already proven here.

## 4. Sequence Ownership Readiness

Sequence support should be treated as a separate maturity lane.

What is already shown:

- `Sequence(type)` literals exist;
- sequence indexing exists;
- sequence iteration exists;
- `Value::Sequence` exists in the VM;
- sequence persistent utilities such as `push`, `prepend`, `pop`, `len`, and
  `contains` are implemented.

What is **not** shown:

- no dedicated sequence ownership-path component;
- no sequence ownership SemCode vocabulary;
- no sequence ownership decode kind;
- no sequence ownership verifier corpus;
- no sequence ownership VM overlap semantics;
- no sequence ownership E2E golden.

The important detail is that `Expr::SequenceIndex` currently maps to the same
generic access-path shape used by tuple-like access-path tracking. That is a
useful internal mechanism, but it is not evidence of a sequence ownership
contract.

Sequence readiness for ownership is therefore **PARTIAL / NOT PROVEN**.

## 5. What Already Works

- Tuple ownership transport is end-to-end for direct tuple paths.
- Direct record-field ownership is already a separate, completed slice and
  provides the best comparison point for tuple support.
- Sequence values and sequence iteration/indexing are already qualified as
  runtime/source features.
- The verifier already enforces structural ownership admission and rejects
  malformed ownership payloads.
- The VM already enforces borrow/write overlap conflicts for admitted tuple and
  direct record-field paths.

## 6. Gaps

Tuple gaps:

- no tuple-specific 7hell wiring was found until T-5 added the tuple smoke;
- no tuple write/update ownership emission was separately qualified.

Sequence gaps:

- no ownership-path vocabulary for sequence indexes;
- no sequence ownership transport in SemCode;
- no sequence ownership verifier / VM / E2E corpus;
- no sequence ownership maturity claim is justified.

## 7. Comparison With Completed Ownership Slices

Compared with the completed ADT payload, Option/Result parity, and direct
record-field slices:

- Tuple ownership is structurally similar: it has path vocabulary, lowering,
  SemCode transport, decode, verifier admission, VM enforcement, tests, and
  7hell smoke.
- Sequence support is not structurally similar yet. It has runtime value
  support and indexing/iteration, but not ownership-path transport.
- ADT payload and record-field slices were qualified by end-to-end ownership
  evidence. Tuple is close to that shape. Sequence is not.

By the audit criteria:

- vocabulary: tuple yes, sequence no;
- typecheck conflict planning: tuple yes, sequence no dedicated slice;
- lowering emission: tuple yes, sequence no;
- SemCode decode: tuple yes, sequence no;
- verifier positive/negative path: tuple yes, sequence no;
- VM overlap semantics: tuple yes, sequence no;
- positive E2E: tuple yes, sequence no;
- negative hardening: tuple yes, sequence no;
- 7hell smoke: not proven here for either tuple or sequence;
- closeout docs: tuple/spec coverage exists; sequence ownership closeout does
  not.

## 8. Proposed T-1..T-5 Plan

Tuple-focused follow-up:

- T-1 - tuple PatternPath / typecheck conflict tests
- T-2 - tuple lowering ownership emission tests
- T-3 - tuple source -> SemCode -> VM positive E2E golden
- T-4 - tuple verifier / VM negative hardening
- T-5 - tuple ownership smoke to the local qualification gate

Sequence follow-up, if the project decides to pursue it separately:

- S-0 - sequence ownership vocabulary audit
- S-1 - define whether sequence indexing should remain runtime-only or gain a
  dedicated ownership-path family

Do not merge the sequence question into the tuple slice unless the project
explicitly decides that sequence ownership should reuse the tuple slice
semantics.

## 9. Explicit Non-Goals

- No Rust code changes.
- No new SemCode format.
- No new ownership path components.
- No VM behavior changes.
- No verifier behavior changes.
- No parser/typechecker changes.
- No sequence ownership implementation.
- No nested ownership generalization.
- No 7hell changes in T-0.
- No release qualification claims.

## 10. Validation Evidence

Commands used for this audit:

- `Get-Location`
- `git status --short --untracked-files=all`
- `git log --oneline -10`
- `Test-Path .\tools\7hell\run.ps1`
- `Test-Path .\docs\roadmap\pcc\cli_public_sample_qualification_matrix.md`
- `Test-Path .\docs\roadmap\pcc\tuple_ownership_matrix.md`
- `Get-ChildItem docs/dna -Recurse -File | Sort-Object FullName`
- `Get-Content docs/dna/SEMANTIC_UI_DNA.md`
- `rg -n "TupleIndex|tuple_index|OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX|PathComponent::TupleIndex|AccessPath::tuple_index|PatternPathElem::TupleIndex" crates tests docs examples`
- `rg -n "Sequence|Value::Sequence|sequence iteration|iterable|for_each|SequenceIndex|sequence_index|OWNERSHIP_PATH_COMPONENT_SEQUENCE|PatternPathElem::Index|AccessPath::index" crates tests docs examples`
- `rg -n "TupleIndex|FieldSymbol|AdtPayload" crates/sm-format crates/sm-runtime-core crates/sm-verify crates/sm-vm crates/sm-ir tests`
- `rg -n "OWNERSHIP_PATH_COMPONENT_SEQUENCE|SequenceIndex|sequence ownership|ownership.*Sequence" crates tests docs`
- `Select-String -Path 'docs/status/feature_maturity_matrix.md' -Pattern 'Sequence indexing and iteration|Sequence|Tuple|ownership'`
- `Select-String -Path 'docs/spec/runtime_ownership.md','docs/spec/semcode.md' -Pattern 'TupleIndex|Sequence|ownership'`
- `Select-String -Path 'crates/sm-runtime-core/src/lib.rs' -Pattern 'pub enum PathComponent|TupleIndex|Field\\(|AdtPayload|tuple_index\\('`
- `Select-String -Path 'crates/sm-vm/src/semcode_vm.rs' -Pattern 'DecodedAccessPathComponent::TupleIndex|DecodedAccessPathComponent::FieldSymbol|DecodedAccessPathComponent::AdtPayload|Value::Sequence|SequenceGet|SequencePush|SequencePop|BorrowWriteConflict'`
- `Select-String -Path 'tests/runtime_ownership_e2e.rs' -Pattern 'TupleIndex|FieldSymbol|AdtPayload|BorrowWriteConflict|sequence'`
- `Select-String -Path 'crates/sm-verify/src/lib.rs' -Pattern 'ownership-path|OWN0|TupleIndex|FieldSymbol|component'`
- `Get-ChildItem tests/fixtures/pcc7_sequence -File`
- `cargo test --test tuple_ownership_golden`
- `cargo fmt --check`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\\tools\\7hell\\run.ps1`

Notes:

- The current checkout has `tools/7hell/run.ps1`, and the gate passed.
- The audit remains doc-only.
