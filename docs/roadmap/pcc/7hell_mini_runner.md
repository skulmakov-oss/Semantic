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
- Ensures the entire workspace compiles under the runner's configured feature
  contour (`cargo check --workspace --all-features`)
- Does **not** execute the workspace test suite. Hell 1 is a compile-health
  gate only; later gates run targeted per-crate test lanes (see Hell 3-6
  below), not the general `cargo test --workspace` suite.

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
- Runs the PCC control-flow negative diagnostics harness (`tests/pcc_control_flow_negative.rs`) to cover stable fail-fast markers for loop, match, and return-path rejection cases.
- Runs the PCC text negative diagnostics harness (`tests/pcc_text_negative.rs`) to cover stable fail-fast markers for text concatenation, `to_text(record)`, multiline text, and text-ordering rejection cases.
- Runs the PCC collections negative diagnostics harness (`tests/pcc_collections_negative.rs`) to cover stable fail-fast markers for unsupported map removal, map iteration, collection formatting, and type-mismatch rejection cases.
- Runs the PCC stdlib negative diagnostics harness (`tests/pcc_stdlib_negative.rs`) to cover stable fail-fast markers for non-text `print(...)`, collection printing, `to_text(record)`, `to_text(collection)`, and premature stdlib namespace usage.
- See [PCC-STDLIB-5 closeout](stdlib_v0_closeout.md) for the final Stdlib v0 contour summary.

### Hell 7 — PCC Documentation Integrity
Proves that architectural documentation matches code reality.
- Checks that `adt_payload_ownership_matrix.md`, `adt_payload_ownership_slice_closeout.md`, and `adt_payload_ownership_paths.md` are present and up to date.

## Qualification Boundary

A 7hell PASS proves only the commands and targeted qualification lanes
actually executed by the gates above (workspace compile-health, targeted
per-crate tests for `sm-format`/`sm-verify`/`sm-vm`, and the other named
checks). It does **not** imply that the entire workspace test suite passed.

Workspace-wide tests (`cargo test --workspace ...`) are a separate
qualification lane. In hosted CI they are currently executed by the
`test-std` job (`cargo test --workspace --all-targets --quiet` and
`cargo test --workspace --doc --quiet`), not by any 7hell gate. A 7hell PASS
alone must not be cited as proof that the complete workspace test suite
passed.

## Usage
Run the scripts in `tools/7hell/`:

- `run.ps1` (Windows) and `run.sh` (Linux/macOS) are the full local 7hell
  runners and execute all 7 gates above.
- `run_ci.ps1` is the fast CI qualification contour and currently executes
  only Hell 1 through Hell 5 - it does not include Hell 6 (Source to SemCode
  Smoke) or Hell 7 (PCC Documentation Integrity).

The scripts will fail-fast at the first broken gate.
