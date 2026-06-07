# Examples Index

Status: current-main index for the curated canonical examples pack

## Purpose

This index maps the current canonical examples to their intent, current reading,
and recommended first command.

The canonical examples pack lives in:

- `examples/canonical/`

The older planning-only pack remains in:

- `examples/readiness_draft_canonical/`

The draft pack is historical context. The canonical pack is the current
onboarding and readiness-facing examples surface.

## Canonical Examples

### `cli_batch_core`

- path: `examples/canonical/cli_batch_core/`
- purpose: small CLI-style computation core over `Sequence(i32)` and `text`
- current reading: `qualified limited release`
- first command:

```powershell
cargo run --bin smc -- run examples/canonical/cli_batch_core/src/main.sm
```

### `rule_state_decision`

- path: `examples/canonical/rule_state_decision/`
- purpose: record-oriented rule/state decision logic with explicit `Result(T, E)` handling
- current reading: `qualified limited release`
- first command:

```powershell
cargo run --bin smc -- run examples/canonical/rule_state_decision/src/main.sm
```

### `data_audit_record_iterable`

- path: `examples/canonical/data_audit_record_iterable/`
- purpose: direct-record `Iterable` data traversal and audit-style processing
- current reading: `qualified limited release`
- first command:

```powershell
cargo run --bin smc -- run examples/canonical/data_audit_record_iterable/src/main.sm
```

### `wave2_local_helper_import`

- path: `examples/canonical/wave2_local_helper_import/`
- purpose: admitted helper-module executable authoring with direct local-path bare import
- current reading: `qualified limited release`
- first command:

```powershell
cargo run --bin smc -- check examples/canonical/wave2_local_helper_import/src/main.sm
```

### `positive_selected_import`

- path: `examples/canonical/positive_selected_import/`
- purpose: admitted helper-module executable authoring with direct local-path selected import
- current reading: `qualified limited release`
- first command:

```powershell
cargo run --bin smc -- check examples/canonical/positive_selected_import/src/main.sm
```

### `boundary_alias_import`

- path: `examples/canonical/boundary_alias_import/`
- purpose: intentional boundary example showing that executable-path alias import is still rejected
- current reading: `out of scope`
- note: this is not a supported workflow; it documents a current executable-module / alias-import limit in the current baseline
- first command:

```powershell
cargo run --bin smc -- check examples/canonical/boundary_alias_import/src/main.sm
```

Expected result:

- this example should fail with the current executable import boundary diagnostic
- it should not be treated as a failing canonical success example
- future support requires an explicit language/source-admission change

## Validation

The canonical examples pack is covered by:

```powershell
cargo test -q --test canonical_examples
```
