# Record Field Ownership Support Matrix

## 1. Executive Status

**Record Field Ownership Slice:**
COMPLETE / PASS for direct named fields

**Nested record-field chains:**
NOT PROVEN

**Broader record language support:**
Not fully complete

---

## 2. Layer Matrix

| Layer | Owner crate/file | Capability | Status | Evidence / tests | Remaining gaps |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Parser | `sm-front/src/parser.rs` | Parse record declarations, field access, record update, and record patterns | COMPLETE | `rustlike_parser_accepts_record_literal_surface`, `rustlike_parser_accepts_record_field_access_surface`, `rustlike_parser_accepts_record_copy_with_surface`, `rustlike_parser_accepts_record_destructuring_bind` | Syntax-level only; no ownership claim by itself |
| Typecheck / PatternPath conflict planning | `sm-front/src/typecheck.rs` | Track direct record-field paths and reject overlapping borrow/write conflicts | COMPLETE | `record_field_same_path_move_and_borrow_rejects`, `record_field_different_fields_allow`, `record_field_parent_child_move_and_borrow_rejects`, `record_field_child_parent_move_and_borrow_rejects`, `record_field_nested_prefix_overlap_rejects` | Nested record-field chains are not proven as a lowering target |
| Lowering ownership emission | `sm-ir/src/legacy_lowering.rs` | Emit borrow/write ownership events for direct record fields | COMPLETE | `lower_record_borrow_capture_records_borrow_path_event`, `lower_record_copy_with_emits_field_write_events`, `lower_record_borrow_capture_records_distinct_field_paths`, `lower_record_copy_with_emits_distinct_field_write_events` | Direct named fields only; no nested chain proof |
| SemCode format vocabulary | `sm-format/src/local_format.rs` | Carry `OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL` and ownership capability bits | COMPLETE | `OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL`, `CAP_OWNERSHIP_FIELD_PATHS` | No format redesign is justified |
| SemCode decode | `sm-format/src/semcode_decode.rs` | Decode `DecodedAccessPathComponent::FieldSymbol` from OWN0 | COMPLETE | `decode_semcode_envelope`, `DecodedAccessPathComponent::FieldSymbol` | Decode supports existing field-symbol component only |
| Verifier valid/malformed coverage | `sm-verify/src/lib.rs` | Accept valid field-symbol ownership and reject malformed OWN0 record-field payloads | COMPLETE | `verifier_accepts_record_field_borrow_ownership_semcode`, `verifier_accepts_record_field_write_ownership_semcode`, `verifier_rejects_invalid_record_field_component_kind`, `verifier_rejects_truncated_record_field_payload`, `verifier_rejects_record_field_payload_under_v11_capabilities` | Feature-wired verifier validation is environment-sensitive |
| VM value execution | `sm-vm/src/semcode_vm.rs`, `sm-runtime-core/src/lib.rs` | Execute record values and preserve field path metadata | COMPLETE | `vm_rejects_record_field_write_after_borrow_same_field`, `vm_allows_record_field_write_to_sibling_field_with_active_borrow`, `vm_rejects_record_parent_write_when_borrowed_child_field` | No nested-chain execution proof |
| VM overlap semantics | `sm-vm/src/semcode_vm.rs` | Enforce deterministic overlap behavior for direct record fields | COMPLETE | Same-field, sibling-field, parent/child record-field overlap tests in `tests/runtime_ownership_e2e.rs` | Nested record-field chains remain unproven |
| Positive E2E golden | `tests/record_field_ownership_golden.rs` | Source -> SemCode -> verify -> VM positive path | COMPLETE | `positive_record_field_ownership_e2e_golden` | Direct named fields only |
| Negative hardening | `tests/runtime_ownership_e2e.rs`, `crates/sm-verify/src/lib.rs`, `crates/sm-vm/src/semcode_vm.rs` | Reject malformed / conflicting record-field ownership paths | COMPLETE BY EXISTING COVERAGE | Existing VM overlap and verifier malformed-path tests | No new negative implementation required |
| 7hell smoke | `tools/7hell/run.ps1`, `tools/7hell/run.sh` | Gate the record-field positive smoke in Hell 6 | COMPLETE | Hell 6 now compiles `tests/fixtures/pcc4_records/positive_record_field_ownership.sm` | Bash validation in this Windows environment did not have `cargo` on PATH |
| Docs | `docs/roadmap/pcc` | Record the slice and its proof boundaries | COMPLETE | `record_field_ownership_audit.md`, this matrix | None for the current slice |

---

## 3. Supported Cases

The direct named record-field slice supports:
- direct named record field path
- same field conflict
- different field allow
- parent record vs child field conflict
- child field vs parent record conflict
- record field borrow ownership emission
- record copy/update write ownership emission
- SemCode `FieldSymbol` ownership component
- positive source -> SemCode -> verify -> VM path
- 7hell record-field smoke

---

## 4. Explicitly Not Covered

This slice does not claim:
- nested record-field chain lowering, for example `outer.inner.x`
- record layout redesign
- generic records
- structural typing expansion
- broad record ergonomics
- new SemCode format
- new VM representation
- broader release qualification

---

## 5. Evidence List

Exact commands used for the slice:
- `cargo fmt --check`
- `cargo test -p sm-front --all-features`
- `cargo test -p sm-ir --all-features`
- `cargo test --test record_field_ownership_golden`
- `cargo test --test runtime_ownership_e2e`
- `cargo test -p sm-verify --all-features --features sm-ir/profile-rust`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1`

Additional validation note:
- `bash tools/7hell/run.sh` failed in the local Windows bash environment because `cargo` was not in `PATH`; this was not treated as a repository regression.

---

## 6. Commit References

Key commits for the slice:
- `6d5c330` `docs(pcc): audit record field ownership support`
- `1fd0d8d` `test(record): add record field PatternPath conflict tests`
- `6b08ddc` `test(record): add record field lowering ownership tests`
- `e50abbe` `test(record): add record field ownership golden`
- `fa942e2` `test(7hell): add record field ownership smoke`

R-4 was closed by existing coverage, so no new commit was required.

---

## 7. Final Verdict

Record Field Ownership Slice is complete for direct named fields.
Nested record-field chains remain explicitly unproven and must not be claimed.
