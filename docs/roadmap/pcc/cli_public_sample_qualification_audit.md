# CLI Public Sample Qualification Audit

## 1. Executive Verdict

The public `smc` surface is already broad and mostly qualified for the current
release contour. The repository has evidence for `check`, `compile`, `verify`,
`run`, `run-smc`, and `disasm` across source fixtures, project-root fixtures,
canonical examples, and emitted SemCode artifacts.

What is *not* proven by this audit:
- that every public CLI command is equally qualified for every source shape;
- that canonical examples are a substitute for release qualification;
- that `smc disasm` is a source-level workflow rather than an artifact-level one.

Current verdict:

Public CLI / canonical sample qualification is **strong, but not complete**.

## 2. Current CLI Command Map

Observed from `cargo run --bin smc -- --help`:

| Command | Surface | Notes |
| --- | --- | --- |
| `smc compile <input.sm|project-root>` | source / project-root to SemCode | primary artifact producer |
| `smc check <input.sm|project-root>` | source / project-root validation | verifier-first admission gate |
| `smc lint <input.sm>` | source diagnostics | local source lint surface |
| `smc watch <input.sm>` | source watch loop | incremental developer workflow |
| `smc fmt <path>` | formatter | public formatting surface |
| `smc dump-ast <input.sm>` | source inspection | debug surface |
| `smc dump-ir <input.sm>` | source inspection | debug surface |
| `smc dump-bytecode <input.sm>` | source inspection | debug surface |
| `smc hash-ast <input.sm|project-root>` | structural hashing | inspection surface |
| `smc hash-ir <input.sm|project-root>` | structural hashing | inspection surface |
| `smc hash-smc <input.sm|project-root>` | artifact hashing | inspection surface |
| `smc snapshots` | snapshot management | tooling surface |
| `smc features` | feature listing | tooling surface |
| `smc explain <error-code|--list>` | diagnostics help | public explanation surface |
| `smc repl` | interactive surface | not covered by this audit |
| `smc verify <input.smc>` | SemCode admission | artifact verification surface |
| `smc run <input.sm|project-root>` | source / project-root execution | source-to-runtime public path |
| `smc run-smc <input.smc>` | SemCode execution | artifact-to-runtime public path |
| `smc disasm <input.smc>` | SemCode disassembly | artifact inspection surface |

## 3. Canonical Sample Inventory

Current canonical examples pack:

- [examples/canonical/README.md](C:\Users\said3\Desktop\EXOcode\Semantic\examples\canonical\README.md)
- [docs/examples_index.md](C:\Users\said3\Desktop\EXOcode\Semantic\docs\examples_index.md)

Inventory:

| Example | Purpose | Current reading | Notes |
| --- | --- | --- | --- |
| `cli_batch_core` | small CLI-style computation core | qualified limited release | public `run` / `compile` example |
| `rule_state_decision` | record-oriented rule/state decision logic | qualified limited release | current canonical record example |
| `data_audit_record_iterable` | direct-record `Iterable` traversal | qualified limited release | current canonical record-heavy example |
| `wave2_local_helper_import` | helper-module executable authoring | qualified limited release | public source workflow example |
| `positive_selected_import` | selected import executable authoring | qualified limited release | public source workflow example |
| `boundary_alias_import` | boundary example for executable alias import | out of scope | intentionally rejected boundary example |

The canonical examples pack is documented as a curated readiness-facing pack,
not as a blanket guarantee for all language or CLI behaviors.

## 4. Existing Qualification Coverage

