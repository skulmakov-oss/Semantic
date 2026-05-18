# Semantic Practical Core Feature Matrix Live Audit

Status: active audit scaffold
Track: PCC-0.5 Feature Matrix Live Audit
Layer: language maturity / readiness discipline
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `practical_core_truth_reset.md`
- `practical_core_completion_v0_3.md`
- `core_trust_freeze/index.md`

## 1. Purpose

This document defines the live audit scaffold for Semantic practical core
readiness.

The audit exists to replace optimistic, historical, or mixed-status assumptions
with current evidence from `main`.

Core question:

```text
What is actually true in the repository right now?
```

The answer must be recorded feature by feature before PCC-1 begins.

## 2. Position in PCC

Current ladder:

```text
PCC-0 Truth Reset
  ↓
PCC-0.5 Feature Matrix Live Audit
  ↓
PCC-0.6 7hell Skeleton Seed
  ↓
PCC language phases
  ↕
CTF Core Trust Freeze Lane
```

PCC-0.5 is a gate. It is not implementation work.

The audit must not add new language features, runtime behavior, UI behavior, or
Workbench code.

## 3. Audit goals

The audit must identify:

```text
what really works
what partially works
what is only documented
what is experimental
what is out of scope
```

For every audited item, the output must include:

- feature name;
- current status;
- evidence source;
- tested path;
- missing edge if partial;
- owning PCC phase if work remains;
- CTF impact;
- 7hell coverage need;
- next action.

## 4. Status vocabulary

Only these statuses are allowed:

| Status | Meaning |
|---|---|
| `confirmed-working` | Current `main` has code and evidence for the stated behavior. |
| `confirmed-partial` | Behavior exists, but a missing edge or unsupported shape is explicit. |
| `documented-only` | A document/spec/roadmap exists, but implementation evidence is absent. |
| `experimental` | Exists as research, donor, legacy, or non-canonical substrate. |
| `out-of-scope` | Explicitly excluded from PCC or this audit phase. |
| `unknown` | Not yet audited. Temporary status only. |

Temporary statuses that must be resolved:

```text
implemented / maybe partial
closed but needs audit
ready but not checked
landed but unverified
probably works
assumed stable
```

Rule:

```text
No `unknown` item may be used as justification for starting PCC-1.
```

## 5. Evidence levels

Evidence must be categorized.

| Evidence level | Description |
|---|---|
| `E0-doc` | Documentation or roadmap only. |
| `E1-code` | Code exists, but no direct test or full pipeline evidence is cited. |
| `E2-test` | Unit/integration/golden test evidence exists. |
| `E3-pipeline` | Source passes `check -> compile -> verify -> run-smc` or equivalent full path. |
| `E4-release-gate` | Covered by stable qualification gate / 7hell / release-facing fixture. |

Rules:

- `confirmed-working` requires at least `E2-test`.
- Practical language features should target `E3-pipeline` before PCC closure.
- `E0-doc` cannot justify implementation readiness.
- `E1-code` is not enough for stable readiness.

## 6. Tested path vocabulary

Use one or more of the following tested paths:

```text
parse
parse -> typecheck
parse -> typecheck -> lower
parse -> typecheck -> lower -> emit
check -> compile -> verify
check -> compile -> verify -> run-smc
check -> lint -> compile -> verify -> run-smc -> disasm
unit test
integration test
golden fixture
manual inspection only
```

If the tested path is manual inspection only, the status cannot be
`confirmed-working` unless there is separate test evidence.

## 7. Audit matrix schema

Use this schema for every row:

| Field | Required | Meaning |
|---|---:|---|
| `id` | yes | Stable audit row id. |
| `area` | yes | Language/runtime/tooling area. |
| `feature` | yes | Feature or contract being audited. |
| `status` | yes | One allowed status. |
| `evidence` | yes | Evidence level and pointer. |
| `tested_path` | yes | Tested path vocabulary. |
| `missing_edge` | yes if partial | Exact gap. |
| `pcc_owner` | yes | PCC phase or `none`. |
| `ctf_impact` | yes | `none`, `value`, `trap`, `determinism`, `verifier`, `symbolid`, `capability`, `trace`. |
| `seven_hell_stage` | yes | Target 7hell stage or `none`. |
| `next_action` | yes | Audit, test, document, implement, exclude. |

