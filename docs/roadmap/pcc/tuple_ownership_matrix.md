# Tuple Ownership Matrix

## 1. Executive Status

Tuple Ownership Slice:
COMPLETE / PASS for direct tuple element ownership paths

Sequence Ownership:
NOT COMPLETE / NOT PROVEN

Tuple write/update ownership emission:
NOT QUALIFIED unless separately proven by existing evidence

Nested tuple broader claim:
LIMITED to explicitly tested PatternPath/source evidence only

## 2. Layer Matrix

| Layer | Owner crate/file | Capability | Status | Evidence / tests | Remaining gaps |
| --- | --- | --- | --- | --- | --- |
| Parser | `crates/sm-front/src/parser.rs` | Tuple element paths are parsed into tuple path vocabulary | READY | `b0b99c6` tuple PatternPath conflict tests; audit evidence in `docs/roadmap/pcc/tuple_sequence_ownership_audit.md` | No broader tuple-family claim beyond direct element paths |
| Typecheck / PatternPath conflict planning | `crates/sm-front/src/typecheck.rs` | Source-level conflict planning for tuple element paths | READY | `b0b99c6 test(tuple): add tuple PatternPath conflict tests` | Sequence ownership conflict planning remains unproven |
| Lowering ownership emission | `crates/sm-ir/src/legacy_lowering.rs` | Emits tuple ownership paths as `TupleIndex` | READY | `1310c7f test(tuple): add tuple lowering ownership tests` | Tuple write/update ownership emission not separately qualified beyond existing evidence |
| SemCode format vocabulary | `crates/sm-format/src/local_format.rs` | `OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX` | READY | `085c81d test(tuple): add tuple ownership E2E golden` | No sequence ownership component kind |
| SemCode decode | `crates/sm-format/src/semcode_decode.rs` | Decode `DecodedAccessPathComponent::TupleIndex` | READY | `085c81d test(tuple): add tuple ownership E2E golden` | No dedicated sequence ownership decode path |
| Verifier valid/malformed coverage | `crates/sm-verify/src/lib.rs` | Structural admission for ownership sections | READY BY EXISTING COVERAGE | `cargo test -p sm-verify --all-features --features sm-ir/profile-rust`; malformed ownership-section tests already exist | Coverage is generic ownership-section handling, not tuple-only malformed corpus |
| VM value execution | `crates/sm-vm/src/semcode_vm.rs` | Verified tuple ownership executes deterministically | READY | `085c81d test(tuple): add tuple ownership E2E golden` | No tuple write/update-specific behavior claim beyond tested direct paths |
| VM overlap semantics | `crates/sm-vm/src/semcode_vm.rs` and `tests/runtime_ownership_e2e.rs` | Borrow/write overlap rejection for tuple elements | READY BY EXISTING COVERAGE | `tests/runtime_ownership_e2e.rs` tuple overlap cases; `cargo test --test runtime_ownership_e2e` | No nested tuple runtime qualification claim |
| Positive E2E golden | `tests/tuple_ownership_golden.rs` | Source -> SemCode -> decode -> verify -> VM | READY | `085c81d test(tuple): add tuple ownership E2E golden` | Direct tuple element paths only |
| Negative hardening | `tests/runtime_ownership_e2e.rs`, `crates/sm-verify/src/lib.rs` | Runtime and verifier negative ownership checks | READY BY EXISTING COVERAGE | `cargo test --test runtime_ownership_e2e`; `cargo test -p sm-verify --all-features --features sm-ir/profile-rust` | Generic malformed ownership handling, not tuple-only payloads |
| 7hell smoke | `tools/7hell/run.ps1`, `tools/7hell/run.sh` | Qualification gate includes tuple ownership smoke | READY | `d73d3a3 test(7hell): add tuple ownership smoke` | Linux shell validation limited by environment PATH in this session |
| Docs | `docs/roadmap/pcc/tuple_sequence_ownership_audit.md`, `docs/roadmap/pcc/tuple_ownership_matrix.md`, `tools/7hell/README.md`, `docs/roadmap/pcc/7hell_mini_runner.md` | Evidence-backed slice documentation | READY | Current doc set and closeout notes | Sequence ownership closeout remains open |

## 3. Supported Cases

- direct tuple element paths
- `TupleIndex(0)`
- `TupleIndex(1)`
- same tuple element conflict rejection at frontend/typecheck
- different tuple elements allowed at frontend/typecheck
- parent tuple vs child tuple element conflict
- child tuple element vs parent tuple conflict
- distinct lowering ownership paths for `pair.0` and `pair.1`
- source -> SemCode -> `sm-format` decode -> `sm-verify` -> `sm-vm` positive golden
- existing runtime/verifier negative hardening coverage
- 7hell Hell 6 tuple smoke

## 4. Explicitly Not Covered

- sequence ownership-path vocabulary
- sequence ownership E2E
- tuple write/update ownership emission
- broad nested tuple runtime qualification unless explicitly proven
- new SemCode format
- new VM representation
- release qualification
- performance guarantees

## 5. Evidence List

Commands used across the slice:

- `cargo fmt --check`
- `cargo test -p sm-front --all-features`
- `cargo test -p sm-ir --all-features`
- `cargo test --test tuple_ownership_golden`
- `cargo test --test runtime_ownership_e2e`
- `cargo test -p sm-verify --all-features --features sm-ir/profile-rust`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1`

Notes:

- `bash tools/7hell/run.sh` failed locally because `cargo` was not in PATH in that bash environment; this is an environment limitation, not a repository regression.

## 6. Commit References

- `fff8134 docs(pcc): audit tuple and sequence ownership support`
- `b0b99c6 test(tuple): add tuple PatternPath conflict tests`
- `1310c7f test(tuple): add tuple lowering ownership tests`
- `085c81d test(tuple): add tuple ownership E2E golden`
- `T-4 closed by existing coverage, no commit required`
- `d73d3a3 test(7hell): add tuple ownership smoke`

## 7. Final Verdict

Tuple Ownership Slice is complete for direct tuple element ownership paths.
Sequence ownership remains explicitly unproven and must not be claimed.
Tuple write/update ownership emission remains outside this slice unless separately qualified.
