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

## Validation

The canonical examples pack is covered by:

```powershell
cargo test -q --test canonical_examples
cargo test -q --test canonical_source_style
```