## 8. Initial audit areas

The live audit should cover these areas first:

```text
A. Language surface
B. Type system
C. Control flow
D. Numeric core
E. Text core
F. Records
G. ADT / match
H. Option / Result
I. Collections
J. Stdlib
K. Modules / imports / exports
L. SemCode / verifier / VM
M. Runtime values / traps / determinism
N. CLI / project model
O. Examples / fixtures / golden tests
P. UI boundary status
Q. Experimental / legacy substrates
```

## 9. Starter matrix

This starter matrix is intentionally conservative. `unknown` means the row must
be audited against current `main` before it can support planning claims.

| id | area | feature | status | evidence | tested_path | missing_edge | pcc_owner | ctf_impact | seven_hell_stage | next_action |
|---|---|---|---|---|---|---|---|---|---|---|
| FM-001 | Language surface | `.sm` source format | unknown | TBD | TBD | TBD | PCC-0.5 | none | Syntax Hell | audit |
| FM-002 | Language surface | `fn` declarations | unknown | TBD | TBD | TBD | PCC-0.5 | none | Syntax Hell | audit |
| FM-003 | Language surface | `let` bindings | unknown | TBD | TBD | TBD | PCC-0.5 | value | Type Hell | audit |
| FM-004 | Language surface | `let mut` | unknown | TBD | TBD | TBD | PCC-0.5 / PCC-1 | value | Type Hell | audit |
| FM-005 | Language surface | reassignment | unknown | TBD | TBD | TBD | PCC-0.5 / PCC-1 | value | Type Hell | audit |
| FM-006 | Control flow | `if / else` | confirmed-working | `E2-test: crates/sm-front/src/parser.rs::rustlike_parser_accepts_if_expression; E2-test: tests/bytecode_compat.rs::compat_i32_value_path_runs_under_v0_header; E0-doc: docs/roadmap/full_readiness_non_ui.md` | integration test | none | PCC-1 | determinism | Syntax Hell | none |
| FM-007 | Control flow | `while` | confirmed-working | `E2-test: crates/sm-front/src/typecheck.rs::while_statement_with_bool_condition_typechecks; E2-test: tests/fixtures/snake_benchmark/positive_while_loop.sm; E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end` | check -> compile -> verify -> run-smc | none | PCC-1 | determinism | Syntax Hell | none |
| FM-008 | Control flow | statement `loop` | confirmed-working | `E2-test: crates/sm-front/src/typecheck.rs::statement_loop_with_continue_and_bare_break_typechecks; E2-test: tests/fixtures/snake_benchmark/positive_loop_control.sm; E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end` | check -> compile -> verify -> run-smc | none | PCC-1 | determinism | Syntax Hell | none |
| FM-009 | Control flow | `break` | confirmed-working | `E2-test: crates/sm-front/src/typecheck.rs::bare_break_outside_loop_rejects; E2-test: tests/fixtures/snake_benchmark/negative_loop_break.sm; E2-test: tests/fixtures/snake_benchmark/positive_loop_control.sm` | integration test | none | PCC-1 | trap / determinism | Syntax Hell | none |
| FM-010 | Control flow | `continue` | confirmed-working | `E2-test: crates/sm-front/src/typecheck.rs::continue_outside_loop_rejects; E2-test: tests/fixtures/snake_benchmark/negative_continue_statement.sm; E2-test: tests/fixtures/snake_benchmark/positive_loop_control.sm` | integration test | none | PCC-1 | determinism | Syntax Hell | none |
| FM-011 | Type system | `quad` | unknown | TBD | TBD | TBD | PCC-0.5 | value | Type Hell | audit |
| FM-012 | Type system | `bool` | unknown | TBD | TBD | TBD | PCC-0.5 | value | Type Hell | audit |
| FM-013 | Numeric core | `i32` arithmetic | unknown | TBD | TBD | TBD | PCC-2 | value/trap | VM Hell | audit |
| FM-014 | Numeric core | `u32` arithmetic | unknown | TBD | TBD | TBD | PCC-2 | value/trap | VM Hell | audit |
| FM-015 | Numeric core | `fx` value behavior | unknown | TBD | TBD | TBD | PCC-2 | value/trap | VM Hell | audit |
| FM-016 | Text core | text literal | unknown | TBD | TBD | TBD | PCC-3 | value | Practical Hell | audit |
| FM-017 | Text core | text equality | unknown | TBD | TBD | TBD | PCC-3 | value | Practical Hell | audit |
| FM-018 | Text core | text concat | unknown | TBD | TBD | TBD | PCC-3 | value | Practical Hell | audit |
| FM-019 | Records | record declaration | confirmed-working | `E3-pipeline: tests/pcc4_records_acceptance.rs::pcc4_record_declaration_fixture_passes_full_cli_path; E2-test: tests/pcc4_records_diagnostics.rs::pcc4_records_unknown_type_rejects_and_does_not_verify` | check -> run -> compile -> verify | none | PCC-4 | value | Practical Hell | closed for PCC-4 current scope |
| FM-020 | Records | record construction | confirmed-working | `E3-pipeline: tests/pcc4_records_acceptance.rs::pcc4_record_construction_and_field_read_fixture_passes_full_cli_path; E2-test: tests/pcc4_records_diagnostics.rs::pcc4_records_missing_required_field_rejects_and_does_not_verify; E2-test: tests/pcc4_records_diagnostics.rs::pcc4_records_duplicate_field_rejects_and_does_not_verify` | check -> run -> compile -> verify | none | PCC-4 | value | Practical Hell | closed for PCC-4 current scope |
| FM-021 | Records | field access | confirmed-working | `E3-pipeline: tests/pcc4_records_acceptance.rs::pcc4_record_construction_and_field_read_fixture_passes_full_cli_path; E3-pipeline: tests/pcc4_records_acceptance.rs::pcc4_record_function_boundary_fixture_passes_full_cli_path; E2-test: tests/pcc4_records_diagnostics.rs::pcc4_records_unknown_field_access_rejects_and_does_not_verify` | check -> run -> compile -> verify | none | PCC-4 | value | Practical Hell | closed for PCC-4 current scope |
| FM-022 | ADT / match | enum declaration | confirmed-working | `E3-pipeline: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end; E2-test: crates/sm-front/src/parser.rs::rustlike_parser_accepts_top_level_enum_and_constructor_surface; E2-test: crates/sm-front/src/typecheck.rs::adt_constructor_surface_typechecks_for_nominal_return_and_local_bindings` | integration test | none | PCC-5 | value | Practical Hell | closed for PCC-5 current scope |
| FM-023 | ADT / match | constructor expression | confirmed-working | `E3-pipeline: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end; E2-test: crates/sm-front/src/parser.rs::rustlike_parser_accepts_top_level_enum_and_constructor_surface; E2-test: crates/sm-ir/src/legacy_lowering.rs::lower_enum_constructor_to_make_adt_ir; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_runs_stage1_enum_constructor_path` | integration test | none | PCC-5 | value | Practical Hell | closed for PCC-5 current scope |
| FM-024 | ADT / match | basic ADT match | confirmed-working | `E3-pipeline: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end; E2-test: crates/sm-front/src/parser.rs::rustlike_parser_accepts_adt_match_expression_surface; E2-test: crates/sm-front/src/typecheck.rs::adt_match_expression_typechecks_with_payload_bindings; E2-test: crates/sm-ir/src/legacy_lowering.rs::lower_adt_match_expression_to_tag_and_payload_ir; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_runs_stage1_adt_match_path` | integration test | none | PCC-5 | determinism | Practical Hell | closed for PCC-5 current scope |
| FM-025 | Option / Result | `Option` shape | confirmed-working | `E3-pipeline: tests/pcc6_option_acceptance.rs::pcc6_option_some_match_fixture_passes_full_cli_path; E2-test: tests/pcc6_option_acceptance.rs::pcc6_option_none_match_fixture_passes_full_cli_path; E2-test: tests/pcc6_option_acceptance.rs::pcc6_option_function_boundary_fixture_passes_full_cli_path` | check -> compile -> verify -> run-smc | none | PCC-6 | value | Practical Hell | closed for PCC-6 current Option scope |
| FM-026 | Option / Result | `Result` shape | confirmed-working | `E3-pipeline: tests/pcc6_result_acceptance.rs::pcc6_result_ok_match_fixture_passes_full_cli_path; E2-test: tests/pcc6_result_acceptance.rs::pcc6_result_err_match_fixture_passes_full_cli_path; E2-test: tests/pcc6_result_acceptance.rs::pcc6_result_function_boundary_fixture_passes_full_cli_path` | check -> compile -> verify -> run-smc | none | PCC-6 | value | Practical Hell | closed for PCC-6 current Result scope |
| FM-027 | Collections | `Sequence<T>` | confirmed-working | `E3-pipeline: tests/pcc7_sequence_acceptance.rs::pcc7_sequence_indexing_fixture_passes_full_cli_path; E3-pipeline: tests/pcc7_sequence_acceptance.rs::pcc7_sequence_iteration_fixture_passes_full_cli_path; E3-pipeline: tests/pcc7_sequence_acceptance.rs::pcc7_sequence_len_empty_contains_fixture_passes_full_cli_path; E3-pipeline: tests/pcc7_sequence_acceptance.rs::pcc7_sequence_push_prepend_fixture_passes_full_cli_path; E3-pipeline: tests/pcc7_sequence_acceptance.rs::pcc7_sequence_pop_fixture_passes_full_cli_path; E3-pipeline: tests/pcc7_sequence_acceptance.rs::pcc7_sequence_function_boundary_fixture_passes_full_cli_path; E3-pipeline: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end` | check -> compile -> verify -> run-smc | none | PCC-7 | value/trap | Practical Hell | keep dedicated Sequence fixtures stable |
| FM-028 | Collections | `Map<K,V>` | confirmed-partial | `E3-pipeline: tests/pcc7_map_acceptance.rs::pcc7_map_empty_contextual_fixture_passes_full_cli_path; E3-pipeline: tests/pcc7_map_acceptance.rs::pcc7_map_basic_insert_lookup_fixture_passes_full_cli_path; E3-pipeline: tests/pcc7_map_acceptance.rs::pcc7_map_persistent_update_fixture_passes_full_cli_path; E2-test: tests/pcc7_collections_diagnostics.rs::pcc7_map_empty_requires_context_rejects_and_does_not_verify; E2-test: tests/pcc7_collections_diagnostics.rs::pcc7_map_key_type_mismatch_rejects_and_does_not_verify; E2-test: tests/pcc7_collections_diagnostics.rs::pcc7_map_value_type_mismatch_rejects_and_does_not_verify; E2-test: tests/pcc7_collections_diagnostics.rs::pcc7_sequence_index_out_of_bounds_traps_deterministically; E2-test: tests/pcc7_collections_diagnostics.rs::pcc7_sequence_pop_empty_traps_deterministically; E3-pipeline: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end; E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_negative_gap_suite_reports_current_blockers; E0-doc: docs/roadmap/language_maturity/first_wave_map_surface.md; E0-doc: docs/roadmap/language_maturity/core_trust_freeze/runtime_value_registry.md` | check -> compile -> verify -> run-smc | Map missing-key behavior, Map iteration policy, assignment / aliasing policy, and memory/quota evidence are still missing | PCC-7 | value/trap | Practical Hell | keep bounded-open policy note until explicit evidence lands |
| FM-029 | Stdlib | `assert` | confirmed-working | `E3-pipeline: tests/pcc8_stdlib_acceptance.rs::pcc8_assert_true_fixture_passes_full_cli_path; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_failed_assert; E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_runtime_negative_reports_contract_violations; E0-doc: docs/roadmap/language_maturity/core_trust_freeze/trap_taxonomy.md` | check -> compile -> verify -> run-smc | none | PCC-8 | trap | User Pain / Diagnostics Hell | keep helper contract and failure wording stable |
| FM-030 | Stdlib | `to_text` | confirmed-working | `E3-pipeline: tests/pcc8_stdlib_acceptance.rs::pcc8_to_text_basic_types_fixture_passes_full_cli_path; E3-pipeline: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end; E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_negative_gap_suite_reports_current_blockers; E0-doc: docs/roadmap/language_maturity/pcc3_text_closeout.md; E0-doc: docs/roadmap/language_maturity/core_trust_freeze/verifier_first_policy.md` | check -> compile -> verify -> run-smc | none | PCC-8 | value | Practical Hell | keep admitted types and public contract stable |
| FM-031 | Modules | import surface | unknown | TBD | TBD | TBD | PCC-0.5 | none | Syntax Hell | audit |
| FM-032 | Modules | export / re-export surface | unknown | TBD | TBD | TBD | PCC-0.5 | none | Syntax Hell | audit |
| FM-033 | Execution | verifier admission | confirmed-working | `E0-doc: docs/spec/semcode.md; E1-code: crates/sm-vm/src/semcode_vm.rs::run_verified_semcode_with_entry_and_config; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_malformed_semcode_rejects_before_execution` | check -> compile -> verify | none | CTF | verifier | Verifier Hell | none |
| FM-034 | Execution | VM execution path | confirmed-working | `E1-code: crates/sm-vm/src/semcode_vm.rs::run_verified_semcode_with_entry_and_config; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_stage_summaries_match_current_baseline; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_repeated_compiles_are_byte_stable` | check -> compile -> verify -> run-smc | none | CTF | determinism | VM Hell | none |
| FM-035 | Execution | trap taxonomy | confirmed-working | `E0-doc: docs/roadmap/language_maturity/core_trust_freeze/trap_taxonomy.md; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_unknown_opcode_on_load; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_failed_assert; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_write_after_borrow_same_path; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_fx_division_by_zero; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_stack_depth; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_register_budget; E2-test: crates/sm-vm/src/semcode_vm.rs::verified_run_rejects_invalid_bytecode_before_execution; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_malformed_semcode_rejects_before_execution` | integration test | none | CTF | trap | VM Hell | none |
| FM-036 | Execution | determinism matrix | confirmed-working | `E0-doc: docs/roadmap/language_maturity/core_trust_freeze/determinism_matrix.md; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_repeated_compiles_are_byte_stable; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_stage_summaries_match_current_baseline; E2-test: tests/canonical_examples.rs::canonical_positive_examples_check_run_compile_and_verify; E2-test: tests/smc_run_smc_cli.rs::smc_run_smc_executes_emitted_semcode_artifact; E2-test: tests/runtime_ownership_e2e.rs::runtime_ownership_rejects_same_path_write_deterministically; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_fx_division_by_zero; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_failed_assert; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_write_after_borrow_same_path; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_stack_depth; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_register_budget; E2-test: crates/sm-vm/src/semcode_vm.rs::verified_run_rejects_invalid_bytecode_before_execution` | integration test | none | CTF | determinism | VM Hell | none |
| FM-037 | CLI | `smc check` | confirmed-working | `E1-code: crates/smc-cli/src/app.rs::cmd_check; E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_positive_surface_passes_end_to_end` | unit test | none | PCC-0.5 | none | Practical Hell | none |
| FM-038 | CLI | `smc compile` | confirmed-working | `E1-code: crates/smc-cli/src/app.rs::cmd_compile; E2-test: tests/canonical_examples.rs::canonical_positive_examples_check_run_compile_and_verify` | unit test | none | PCC-0.5 | none | Practical Hell | none |
| FM-039 | CLI | `smc verify` | confirmed-working | `E1-code: crates/smc-cli/src/app.rs::cmd_verify; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_stage_summaries_match_current_baseline` | unit test | none | PCC-0.5 | verifier | Verifier Hell | none |
| FM-040 | CLI | `smc run-smc` | confirmed-working | `E2-test: tests/smc_run_smc_cli.rs::smc_run_smc_executes_emitted_semcode_artifact; E1-code: crates/smc-cli/src/app.rs::cmd_run_smc` | check -> compile -> verify -> run-smc | none | PCC-0.5 | none | Practical Hell | none |
| FM-041 | CLI | project model v0 | unknown | TBD | TBD | TBD | PCC-9 | none | Practical Hell | audit |
| FM-042 | Examples | canonical full-pipeline examples | confirmed-working | `E2-test: tests/canonical_examples.rs::canonical_positive_examples_check_run_compile_and_verify; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_stage_summaries_match_current_baseline` | check -> compile -> verify -> run-smc | none | PCC-0.5 | trace | Practical Hell | none |
| FM-043 | UI boundary | UI docs phase v0 | confirmed-working | PR-PCC-0A / I67-I69 docs closure | documentation audit | none | none | none | none | keep frozen |
| FM-044 | UI boundary | Workbench implementation | out-of-scope | PCC-0 truth reset | not applicable | frozen | post-PCC | none | none | exclude |
| FM-045 | Experimental | sm-quad / packed quad substrate | unknown | TBD | TBD | TBD | post-PCC / experimental | value | none | audit as experimental |

