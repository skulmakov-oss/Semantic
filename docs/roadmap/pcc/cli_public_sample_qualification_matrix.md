# Public CLI Qualification Matrix

## 1. Executive Status

**Public CLI Smoke Qualification Slice:**
COMPLETE / PASS

**Full CLI Release Qualification:**
NOT COMPLETE

**Unqualified areas:**
- `smc repl`
- `smc watch`
- full disasm workflow
- full diagnostics/help matrix
- release packaging

---

## 2. CLI Command Matrix

| Command | Current status | Evidence | Covered fixtures/examples | Remaining gaps |
| :--- | :--- | :--- | :--- | :--- |
| `smc check` | Covered by canonical smoke and earlier acceptance paths | `tests/canonical_examples.rs`, `tests/pcc1_control_flow_gate.rs`, `tests/pcc2_numeric_core_gate.rs`, `tests/pcc3_text_core_gate.rs`, `tests/pcc4_records_acceptance.rs`, `tests/pcc5_adt_acceptance.rs`, `tests/pcc6_option_acceptance.rs`, `tests/pcc7_sequence_acceptance.rs`, `tests/pcc8_stdlib_acceptance.rs`, `tests/pcc9_project_model_acceptance.rs`, `tests/cli_public_smoke_matrix.rs` | `examples/canonical/cli_batch_core/src/main.sm`, `examples/canonical/data_audit_record_iterable/src/main.sm`, `examples/canonical/rule_state_decision/src/main.sm` | Not claimed as full CLI coverage for every source shape |
| `smc run` | Covered by canonical smoke and earlier acceptance paths | `tests/canonical_examples.rs`, `tests/pcc1_control_flow_gate.rs`, `tests/pcc2_numeric_core_gate.rs`, `tests/pcc3_text_core_gate.rs`, `tests/pcc7_sequence_acceptance.rs`, `tests/pcc9_project_model_acceptance.rs`, `tests/cli_public_smoke_matrix.rs` | Same canonical fixtures as above | Not claimed for `repl` or `watch`-driven flows |
| `smc compile` | Covered by canonical smoke and earlier acceptance paths | `tests/canonical_examples.rs`, `tests/pcc1_control_flow_lowering_stability.rs`, `tests/pcc2_numeric_lowering_stability.rs`, `tests/pcc3_text_lowering_stability.rs`, `tests/pcc9_project_model_acceptance.rs`, `tests/cli_public_smoke_matrix.rs` | Same canonical fixtures as above | Artifact production is covered for the smoke set only |
| `smc verify` | Covered by artifact admission paths and canonical smoke | `tests/canonical_examples.rs`, `tests/pcc2_numeric_*`, `tests/pcc3_text_*`, `tests/pcc6_option_result_diagnostics.rs`, `tests/pcc7_collections_diagnostics.rs`, `tests/pcc8_stdlib_diagnostics.rs`, `tests/pcc9_project_model_acceptance.rs`, `tests/cli_public_smoke_matrix.rs` | Same canonical fixtures as above | Not a claim of complete CLI qualification for all SemCode inputs |
| `smc run-smc` | Covered by artifact execution paths and canonical smoke | `tests/smc_run_smc_cli.rs`, `tests/pcc1_control_flow_gate.rs`, `tests/pcc2_numeric_core_gate.rs`, `tests/pcc3_text_core_gate.rs`, `tests/pcc9_project_model_acceptance.rs`, `tests/cli_public_smoke_matrix.rs` | Same canonical fixtures as above | No claim for release packaging or alternate shell wrappers |
| `smc disasm` | Partially covered | `tests/pcc9_project_model_acceptance.rs` | Artifact-only inputs, not source-level canonical workflows | Source-to-disasm workflow is not proven here |
| `smc dump-ast` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |
| `smc dump-ir` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |
| `smc dump-bytecode` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |
| `smc lint` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |
| `smc fmt` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |
| `smc repl` | Unqualified | Public help surface only | None in this slice | No qualification evidence |
| `smc watch` | Unqualified | Public help surface only | None in this slice | No qualification evidence |
| `smc hash-ast` / `smc hash-ir` / `smc hash-smc` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |
| `smc snapshots` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |
| `smc features` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |
| `smc explain` | Unqualified in this slice | Public help surface only | None in this slice | No dedicated qualification matrix yet |

---

## 3. Canonical Examples Covered

CLI-1 established the public smoke path over these canonical examples:

| Example | Smoke path | Status | Notes |
| :--- | :--- | :--- | :--- |
| `examples/canonical/cli_batch_core/src/main.sm` | `check`, `run`, `compile`, `verify`, `run-smc` | Covered | Small CLI-style computation core used as the baseline smoke path |
| `examples/canonical/data_audit_record_iterable/src/main.sm` | `check`, `run`, `compile`, `verify`, `run-smc` | Covered | Record-heavy canonical example |
| `examples/canonical/rule_state_decision/src/main.sm` | `check`, `run`, `compile`, `verify`, `run-smc` | Covered | Canonical record-oriented decision example |

The smoke matrix keeps artifacts under `target/cli-smoke/` and asserts the generated SemCode root stays there.

---

## 4. 7hell Coverage

CLI-2 adds the public CLI smoke matrix to Hell 6 in `7hell`.

The current Hell 6 path now includes:

- record-field source-to-SemCode smoke
- Option/Result source-to-SemCode smoke
- ADT source-to-SemCode smoke
- public CLI smoke matrix

Relevant gate command:

```text
cargo test --test cli_public_smoke_matrix
```

Validation note:

- PowerShell runner passed.
- Bash runner was attempted in the local Windows bash environment and failed because `cargo` was not in `PATH`.
- That is an environment limitation, not a repository regression.

---

## 5. Explicitly Not Covered

This slice does not claim:

- `smc repl`
- `smc watch`
- full disasm source workflow
- all diagnostic commands
- output stability or golden text snapshots
- release packaging
- cross-platform shell validation beyond the PowerShell proof
- performance guarantees

---

## 6. Evidence List

Commands used for the slice:

- `cargo fmt --check`
- `cargo test --test cli_public_smoke_matrix`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1`

Additional note:

- `bash tools/7hell/run.sh` was attempted during CLI-2 validation and failed because `cargo` was not in `PATH` in that local shell environment.

---

## 7. Commit References

- `31449f3` `docs(pcc): audit public CLI and canonical samples`
- `798518d` `test(cli): add public smoke matrix for canonical fixtures`
- `7cf35a1` `test(7hell): add public CLI smoke matrix gate`

---

## 8. Final Verdict

Public CLI Smoke Qualification Slice is complete.
It qualifies a stable smoke path for selected canonical examples.
It does not claim full CLI release qualification.
