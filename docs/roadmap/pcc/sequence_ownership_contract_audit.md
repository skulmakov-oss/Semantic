# Sequence Ownership Contract Audit

## 1. Executive Verdict

Sequence runtime support exists, but sequence ownership is **NOT PROVEN**.

The repository already supports sequence literals, sequence indexing, iteration,
and sequence value execution. That is runtime and language support. It is not,
by itself, evidence of a dedicated sequence ownership-path contract.

Current verdict:

- Sequence runtime support: **YES**
- Sequence ownership: **NOT PROVEN**

Do not claim sequence ownership readiness unless explicit ownership-path
evidence is added.

## 2. Current Support Matrix

| Subsystem | Runtime / language support | Ownership-path support | Evidence | Status | Gaps |
| --- | --- | --- | --- | --- | --- |
| Parser | Yes: sequence literals, indexing, and `for each`-style iteration syntax exist | No dedicated sequence ownership vocabulary | `crates/sm-front/src/parser.rs` handles `Expr::SequenceLiteral`, `Expr::SequenceIndex`, and `Stmt::ForEach` | Runtime/language READY; ownership NOT PROVEN | Sequence syntax does not establish an ownership-path family |
| Typecheck | Yes: sequence literals and sequence indexing are typechecked | No sequence ownership conflict planning is qualified | `crates/sm-front/src/typecheck.rs` routes `Expr::SequenceLiteral` and `Expr::SequenceIndex` through sequence typing; `expr_access_path_sequence_index_literal` maps literal index access to tuple-index path material | Runtime/typecheck READY; ownership NOT PROVEN | Sequence indexing is treated as access-path material, not a sequence ownership contract |
| Pattern conflict planning | Limited runtime-path extraction exists | No dedicated sequence PatternPath / conflict suite | `expr_access_path_sequence_index_literal` and `infer_sequence_index_type` in `crates/sm-front/src/typecheck.rs` | PARTIAL | No sequence ownership conflict tests or claim |
| Lowering | Yes: sequence expressions lower for runtime/value behavior | No sequence ownership emission component | `crates/sm-ir/src/legacy_lowering.rs` lowers `Expr::SequenceIndex`; ownership emission uses `PathComponent::TupleIndex` only | Runtime READY; ownership NOT PROVEN | No `SequenceIndex` ownership path component and no sequence ownership event emission |
| SemCode format vocabulary | Yes: sequence-related execution capabilities exist | No sequence ownership vocabulary exists | `crates/sm-format/src/local_format.rs` defines `OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX`, `CAP_SEQUENCE_VALUES`, and `CAP_SEQUENCE_ITERATION`; `crates/sm-format/src/semcode_decode.rs` only decodes `TupleIndex`, `FieldSymbol`, and `AdtPayload` | Runtime READY; ownership NOT PROVEN | No `OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX` or equivalent |
| SemCode decode | Yes: sequence programs can decode as ordinary SemCode payloads | No sequence ownership decode kind exists | `crates/sm-format/src/semcode_decode.rs` decodes tuple, field, and ADT ownership components only | Runtime READY; ownership NOT PROVEN | No sequence ownership component to decode or reject specifically |
| Verifier | Yes: structural admission for ownership sections exists | No sequence-specific ownership admission is qualified | `crates/sm-verify/src/lib.rs` rejects malformed ownership sections generically and recognizes tuple/field/ADT components | Structural READY; sequence ownership NOT PROVEN | No sequence ownership payload kind, positive corpus, or malformed corpus |
| VM value execution | Yes: `Value::Sequence` and sequence ops exist | No sequence ownership-overlap semantics are qualified | `crates/sm-vm/src/semcode_vm.rs` supports `Value::Sequence`, `SequenceGet`, `SequencePush`, `SequencePop`, `SequenceLen`, and `SequenceContains` | Runtime READY; ownership NOT PROVEN | Runtime execution does not imply sequence borrow/write overlap behavior |
| VM overlap semantics | Yes: VM enforces overlap semantics for admitted tuple and record-field paths | No sequence overlap semantics are proven | `tests/runtime_ownership_e2e.rs` covers tuple and record-field conflicts; `crates/sm-vm/src/semcode_vm.rs` remaps ownership paths back into runtime access paths | READY for tuple/record; sequence NOT PROVEN | No evidence that `seq[0]` and `seq[1]` are distinct ownership paths |
| Positive E2E tests | Yes: sequence runtime behavior has positive fixture coverage | No ownership-specific sequence golden exists | `tests/fixtures/pcc7_sequence/*` and sequence acceptance tests show runtime support; no sequence ownership golden was found | Runtime READY; ownership NOT PROVEN | No positive source -> SemCode -> decode -> verify -> VM ownership test for sequences |
| Negative hardening | Yes: generic malformed ownership handling exists | No sequence ownership hardening corpus exists | `crates/sm-verify/src/lib.rs` has generic ownership-section rejection tests; `tests/runtime_ownership_e2e.rs` covers tuple and record-field negatives | Structural READY; sequence ownership NOT PROVEN | No sequence-specific negative corpus |
| 7hell coverage | Yes: 7hell gates tuple, record-field, ADT, Option/Result, and CLI smoke slices | No sequence smoke gate is present | `tools/7hell/run.ps1`, `tools/7hell/run.sh`, `tools/7hell/README.md`, and `docs/roadmap/pcc/7hell_mini_runner.md` do not provide a sequence ownership gate | READY for other slices; sequence NOT PROVEN | No sequence ownership 7hell gate |
| Docs | Yes: sequence runtime support is documented in fixtures and roadmap artifacts | No sequence ownership closeout exists | `docs/roadmap/pcc/tuple_sequence_ownership_audit.md` explicitly says sequence ownership is not proven; `tests/fixtures/pcc7_sequence/*` show runtime support | PARTIAL | No sequence ownership matrix/closeout doc and no ownership contract decision |

