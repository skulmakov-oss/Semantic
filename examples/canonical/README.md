# Canonical Examples Pack

Status: finalized canonical examples pack for `PR-D1`

## Purpose

This directory publishes the curated examples pack used by the current
readiness contour.

It replaces the earlier planning-only pack in:

- `examples/readiness_draft_canonical/`

This pack is intentionally split into:

- twelve positive examples inside the current `qualified limited release` contour
- one boundary example that shows a still-real limit honestly

This pack is also the canonical `.sm` sample surface intended for future GitHub
Linguist review. The twelve positive examples are small, readable, and stable
enough to serve as representative language samples:

- `cli_batch_core`
- `rule_state_decision`
- `data_audit_record_iterable`
- `text_collections_toolbox`
- `stdlib_v0_helpers`
- `collections_core`
- `text_core`
- `match_control_flow`
- `option_result_control_flow`
- `loop_control_flow`
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

4. `text_collections_toolbox`
   - purpose: practical toolbox example for control flow, text, collections,
     and stdlib
   - current reading: `qualified limited release`

5. `stdlib_v0_helpers`
   - purpose: practical helper-surface example for current PCC stdlib v0
   - current reading: `qualified limited release`

6. `collections_core`
   - purpose: standalone practical collections surface example
   - current reading: `qualified limited release`

7. `text_core`
   - purpose: standalone practical `text` surface example
   - current reading: `qualified limited release`

8. `match_control_flow`
   - purpose: practical `match`-driven control-flow over `quad`
   - current reading: `qualified limited release`

9. `option_result_control_flow`
   - purpose: practical `Option` / `Result` control flow over the admitted
     public surface
   - current reading: `qualified limited release`

10. `loop_control_flow`
   - purpose: practical loop-driven control flow over admitted `while`,
     `loop`, `break`, and `continue`
   - current reading: `qualified limited release`

11. `wave2_local_helper_import`
   - purpose: admitted helper-module executable authoring with direct local-path
     bare import
   - current reading: `qualified limited release`

12. `positive_selected_import`
   - purpose: admitted helper-module executable authoring with direct local-path
     selected import over the current function-only helper slice
   - current reading: `qualified limited release`

13. `boundary_alias_import`
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
