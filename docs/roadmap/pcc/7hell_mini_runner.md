# 7hell PCC Qualification Mini-Runner

## Purpose
The `7hell` mini-runner is a deterministic local qualification harness. It acts as a set of strict gates to prove that the critical boundaries, formats, and semantic guarantees (like the PCC-ADT Payload Ownership Slice) remain fully intact during development.

This is **not** a full release qualification tool (which would cover packaging, no_std, broader negative corpuses, deterministic rerun matrices, etc.). Instead, `7hell v0` specifically focuses on:
- Core Trust Boundary Repair v1
- SemCode Format Authority
- PCC-ADT Payload Ownership Slice

## The 7 Gates

### Hell 1 — Workspace Health
Proves the basic structural health of the codebase.
- Checks formatting (`cargo fmt --check`)
- Ensures the entire workspace compiles without warnings or errors (`cargo check`)
- Validates that the entire general test suite passes (`cargo test`)

### Hell 2 — Trust Boundary Guards
Proves that critical lower-level crates do not acquire forbidden dependencies on higher-level abstractions.
- Executes `semantic_language`'s `trust_boundary_guards` test.
- Audits the `cargo tree` to ensure `sm-vm`, `sm-verify`, and `sm-format` have no dependencies on `sm-ir`, `sm-emit`, or `prom-ui`.

### Hell 3 — SemCode Format Authority
Proves that `sm-format` retains absolute authority over the binary contract and is independent of intermediate representation dependencies.
- Runs `sm-format` isolated tests.
- Uses `rg` to ensure no upstream code (like `sm_ir::semcode_decode` or `sm_emit::semcode_format`) leaks into `sm-format`.

### Hell 4 — Verifier Negative Corpus
Proves admission-hardening by the verifier logic.
- Runs `sm-verify` tests to guarantee rejection of malformed or truncated binary formats (like missing ADT ownership payload parameters).

### Hell 5 — VM Ownership Semantics
Proves semantic borrow/ownership overlap enforcement at runtime.
- Runs `sm-vm` tests to guarantee overlap logic correctly handles ADT payloads (same-payload conflicts, different-index allowances, different-variant allowances, parent/child conflicts).

### Hell 6 — Source to SemCode Smoke
Proves the end-to-end frontend compiler pipeline works correctly up to the binary artifact generation.
- Compiles a critical semantic test case (`crates/sm-front/tests/adt_match_local.sm`) into a SemCode artifact (`out.smc`) and ensures the compilation is successful.
- Compiles the tuple ownership E2E golden fixture (`tests/fixtures/pcc_tuple_ownership/positive_tuple_ownership.sm`) into `target/7hell/positive_tuple_ownership.smc`.
- Runs the public CLI smoke matrix (`tests/cli_public_smoke_matrix.rs`) across canonical examples to cover `smc check`, `smc run`, `smc compile`, `smc verify`, and `smc run-smc`.

### Hell 7 — PCC Documentation Integrity
Proves that architectural documentation matches code reality.
- Checks that `adt_payload_ownership_matrix.md`, `adt_payload_ownership_slice_closeout.md`, and `adt_payload_ownership_paths.md` are present and up to date.

## Usage
Run the scripts in `tools/7hell/` (either `run.ps1` for Windows or `run.sh` for Linux/macOS). The scripts will fail-fast at the first broken gate.