## 3. What Already Works

- Sequence literals parse and typecheck.
- Sequence indexing parses and typecheck.
- Sequence iteration syntax exists.
- The VM executes sequence values through `Value::Sequence`.
- Sequence runtime helpers such as `push`, `prepend`, `pop`, `len`, and
  `contains` are covered by tests.
- The typechecker can treat literal sequence indexing as access-path material
  in the generic path machinery, but that is still not sequence ownership
  evidence.

This is enough to say the language/runtime supports sequences. It is not enough
to say sequence ownership is qualified.

## 4. Ownership Gaps

The missing pieces are ownership-specific, not runtime-specific:

- ownership path vocabulary for sequence elements;
- `AccessPath` component or equivalent for sequence ownership;
- `PatternPath` element for sequence ownership conflicts;
- lowering emission for sequence ownership events;
- SemCode format vocabulary for sequence ownership;
- verifier support for sequence ownership paths;
- VM overlap semantics for sequence element ownership;
- positive sequence ownership E2E tests;
- negative sequence ownership hardening;
- 7hell coverage for sequence ownership.

The strongest current clue is that sequence indexing already participates in
generic access-path extraction in the frontend. That may be useful for future
contract design, but it is not a proven ownership slice.

## 5. Comparison With Tuple Ownership

Tuple readiness does not imply sequence readiness.

Tuple ownership was qualified through a full chain:

- PatternPath conflict planning;
- lowering ownership emission;
- SemCode `TupleIndex` vocabulary;
- decode and verifier admission;
- VM overlap semantics;
- positive E2E golden;
- negative hardening;
- 7hell smoke;
- closeout matrix.

Sequence currently has only runtime/language support:

- literals;
- indexing;
- iteration;
- value execution.

It does not have the ownership-path transport, overlap semantics, or qualified
gate coverage that tuple now has. Therefore tuple completion must not be used
as a proxy for sequence completion.

## 6. Proposed SEQ-1..SEQ-5 Plan

The sequence slice should start with a contract decision, not tests-first.

- SEQ-1 - sequence ownership contract decision / vocabulary design
- SEQ-2 - sequence PatternPath / typecheck conflict tests
- SEQ-3 - sequence lowering ownership emission tests
- SEQ-4 - sequence E2E plus verifier / VM negative hardening
- SEQ-5 - sequence smoke in 7hell plus closeout