## 10. Audit procedure

For each row:

1. inspect current `main`;
2. locate code / tests / docs / fixtures;
3. classify evidence level;
4. run or cite tested path where available;
5. resolve status;
6. write missing edge if partial;
7. assign owner phase;
8. mark CTF impact;
9. mark 7hell stage;
10. set next action.

Do not mark a row `confirmed-working` because it was discussed, planned,
merged historically, or assumed from memory.

## 11. Output requirement

PCC-0.5 should leave a filled matrix in one of these forms:

```text
docs/roadmap/language_maturity/practical_core_feature_matrix_live_audit.md
```

or, if the matrix becomes too large:

```text
docs/roadmap/language_maturity/feature_matrix/
  index.md
  language_surface.md
  execution_core.md
  tooling.md
  experimental.md
```

The first pass may remain in this single file. Split only when the file becomes
hard to review.

## 12. Blocking rule before PCC-1

PCC-1 must not start while any of the following are true:

```text
[ ] control-flow rows are still unknown
[ ] verifier / VM rows are still unknown
[ ] trap / determinism rows are still unknown
[ ] CLI check/compile/verify/run-smc rows are still unknown
[ ] canonical example status is unknown
```

PCC-1 may start with known partials only if each partial has:

- exact missing edge;
- owner phase;
- test need;
- CTF impact;
- 7hell stage.

