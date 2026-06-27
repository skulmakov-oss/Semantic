# Record Field Ownership Audit

Status: audit-only
Scope: documentation only

## 1. Executive Verdict

Record field ownership support is already present across the core pipeline for the supported slice:
parser, typecheck, pattern-path tracking, lowering, SemCode encoding/decoding, verifier admission, VM overlap enforcement, and end-to-end runtime tests.

The supported slice is narrow and specific. It covers direct named record field projection only, with deterministic overlap behavior for sibling and parent/child paths. I did not find a dedicated record-field malformed ownership corpus or a 7hell smoke entry that specifically gates record-field ownership, so this is not yet a fully closed record-field qualification slice.

## 2. Current Support Matrix

| Subsystem | Current Record Field Support | Evidence | Status | Gaps |
| --- | --- | --- | --- | --- |
| Parser | Record declarations, record field access, record update, and record destructuring syntax are admitted. | [`crates/sm-front/src/parser.rs`](../../crates/sm-front/src/parser.rs#L428), [`crates/sm-front/src/parser.rs`](../../crates/sm-front/src/parser.rs#L1137), [`crates/sm-front/src/parser.rs`](../../crates/sm-front/src/parser.rs#L1525), [`crates/sm-front/src/types.rs`](../../crates/sm-front/src/types.rs#L207) | Present | Parser support is syntax-level only; ownership meaning is carried later. |
| Typecheck | Record tables, field validation, record field access typing, record update typing, and record pattern path state are present. | [`crates/sm-front/src/typecheck.rs`](../../crates/sm-front/src/typecheck.rs#L8885), [`crates/sm-front/src/typecheck.rs`](../../crates/sm-front/src/typecheck.rs#L9757), [`crates/sm-front/src/typecheck.rs`](../../crates/sm-front/src/typecheck.rs#L9867), [`crates/sm-front/src/typecheck.rs`](../../crates/sm-front/src/typecheck.rs#L7125) | Present | No dedicated record-field ownership qualification slice yet; current coverage is mixed with general record typing. |
| Pattern conflict planning | `PatternPathElem::RecordField` exists and `expr_access_path` can recover record-field paths from nested expressions. | [`crates/sm-front/src/types.rs`](../../crates/sm-front/src/types.rs#L242), [`crates/sm-front/src/types.rs`](../../crates/sm-front/src/types.rs#L275), [`crates/sm-front/src/typecheck.rs`](../../crates/sm-front/src/typecheck.rs#L10948), [`crates/sm-front/src/typecheck.rs`](../../crates/sm-front/src/typecheck.rs#L11212) | Present | I did not find a record-field-specific conflict-planning test corpus separate from general record path tests. |
| Lowering | `Expr::RecordField` lowers to `IrInstr::RecordGet`; record update writes emit `PathComponent::Field` ownership events. | [`crates/sm-ir/src/legacy_lowering.rs`](../../crates/sm-ir/src/legacy_lowering.rs#L2622), [`crates/sm-ir/src/legacy_lowering.rs`](../../crates/sm-ir/src/legacy_lowering.rs#L2732), [`crates/sm-ir/src/legacy_lowering.rs`](../../crates/sm-ir/src/legacy_lowering.rs#L1931) | Present | Lowering is present, but there is no separate record-field ownership doc trail for this slice. |
| SemCode format | `OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL` and `CAP_OWNERSHIP_FIELD_PATHS` exist in the local format contract. | [`crates/sm-format/src/local_format.rs`](../../crates/sm-format/src/local_format.rs#L31), [`crates/sm-format/src/local_format.rs`](../../crates/sm-format/src/local_format.rs#L42) | Present | No format redesign is justified by the current evidence. |
| SemCode decode | `DecodedAccessPathComponent::FieldSymbol` is decoded from OWN0 ownership sections. | [`crates/sm-format/src/semcode_decode.rs`](../../crates/sm-format/src/semcode_decode.rs#L28), [`crates/sm-format/src/semcode_decode.rs`](../../crates/sm-format/src/semcode_decode.rs#L305) | Present | Decode support is present only for the existing field-symbol component. |
| Verifier | Field-symbol ownership components are recognized and cause `CAP_OWNERSHIP_FIELD_PATHS` to be required. | [`crates/sm-verify/src/lib.rs`](../../crates/sm-verify/src/lib.rs#L445), [`crates/sm-verify/src/lib.rs`](../../crates/sm-verify/src/lib.rs#L519) | Present | I did not locate a dedicated malformed record-field OWN0 verifier corpus in the searched files. |
| VM value execution | Runtime ownership paths carry direct record fields via `AccessPath::field` and `PathComponent::Field`; record values execute through `RecordCarrier<Value>`. | [`crates/sm-runtime-core/src/lib.rs`](../../crates/sm-runtime-core/src/lib.rs#L21), [`crates/sm-runtime-core/src/lib.rs`](../../crates/sm-runtime-core/src/lib.rs#L73), [`crates/sm-runtime-core/src/lib.rs`](../../crates/sm-runtime-core/src/lib.rs#L93), [`crates/sm-vm/src/semcode_vm.rs`](../../crates/sm-vm/src/semcode_vm.rs#L633), [`crates/sm-vm/src/semcode_vm.rs`](../../crates/sm-vm/src/semcode_vm.rs#L1389) | Present | Execution is present, but the slice is still bounded to direct named fields. |
| VM overlap semantics | Direct record-field overlap behavior is enforced deterministically: sibling fields pass, same-path and parent/child conflicts reject. | [`tests/runtime_ownership_e2e.rs`](../../tests/runtime_ownership_e2e.rs#L155), [`tests/runtime_ownership_e2e.rs`](../../tests/runtime_ownership_e2e.rs#L182), [`tests/runtime_ownership_e2e.rs`](../../tests/runtime_ownership_e2e.rs#L206), [`tests/runtime_ownership_e2e.rs`](../../tests/runtime_ownership_e2e.rs#L230), [`tests/runtime_ownership_e2e.rs`](../../tests/runtime_ownership_e2e.rs#L503) | Present | Negative coverage is runtime-level; I did not find a separate malformed field-path verifier corpus. |
| Positive E2E tests | Record-field ownership is exercised through compile -> verify -> run paths with sibling-pass and inner-frame cleanup cases. | [`tests/runtime_ownership_e2e.rs`](../../tests/runtime_ownership_e2e.rs#L155), [`tests/runtime_ownership_e2e.rs`](../../tests/runtime_ownership_e2e.rs#L503) | Present | Positive E2E coverage exists, but it does not by itself prove verifier hardening against malformed field paths. |
| Negative tests | Negative field behavior exists for typecheck/runtime conflict cases, but not as a dedicated malformed ownership-path rejection corpus. | [`crates/sm-front/src/typecheck.rs`](../../crates/sm-front/src/typecheck.rs#L5937), [`crates/sm-front/src/typecheck.rs`](../../crates/sm-front/src/typecheck.rs#L6117), [`tests/runtime_ownership_e2e.rs`](../../tests/runtime_ownership_e2e.rs#L182) | Partial | Missing dedicated malformed OWN0 record-field rejection coverage. |
| 7hell coverage | 7hell currently gates ADT payload ownership and Option/Result smoke, plus docs integrity, but no record-field smoke entry. | [`tools/7hell/run.ps1`](../../tools/7hell/run.ps1#L95), [`tools/7hell/run.ps1`](../../tools/7hell/run.ps1#L103), [`tools/7hell/run.sh`](../../tools/7hell/run.sh#L91), [`tools/7hell/run.sh`](../../tools/7hell/run.sh#L99) | Partial | Record-field smoke is not part of the current 7hell gate. |
| Docs | Public docs already freeze direct record-field ownership semantics in the runtime ownership and VM docs. | [`docs/spec/runtime_ownership.md`](../../docs/spec/runtime_ownership.md#L13), [`docs/spec/vm.md`](../../docs/spec/vm.md#L151), [`docs/spec/source_semantics.md`](../../docs/spec/source_semantics.md#L673) | Present | The audit doc itself is the missing record-field-specific ledger entry. |

## 3. What Already Works

- Parser and AST support are already in place for records, field access, and record update.
- Typecheck already resolves record field access, validates record declarations, and preserves record field path state through `PatternPathElem::RecordField`.
- Lowering already emits `RecordGet` for field reads and `PathComponent::Field` ownership metadata for record update writes.
- The SemCode format already has the field-symbol ownership path component and the corresponding capability bit.
- The decoder and verifier already understand `FieldSymbol` ownership components.
- VM runtime semantics already enforce overlap rules for direct record fields, including deterministic sibling-pass and same-path or parent/child rejection.
- E2E runtime tests already prove the direct record field slice across compile, verify, and run.

## 4. Gaps

- I did not find a dedicated malformed record-field OWN0 verifier test corpus.
- I did not find a record-field smoke test in 7hell.
- The evidence is spread across general record support and runtime ownership tests rather than a single record-field ownership closeout trail.
- The supported slice is direct named record fields only; no broader record-layout redesign is implied by current evidence.

## 5. Comparison With ADT Payload Ownership Slice

Record field ownership and ADT payload ownership are structurally similar, but the record field slice is less fully qualified.

| Area | ADT Payload Slice | Record Field Slice |
| --- | --- | --- |
| Vocabulary | `AdtPayload { variant, index }` path component exists. | `Field(SymbolId)` path component exists. |
| Lowering emission | Lowering emits ADT payload ownership events. | Lowering emits field-symbol ownership events. |
| VM overlap semantics | Sibling and parent/child overlap behavior is exercised. | Sibling and parent/child overlap behavior is exercised. |
| Positive E2E | Source -> SemCode -> VM golden coverage exists. | Source -> SemCode -> VM coverage exists. |
| Malformed verifier rejection | Dedicated malformed ADT ownership rejection work exists in the completed slice. | I did not find an equivalent dedicated malformed record-field verifier corpus. |
| Source-level negative tests | The ADT slice has slice-specific negative diagnostics. | Record field negatives exist, but they are mostly general record typing or runtime overlap cases. |
| 7hell smoke | ADT payload / Option-Result smoke is already gated. | No record-field smoke gate yet. |

Conclusion: record field ownership has the same architectural shape as the completed ADT payload slice, but it is missing the same level of qualification and dedicated gate coverage.

## 6. Proposed R-1..R-5 Plan

- R-1: add record-field conflict tests at typecheck / PatternPath level.
- R-2: add lowering tests that assert record-field ownership emission for reads and writes.
- R-3: add one positive source -> SemCode -> VM golden for record-field ownership.
- R-4: add negative verifier / VM hardening for malformed field-path ownership and overlap cases.
- R-5: add record-field smoke coverage to 7hell.

This plan is intentionally narrow. It follows the existing ADT payload slice pattern without inventing new record behavior.

## 7. Explicit Non-Goals

- No record layout redesign.
- No generic records.
- No structural typing expansion.
- No SemCode format change unless a later audit proves a real gap.
- No VM representation redesign.
- No new opcodes.
- No Workbench or UI work.

## 8. Validation Evidence

Commands already run for this audit:

- `pwd`
- `git status --short`
- `git log --oneline -10`
- `cargo metadata --format-version 1 --no-deps`
- `powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1`

Observed results:

- repo root confirmed at `C:\Users\said3\Desktop\EXOcode\Semantic`
- workspace had pre-existing dirty/untracked files outside this change
- `cargo metadata` resolved the expected Semantic workspace
- 7hell passed end-to-end on the current tree

Planned validation after this doc write:

- `cargo fmt --check`
- `cargo test --workspace --all-features`