If the project decides sequence ownership should exist, SEQ-1 must define the
ownership vocabulary before lower-layer tests are added. The current codebase
does not justify skipping that contract step.

## 7. Explicit Non-Goals

- No Rust code changes in SEQ-0.
- No new SemCode format.
- No new sequence ownership component.
- No VM behavior changes.
- No verifier behavior changes.
- No parser/typechecker behavior changes.
- No lowering changes.
- No 7hell changes.
- No sequence ownership implementation.
- No release qualification claim.

## 8. Next Contract Step

- Proposed design follow-up:
  [sequence_ownership_contract_design.md](./sequence_ownership_contract_design.md)

SEQ-1 should decide the vocabulary before any implementation or tests are added.

## 9. Validation Evidence

Commands run for this audit:

- `Get-Location`
- `git status --short --untracked-files=all`
- `git branch --show-current`
- `git log --oneline -10`
- `git branch --contains adcdbfb`
- `Test-Path .\tools\7hell\run.ps1`
- `Test-Path .\docs\roadmap\pcc\tuple_ownership_matrix.md`
- `Test-Path .\docs\roadmap\pcc\tuple_sequence_ownership_audit.md`
- `Get-ChildItem docs/dna -Recurse -File | Sort-Object FullName`
- `Get-Content docs/dna/SEMANTIC_UI_DNA.md`
- `rg -n "SequenceCollectionFamily|SequenceIndexExpr|SequenceLiteral|SequenceIndex|ForEach|Expr::SequenceIndex|Expr::SequenceLiteral|infer_sequence_index_type|expr_access_path_sequence_index_literal" crates/sm-front/src/parser.rs crates/sm-front/src/typecheck.rs crates/sm-front/src/types.rs`
- `rg -n "OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX|TupleIndex|FieldSymbol|AdtPayload|SequenceIndex|Sequence.*ownership|PathComponent::TupleIndex|DecodedAccessPathComponent::TupleIndex" crates/sm-format/src/local_format.rs crates/sm-format/src/semcode_decode.rs crates/sm-ir/src/legacy_lowering.rs crates/sm-verify/src/lib.rs crates/sm-vm/src/semcode_vm.rs`
- `Get-Content crates/sm-front/src/parser.rs | Select-Object -Skip 1528 -First 30`
- `Get-Content crates/sm-front/src/typecheck.rs | Select-Object -Skip 1982 -First 40`
- `Get-Content crates/sm-front/src/typecheck.rs | Select-Object -Skip 11472 -First 35`
- `Get-Content crates/sm-format/src/local_format.rs | Select-Object -First 80`
- `Get-Content crates/sm-format/src/semcode_decode.rs | Select-Object -First 360`
- `Get-Content crates/sm-vm/src/semcode_vm.rs | Select-Object -Skip 620 -First 30`
- `Get-Content crates/sm-vm/src/semcode_vm.rs | Select-Object -Skip 4588 -First 25`
- `Get-Content crates/sm-verify/src/lib.rs | Select-Object -Skip 430 -First 50`
- `Get-Content crates/sm-ir/src/legacy_lowering.rs | Select-Object -Skip 350 -First 60`
- `Get-Content tests/runtime_ownership_e2e.rs | Select-Object -First 140`
- `Get-Content docs/roadmap/pcc/tuple_ownership_matrix.md | Select-Object -First 260`
- `Get-Content docs/roadmap/pcc/tuple_sequence_ownership_audit.md | Select-Object -First 260`
- `Get-Content docs/dna/SEMANTIC_UI_DNA.md | Select-Object -First 260`
- `cargo fmt --check`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\\tools\\7hell\\run.ps1`

Notes:

- `bash tools/7hell/run.sh` was not required for this audit; prior tuple work
  showed the Linux shell path can fail in this environment if `cargo` is missing
  from `PATH`. That is an environment limitation, not a repository regression.

## 10. Final Verdict

Sequence runtime support exists, but sequence ownership is not proven.

Sequence ownership remains explicitly unproven and must not be claimed.