| Surface | Current coverage | Evidence | Status |
| --- | --- | --- | --- |
| `smc check` | source and project-root checks | `tests/canonical_examples.rs`, `tests/pcc1_control_flow_gate.rs`, `tests/pcc2_numeric_core_gate.rs`, `tests/pcc3_text_core_gate.rs`, `tests/pcc4_records_acceptance.rs`, `tests/pcc5_adt_acceptance.rs`, `tests/pcc6_option_acceptance.rs`, `tests/pcc7_sequence_acceptance.rs`, `tests/pcc8_stdlib_acceptance.rs`, `tests/pcc9_project_model_acceptance.rs` | covered |
| `smc compile` | source and project-root compilation | `tests/canonical_examples.rs`, `tests/pcc1_control_flow_lowering_stability.rs`, `tests/pcc2_numeric_lowering_stability.rs`, `tests/pcc3_text_lowering_stability.rs`, `tests/pcc9_project_model_acceptance.rs` | covered |
| `smc verify` | artifact admission | `tests/canonical_examples.rs`, `tests/pcc2_numeric_*`, `tests/pcc3_text_*`, `tests/pcc6_option_result_diagnostics.rs`, `tests/pcc7_collections_diagnostics.rs`, `tests/pcc8_stdlib_diagnostics.rs`, `tests/pcc9_project_model_acceptance.rs` | covered |
| `smc run` | source / project-root execution | `tests/canonical_examples.rs`, `tests/pcc1_control_flow_gate.rs`, `tests/pcc2_numeric_core_gate.rs`, `tests/pcc3_text_core_gate.rs`, `tests/pcc7_sequence_acceptance.rs`, `tests/pcc9_project_model_acceptance.rs` | covered |
| `smc run-smc` | artifact execution | `tests/smc_run_smc_cli.rs`, `tests/pcc1_control_flow_gate.rs`, `tests/pcc2_numeric_core_gate.rs`, `tests/pcc3_text_core_gate.rs`, `tests/pcc9_project_model_acceptance.rs` | covered |
| `smc disasm` | artifact disassembly | `tests/pcc9_project_model_acceptance.rs` | covered for artifact-only inputs, not source inputs |
| `svm disasm` | VM disassembly | `tests/cli_quickstart_svm_smoke.rs`, `tests/g1_execution_integrity.rs`, `tests/bytecode_compat.rs` | covered |
| Canonical example pack | readiness-facing curated examples | `examples/canonical/README.md`, `docs/examples_index.md`, `tests/canonical_examples.rs` | covered |

## 5. Gaps

The current public CLI surface is qualified enough to support common source and
artifact workflows, but the following remain explicitly unproven or only
partially qualified:

1. `smc repl` has no qualification evidence in this audit.
2. `smc watch` has no qualification evidence in this audit.
3. `smc lint`, `smc fmt`, `smc dump-*`, and `smc hash-*` exist as public
   commands, but this audit did not establish a dedicated qualification matrix
   for them.
4. `smc disasm` is only shown as artifact-only coverage; source-level or
   canonical-example disassembly workflows are not claimed.
5. Canonical examples are curated readiness evidence, not full release
   qualification.

## 6. Proposed CLI-1..CLI-5 Plan

CLI-1 — source command inventory and smoke matrix
- add a focused command coverage matrix for `check`, `compile`, `run`, and
  `verify` against canonical examples and selected fixtures.

CLI-2 — artifact command qualification
- add dedicated coverage for `run-smc` and `smc disasm` on emitted SemCode
  artifacts.

CLI-3 — project-root workflow audit
- qualify `smc check <project-root>`, `smc run <project-root>`, and
  `smc compile <project-root>` against canonical project-root examples.

CLI-4 — diagnostics and helper command audit
- audit `smc lint`, `smc fmt`, `smc dump-*`, `smc hash-*`, `smc explain`, and
  `smc features` as tooling surfaces, not language-release claims.

CLI-5 — canonical sample closeout
- document which canonical examples are readiness-facing, which are boundary
  examples, and which command paths remain intentionally out of scope.

## 7. Explicit Non-Goals

- No CLI redesign.
- No new public CLI verbs.
- No SemCode format change.
- No VM change.
- No verifier admission change.
- No snapshot updates.
- No claim that canonical examples equal full language readiness.
- No claim that `smc repl` or `smc watch` are qualified unless separate
  evidence is added.

## 8. Validation Evidence

Commands observed or run during this audit:

- `cargo run --bin smc -- --help`
- `rg -n "smc (check|compile|verify|run-smc)|run-smc|disasm|canonical|fixture" docs tests crates examples`
- `rg --files examples tests/fixtures crates/sm-front/tests docs | sort`
- `rg -n "smc check|smc compile|smc verify|smc run-smc|smc run|svm disasm|canonical_examples|run-smc" tests crates/smc-cli examples docs`
- `cargo fmt --check`
- `cargo test --workspace --all-features`
- `powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1`

Notes:

- `cargo run --bin smc -- --help` showed the public CLI command map above.
- The workspace and `7hell` gates remained green.
- `bash tools/7hell/run.sh` was not part of this audit turn.