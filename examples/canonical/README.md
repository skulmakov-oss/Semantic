# Canonical Examples Pack

Status: finalized canonical examples pack for `PR-D1`

## Purpose

This directory publishes the curated examples pack used by the current
readiness contour.

It replaces the earlier planning-only pack in:

- `examples/readiness_draft_canonical/`

This pack is intentionally split into:

- five positive examples inside the current `qualified limited release` contour
- one boundary example that shows a still-real limit honestly

This pack is also the canonical `.sm` sample surface intended for future GitHub
Linguist review. The five positive examples are small, readable, and stable
enough to serve as representative language samples:

- `cli_batch_core`
- `rule_state_decision`
- `data_audit_record_iterable`
- `wave2_local_helper_import`
- `positive_selected_import`

The boundary example remains included as an honest exclusion marker and should
not be treated as a positive sample for Linguist detection.

## Canonical Examples

1. `cli_batch_core`
   - purpose: small CLI-style computation core over `Sequence(i32)` and `text`
   - current reading: `qualified limited release`

2. `rule_state_decision`
   - purpose: record-oriented rule/state decision logic with explicit
     `Result(T, E)` handling
   - current reading: `qualified limited release`

3. `data_audit_record_iterable`
   - purpose: data-heavy audit pass over direct-record `Iterable` dispatch
   - current reading: `qualified limited release`

4. `wave2_local_helper_import`
   - purpose: admitted helper-module executable authoring with direct local-path
     bare import
   - current reading: `qualified limited release`

5. `positive_selected_import`
   - purpose: admitted helper-module executable authoring with direct local-path
     selected import over the current function-only helper slice
   - current reading: `qualified limited release`

6. `boundary_alias_import`
   - purpose: honest boundary example showing that top-level alias import on the
     executable path is still rejected
   - current reading: `out of scope`

## Validation

Canonical examples are validated by:

```text
cargo test -q --test canonical_examples
```

Positive examples are checked, compiled, verified, and run through the public
`smc` command surface.

The boundary example is checked to ensure the current diagnostic remains
explicit and deterministic.