## 13. Out of scope

This audit scaffold does not perform implementation.

Out of scope:

- fixing features;
- adding tests;
- changing parser / typecheck / IR / VM;
- changing CLI behavior;
- implementing 7hell;
- starting Workbench;
- starting I70;
- changing UI runtime docs except reference corrections.

## 14. Acceptance checklist

This PR is complete when:

- PCC-0.5 has a documented purpose;
- allowed status vocabulary is defined;
- evidence levels are defined;
- tested path vocabulary is defined;
- audit matrix schema is defined;
- starter rows exist for practical core features;
- blocking rule before PCC-1 is explicit;
- UI / Workbench remains frozen;
- CTF impact field is required;
- 7hell stage field is required;
- no code is changed.

## 15. PCC-0D audit pass 1 summary

| Group | Resolved | Still unknown | Main blocker |
|---|---:|---:|---|
| Control flow | 5 | 0 | none |
| Execution trust | 4 | 0 | none |
| CLI | 4 | 0 | none |
| Examples | 1 | 0 | none |

PCC-1 start status: `conditionally unblockable`
Reason: all PCC-0D blocker rows are resolved, but PCC-1 start still requires
maintainer acceptance of the audit state and no newer mainline changes
invalidate the blocker matrix.

## 16. PCC-0H blocker audit gate closure

| Gate item | Status | Evidence |
|---|---|---|
| Control-flow blockers FM-006..FM-010 | closed | PCC-0D |
| Verifier / VM blockers FM-033..FM-034 | closed | PCC-0D |
| Trap taxonomy FM-035 | closed | PCC-0F |
| Determinism matrix FM-036 | closed | PCC-0G |
| CLI blockers FM-037..FM-040 | closed | PCC-0D / PCC-0E |
| Canonical examples FM-042 | closed | PCC-0D |
| UI / Workbench freeze | still frozen | PCC-0A |
| 7hell qualification path | seeded, not implemented | PCC-0C |

PCC-0 blocker audit gate: `closed`
PCC-1 start status: `conditionally unblockable`

Reason:
All PCC-0D blocker rows have been resolved, but PCC-1 still requires
maintainer acceptance and must remain constrained by CTF and 7hell growth
rules.

## 17. Final state

After this scaffold exists:

```text
PCC-0 blocker audit gate = closed
PCC-0.5 Live Audit = scaffolded and blocker pass closed
PCC-0.6 7hell Qualification Contract = seeded
PCC-1 = conditionally unblockable after maintainer acceptance
CTF = parallel trust lane
7hell = qualification growth path
```
