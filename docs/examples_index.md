# Examples Index

Status: current-main index for the curated canonical examples pack

## Purpose

This index maps the current canonical examples to a recommended first
command. For purpose, source profile, qualification level, expected result,
and Source Style v0 migration status, see the **authoritative inventory
table** in
[`examples/canonical/README.md`](../examples/canonical/README.md#canonical-examples--authoritative-inventory).
This index does not duplicate those fields — update the table there, and this
page stays correct by reference.

The canonical examples pack lives in:

- `examples/canonical/`

The older planning-only pack remains in:

- `examples/readiness_draft_canonical/`

The draft pack is historical context. The canonical pack is the current
onboarding and readiness-facing examples surface.

`match_control_flow` and `rule_state_decision` demonstrate the frozen
[Canonical Source Style v0](spec/source_style.md); `quad_cycle_logos`
demonstrates the same contract's Logos declarative-profile presentation
rules.

## First Commands

| Example | First command |
|---|---|
| [`cli_batch_core`](../examples/canonical/cli_batch_core/) | `cargo run --bin smc -- run examples/canonical/cli_batch_core/src/main.sm` |
| [`rule_state_decision`](../examples/canonical/rule_state_decision/) | `cargo run --bin smc -- run examples/canonical/rule_state_decision/src/main.sm` |
| [`data_audit_record_iterable`](../examples/canonical/data_audit_record_iterable/) | `cargo run --bin smc -- run examples/canonical/data_audit_record_iterable/src/main.sm` |
| [`text_collections_toolbox`](../examples/canonical/text_collections_toolbox/) | `cargo run --bin smc -- run examples/canonical/text_collections_toolbox/src/main.sm` |
| [`stdlib_v0_helpers`](../examples/canonical/stdlib_v0_helpers/) | `cargo run --bin smc -- run examples/canonical/stdlib_v0_helpers/src/main.sm` |
| [`collections_core`](../examples/canonical/collections_core/) | `cargo run --bin smc -- run examples/canonical/collections_core/src/main.sm` |
| [`text_core`](../examples/canonical/text_core/) | `cargo run --bin smc -- run examples/canonical/text_core/src/main.sm` |
| [`match_control_flow`](../examples/canonical/match_control_flow/) | `cargo run --bin smc -- run examples/canonical/match_control_flow/src/main.sm` |
| [`option_result_control_flow`](../examples/canonical/option_result_control_flow/) | `cargo run --bin smc -- run examples/canonical/option_result_control_flow/src/main.sm` |
| [`loop_control_flow`](../examples/canonical/loop_control_flow/) | `cargo run --bin smc -- run examples/canonical/loop_control_flow/src/main.sm` |
| [`wave2_local_helper_import`](../examples/canonical/wave2_local_helper_import/) | `cargo run --bin smc -- check examples/canonical/wave2_local_helper_import/src/main.sm` |
| [`positive_selected_import`](../examples/canonical/positive_selected_import/) | `cargo run --bin smc -- check examples/canonical/positive_selected_import/src/main.sm` |
| [`boundary_alias_import`](../examples/canonical/boundary_alias_import/) (intentional rejection — not a positive sample) | `cargo run --bin smc -- check examples/canonical/boundary_alias_import/src/main.sm` |
| [`quad_cycle_logos`](../examples/canonical/quad_cycle_logos/) (Logos profile) | `cargo run --bin smc -- dump-ast examples/canonical/quad_cycle_logos/src/main.sm` |

## SSF-11 Application Corpus (Semantic By Example)

The SSF-11 (#1582) corpus answers, for each #1582 application family: what
the example proves, which profile it uses, its maturity, the command to run,
the deterministic result, which contract owns the behavior, and whether it is
Foundation qualification, boundary evidence, or historical evidence.

- Machine-readable index: [`examples/qualification/ssf11/corpus.json`](../examples/qualification/ssf11/corpus.json)
- Full matrix: [`ssf11_application_onboarding_matrix.md`](roadmap/stable_foundation/ssf11_application_onboarding_matrix.md)

| Family | Example | Profile | Maturity | Command | Deterministic result | Contract owner | Role |
|---|---|---|---|---|---|---|---|
| F01 minimal | [`f01_minimal`](../examples/qualification/ssf11/f01_minimal/main.sm) | `pure` | Landed and qualified on `main` | `smc check`, `compile -o`, `verify`, `run-smc` | exit 0 at every step | `docs/spec/cli.md` | Foundation + onboarding |
| F02 quad | [`f02_quad_decision`](../examples/qualification/ssf11/f02_quad_decision/main.sm) | `pure` | Landed and qualified on `main` | `smc run` | exit 0; `T && F` is `N`, `T \|\| F` is `S` | `docs/spec/types.md` | Foundation |
| F02 quad (boundary) | [`f02_quad_bare_condition`](../examples/qualification/ssf11/f02_quad_bare_condition/main.sm) | `pure` | Landed and qualified on `main` | `smc check --format json` | exit 1, `E0201` | `docs/spec/branch_condition_quad_rule.md` | Foundation (negative) |
| F03 data | [`f03_records_enums_option_result`](../examples/qualification/ssf11/f03_records_enums_option_result/main.sm) | `pure` | Landed and qualified on `main` | `smc run` | exit 0 | `docs/spec/source_semantics.md` | Foundation |
| F04 collections | [`collections_core`](../examples/canonical/collections_core/) | `pure` | Landed and qualified on `main` | `smc run` | exit 0 | `docs/spec/foundation_stdlib_v0.md` | Foundation + onboarding |
| F05 generics (boundary) | [`f05_generic_boundary`](../examples/qualification/ssf11/f05_generic_boundary/main.sm) | `pure` | **Roadmap** (excluded) | `smc compile -o out.smc` | exit 1, no artifact: "concrete IR monomorphisation is not implemented" | SSF-07 matrix rows | Boundary only, not a positive |
| F06 project | [`f06_project`](../examples/qualification/ssf11/f06_project/) | `pure` | Landed and qualified on `main` | `smc test` | `ok tests/double.sm` | `docs/spec/project_model_v0.md` | Foundation + onboarding |
| F06 packages | [`f06_packages/app`](../examples/qualification/ssf11/f06_packages/app/) | `pure` | Landed and qualified on `main` | `smc package inspect` | deterministic provenance JSON | `docs/spec/package_baseline_v0.md` | Foundation + onboarding |
| F07 serialization (boundary) | [`f07_serialization_boundary`](../examples/qualification/ssf11/f07_serialization_boundary/main.sm) | `pure` | **Roadmap** (excluded) | `smc check` | exit 1, `E0201` unknown function | `standard_library_v0_evidence.md` | Boundary only, not a positive |
| F08 file transform | [`f08_file_transform`](../examples/qualification/ssf11/f08_file_transform/main.sm) | `cli-file-transform` | Landed and qualified on `main` | `smc run ... --profile cli-file-transform --root . -- input.txt output.txt` | `output.txt` = `transformed:hello` + newline | `docs/spec/controlled_application_boundary_v0.md` | Foundation + onboarding |
| F09 verifier rejection | [`f09_verifier_rejection`](../examples/qualification/ssf11/f09_verifier_rejection/unsupported_header.smc) | n/a | Landed and qualified on `main` | `smc verify` | exit 1, `[UnsupportedVersion]` | `docs/spec/verifier.md` | Foundation (negative) |
| F10 trap/quota | [`f10_runtime_failure`](../examples/qualification/ssf11/f10_runtime_failure/) | `pure` | Landed and qualified on `main` | `smc run` | `DivisionByZero`; `Steps limit=100000 used=100001` | `docs/spec/quotas.md` | Foundation (negative) |
| F11 benchmark | [`snake_core.sm`](../examples/benchmarks/snake_core.sm) | `pure` | Landed and qualified on `main` | `smc run` | `snake_core: score=0 steps=200` | `examples/benchmarks/README.md` | Foundation |
| F12 native application | none: retired Workbench/Studio/native UI | n/a | Out of scope | n/a | n/a | `docs/architecture/ui_boundary_index.md` | **Historical, non-qualifying** |

Every command above runs as `cargo run --bin smc -- <command> <path>` from
the repository root. Boundary and historical rows are not stable positives.
Nothing in this index is a published-stable claim. SSF-12 (#1583) has not
started.

## Validation

The canonical examples pack is covered by:

```powershell
cargo test -q --test canonical_examples
cargo test -q --test canonical_source_style
cargo test -q --test ssf11_canonical_applications
cargo test -q --test ssf11_onboarding_docs
```
