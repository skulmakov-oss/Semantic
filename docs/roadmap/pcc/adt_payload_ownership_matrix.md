# ADT Payload Ownership Support Matrix

## 1. Executive Status

**PCC-ADT Payload Ownership Slice:**
COMPLETE / PASS

**Core Trust Freeze:**
Advanced, but not fully closed.

**Broader ADT Language Support:**
Not fully complete.

---

## 2. Layer Matrix

| Layer | Owner crate | Capability | Status | Evidence / tests | Remaining gaps |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Parser** | `sm-front` | Parse ADT/enum patterns & bindings | COMPLETE | `ref_binding_in_adt_pattern_parses` | None for this slice |
| **Typecheck** | `sm-front` | Infer and canonicalize ADT scrutinee & payload | COMPLETE | `wildcard_match_pattern_typechecks`, `adt_match_local.sm` | Exhaustiveness checks |
| **Pattern conflict planning** | `sm-front` | Detect overlapping moves/borrows on payloads | COMPLETE | `adt_payload_prefix_overlap_...` | Surface mutation conflict |
| **Lowering** | `sm-ir` | Emit `AdtPayload` paths & borrow events | COMPLETE | `legacy_lowering.rs` static extraction | - |
| **SemCode format** | `sm-format` | Express `AdtPayload { variant, index }` | COMPLETE | `AccessPathComponent::AdtPayload` | - |
| **SemCode decode** | `sm-runtime-core`| Decode binary to `AdtPayload` path component | COMPLETE | `DecodedAccessPathComponent::AdtPayload` | - |
| **Verifier positive path** | `sm-verify` | Accept well-formed payload paths | COMPLETE | `test_valid_adt_payload_path_decode` | - |
| **Verifier malformed rejection** | `sm-verify` | Reject invalid variant/index or missing values | COMPLETE | `test_malformed_adt_payload_...` | - |
| **VM value execution** | `sm-vm` | Execute ownership events against ADT memory | COMPLETE | `sm-vm` payload mapper logic | - |
| **VM borrow overlap** | `sm-vm` | Semantics for overlapping paths with payload | COMPLETE | `test_borrow_overlap_adt_payload_...` | - |
| **Positive E2E** | `semantic_language`| Golden execution: source → SemCode → VM | COMPLETE | `test_e2e_adt_payload_ownership_path` | - |
| **Source-level negative ownership tests** | `sm-front` | Test prefix overlaps & different variant allows | COMPLETE | `typecheck::tests::adt_payload_...` | - |
| **CLI/compiler smoke** | `smc-cli` | Compile local adt scripts successfully | COMPLETE | `smc compile adt_match_local.sm` | - |
| **Docs/closeout** | workspace | Track what is done vs what is deferred | COMPLETE | `adt_payload_ownership_slice_closeout.md` | - |

---

## 3. Supported Cases

The current slice supports:
- enum declaration with payload variants
- ADT constructor
- ADT match
- `ref` payload binding
- `AdtPayload` ownership path emission
- variant-qualified payload path: `AdtPayload { variant, index }`
- borrow overlap rejection for same payload
- no false conflict for different variants
- no false conflict for different payload indexes
- local let-bound ADT match lowering

---

## 4. Explicitly Not Covered

This slice does not claim:
- full generic ADT support
- broad ADT ergonomics
- full exhaustiveness redesign
- source-level mutation conflict over borrowed ADT payload if language surface cannot express it cleanly yet
- `Option`/`Result` polish beyond what current tests cover
- broader collection/text/stdlib support

---

## 5. Evidence List

Exact commands used for final verification of this slice:
- `cargo fmt --check`
- `cargo check --workspace --all-features`
- `cargo test --workspace --all-features`
- `cargo run --bin smc -- compile crates/sm-front/tests/adt_match_local.sm -o out.smc`

(Note: `out.smc` was removed after successful validation to keep the workspace clean).

---

## 6. Commit References

The work is spread across the following key commits:
- `feat(adt): add ADT payload ownership path vocabulary`
- `feat(adt): emit ADT payload ownership paths from lowering`
- `test(vm): add tests for ADT payload overlap`
- `test(adt): add end-to-end golden tests for ADT payload ownership paths`
- `test(adt): reject malformed ADT payload ownership paths`
- `feat(adt): fix local ADT match lowering and source ownership conflicts`

---

## 7. Final Verdict

**PCC-ADT Payload Ownership Slice is complete.**
Next recommended work should be selected from the PCC matrix, not by expanding ADT scope opportunistically.
